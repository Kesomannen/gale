use std::{
    collections::{HashMap, HashSet},
    fs,
};

use eyre::{anyhow, ensure, Context, OptionExt, Result};
use itertools::Itertools;
use log::info;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Listener};
use uuid::Uuid;

use super::{
    export::{self, IncludeExtensions, IncludeGenerated},
    import,
    install::PackageInstaller,
    Dependant, ManagedGame, Profile, ProfileMod,
};
use crate::{
    config::ConfigCache,
    db::Db,
    logger,
    state::ManagerExt,
    thunderstore::Thunderstore,
    util::{
        self,
        error::IoResultExt,
        fs::{Overwrite, UseLinks},
    },
};

pub fn setup(app: &AppHandle) -> Result<()> {
    let handle = app.to_owned();
    app.listen("reorder_mod", move |event| {
        if let Err(err) = handle_reorder_event(event, &handle) {
            logger::log_webview_err("Failed to reorder mod", err, &handle);
        }
    });

    let handle = app.to_owned();
    app.listen("finish_reorder", move |_| {
        if let Err(err) = handle_finish_reorder_event(&handle) {
            logger::log_webview_err("Failed to finish reordering", err, &handle);
        }
    });

    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum ActionResult {
    Done,
    Confirm { dependants: Vec<Dependant> },
}

impl Profile {
    pub fn rename(&mut self, name: String) -> Result<()> {
        ensure!(
            Self::is_valid_name(&name),
            "invalid profile name '{}'",
            name
        );

        let new_path = self.path.parent().unwrap().join(&name);

        ensure!(
            !new_path.exists(),
            "profile with name '{}' already exists",
            name
        );

        fs::rename(&self.path, &new_path).fs_context("renaming profile directory", &self.path)?;

        info!("renamed profile: {} -> {}", self.name, name);

        self.name = name;
        self.path = new_path;

        Ok(())
    }

    pub fn remove_mod(&mut self, uuid: Uuid, thunderstore: &Thunderstore) -> Result<ActionResult> {
        if self.get_mod(uuid)?.enabled {
            if let Some(dependants) = self.check_dependants(uuid, thunderstore) {
                return Ok(ActionResult::Confirm { dependants });
            }
        }

        self.force_remove_mod(uuid)?;
        Ok(ActionResult::Done)
    }

    pub fn force_remove_mod(&mut self, uuid: Uuid) -> Result<()> {
        let index = self.index_of(uuid)?;
        let profile_mod = &self.mods[index];

        self.installer_for(profile_mod)
            .uninstall(profile_mod, self)?;

        self.mods.remove(index);

        Ok(())
    }

    pub fn toggle_mod(&mut self, uuid: Uuid, thunderstore: &Thunderstore) -> Result<ActionResult> {
        let dependants = match self.get_mod(uuid)?.enabled {
            true => self.check_dependants(uuid, thunderstore),
            false => self.check_dependencies(uuid, thunderstore),
        };

        match dependants {
            Some(dependants) => Ok(ActionResult::Confirm { dependants }),
            None => {
                self.force_toggle_mod(uuid)?;
                Ok(ActionResult::Done)
            }
        }
    }

    pub fn force_toggle_mod(&mut self, uuid: Uuid) -> Result<()> {
        let profile_mod = self.get_mod(uuid)?;
        let enabled = profile_mod.enabled;

        self.installer_for(profile_mod)
            .toggle(enabled, profile_mod, self)?;

        self.get_mod_mut(uuid).unwrap().enabled = !enabled;

        Ok(())
    }

    fn check_dependants(&self, uuid: Uuid, thunderstore: &Thunderstore) -> Option<Vec<Dependant>> {
        let dependants = self
            .dependants(uuid, thunderstore)
            .filter(|profile_mod| {
                // ignore disabled mods and modpacks
                profile_mod.enabled
                    && profile_mod
                        .as_thunderstore()
                        .and_then(|(ts_mod, _)| {
                            ts_mod
                                .id
                                .borrow(thunderstore)
                                .map(|borrowed| !borrowed.package.is_modpack())
                                .ok()
                        })
                        .unwrap_or(true)
            })
            .map_into()
            .collect_vec();

        match dependants.is_empty() {
            true => None,
            false => Some(dependants),
        }
    }

    /// Finds disabled dependencies in the profile.
    fn check_dependencies(
        &self,
        uuid: Uuid,
        thunderstore: &Thunderstore,
    ) -> Option<Vec<Dependant>> {
        let disabled_deps = self
            .get_mod(uuid)
            .ok()?
            .dependencies(thunderstore)
            .filter(|dep| {
                self.get_mod(dep.package.uuid)
                    .is_ok_and(|profile_mod| !profile_mod.enabled)
            })
            .map_into()
            .collect_vec();

        match disabled_deps.is_empty() {
            true => None,
            false => Some(disabled_deps),
        }
    }

    pub fn open_mod_dir(&self, uuid: Uuid) -> Result<()> {
        let profile_mod = self.get_mod(uuid)?;

        if let Some(path) = self
            .installer_for(profile_mod)
            .mod_dir(&profile_mod.full_name(), self)
        {
            open::that(path)?;
            Ok(())
        } else {
            Err(anyhow!("unsupported"))
        }
    }

    fn installer_for(&self, profile_mod: &ProfileMod) -> Box<dyn PackageInstaller> {
        self.game.mod_loader.installer_for(&profile_mod.full_name())
    }

    fn reorder_mod(&mut self, uuid: Uuid, delta: i32) -> Result<()> {
        let index = self
            .mods
            .iter()
            .position(|m| m.uuid() == uuid)
            .ok_or_eyre("mod not found in profile")?;

        let target = (index as i32 + delta).clamp(0, self.mods.len() as i32 - 1) as usize;
        let profile_mod = self.mods.remove(index);
        self.mods.insert(target, profile_mod);

        Ok(())
    }
}

fn handle_reorder_event(event: tauri::Event, app: &AppHandle) -> Result<()> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Payload {
        uuid: Uuid,
        delta: i32,
    }

    let Payload { uuid, delta } = serde_json::from_str(event.payload())?;

    let mut manager = app.lock_manager();
    manager.active_profile_mut().reorder_mod(uuid, delta)?;

    Ok(())
}

fn handle_finish_reorder_event(app: &AppHandle) -> Result<()> {
    app.lock_manager().save_active_profile(app.db())
}

impl ManagedGame {
    pub fn create_profile(&mut self, name: String, db: &Db) -> Result<&mut Profile> {
        ensure!(
            Profile::is_valid_name(&name),
            "profile name '{}' is invalid",
            name
        );

        let mut path = self.path.join("profiles");
        path.push(&name);

        ensure!(
            !path.exists(),
            "profile with name '{}' already exists",
            name
        );

        fs::create_dir_all(&path).fs_context("creating profile directory", &path)?;

        let id = db.next_profile_id()?;

        self.profiles.push(Profile {
            id,
            name,
            path,
            mods: Vec::new(),
            game: self.game,
            ignored_updates: HashSet::new(),
            config_cache: ConfigCache::default(),
            linked_config: HashMap::new(),
            modpack: None,
        });

        self.active_profile_id = id;
        Ok(self.active_profile_mut())
    }

    pub fn delete_profile(&mut self, index: usize, allow_delete_last: bool) -> Result<()> {
        ensure!(
            allow_delete_last || self.profiles.len() > 1,
            "cannot delete last profile"
        );

        let profile = self.profile_at(index)?;

        fs::remove_dir_all(&profile.path)?;
        self.profiles.remove(index);

        if !self.profiles.is_empty() {
            self.active_profile_id = self.profiles[0].id;
        }

        Ok(())
    }

    pub fn duplicate_profile(&mut self, duplicate_name: String, id: i64, db: &Db) -> Result<()> {
        self.create_profile(duplicate_name, db)?;

        let old_profile = self.find_profile(id)?;
        let new_profile = self.active_profile();

        // Make sure generated files and configs are properly copied
        // and not linked between the two profiles.
        let config_files = export::find_config(
            &old_profile.path,
            IncludeExtensions::Default,
            IncludeGenerated::Yes,
        );
        import::import_config(&new_profile.path, &old_profile.path, config_files)
            .context("failed to copy config files")?;

        util::fs::copy_dir(
            &old_profile.path,
            &new_profile.path,
            Overwrite::No, // don't override the copied mutable files
            UseLinks::Yes,
        )
        .context("failed to copy profile directory")?;

        let mods = old_profile.mods.clone();
        let ignored_updates = old_profile.ignored_updates.clone();

        let new_profile = self.active_profile_mut();
        new_profile.mods = mods;
        new_profile.ignored_updates = ignored_updates;

        Ok(())
    }
}
