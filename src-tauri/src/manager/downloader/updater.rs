use uuid::Uuid;

use super::{install_with_deps, InstallOptions, ModInstall};
use crate::{
    manager::{ModManager, Profile, Result},
    thunderstore::{BorrowedMod, ModRef, Thunderstore},
};
use anyhow::Context;
use itertools::Itertools;
use std::sync::Mutex;
use tauri::Manager;

pub mod commands;

pub struct AvailableUpdate<'a> {
    pub mod_ref: &'a ModRef,
    pub enabled: bool,
    pub index: usize,
    pub current_num: &'a semver::Version,
    pub latest: BorrowedMod<'a>,
}

impl From<AvailableUpdate<'_>> for ModInstall {
    fn from(value: AvailableUpdate<'_>) -> Self {
        ModInstall::new(value.latest.reference())
            .with_state(value.enabled)
            .at(value.index)
    }
}

impl Profile {
    pub fn update_available<'a>(
        &'a self,
        uuid: &Uuid,
        thunderstore: &'a Thunderstore,
    ) -> Result<Option<AvailableUpdate<'a>>> {
        let index = self.index_of(uuid)?;

        let (mod_ref, enabled) = match self.mods[index].as_remote() {
            Some(x) => x,
            None => return Ok(None),
        };

        let installed_vers = &mod_ref.borrow(thunderstore)?.version.version_number;

        let latest = thunderstore.get_package(uuid)?;

        let latest_version = latest
            .versions
            .first()
            .context("package should have at least one version")?;

        if installed_vers >= &latest_version.version_number {
            return Ok(None);
        }

        Ok(Some(AvailableUpdate {
            index,
            mod_ref,
            enabled,
            current_num: installed_vers,
            latest: BorrowedMod {
                package: latest,
                version: latest_version,
            },
        }))
    }

    pub fn available_updates<'a>(
        &'a self,
        thunderstore: &'a Thunderstore,
    ) -> impl Iterator<Item = Result<AvailableUpdate<'a>>> + 'a {
        self.remote_mods().filter_map(move |(m, _)| {
            self.update_available(&m.package_uuid, thunderstore)
                .transpose()
        })
    }
}

pub async fn change_version(mod_ref: ModRef, app: &tauri::AppHandle) -> Result<()> {
    let install = {
        let manager = app.state::<Mutex<ModManager>>();
        let thunderstore = app.state::<Mutex<Thunderstore>>();

        let mut manager = manager.lock().unwrap();
        let thunderstore = thunderstore.lock().unwrap();

        let profile = manager.active_profile_mut();
        let index = profile.index_of(&mod_ref.package_uuid)?;

        profile.force_remove_mod(&mod_ref.package_uuid, &thunderstore)?;

        ModInstall::new(mod_ref)
            .with_state(profile.mods[index].enabled)
            .at(index)
    };

    super::install_mod(install, InstallOptions::default().can_cancel(false), app).await
}

pub async fn update_mods(uuids: &[Uuid], app: &tauri::AppHandle) -> Result<()> {
    let to_update = {
        let manager = app.state::<Mutex<ModManager>>();
        let thunderstore = app.state::<Mutex<Thunderstore>>();

        let mut manager = manager.lock().unwrap();
        let thunderstore = thunderstore.lock().unwrap();

        let profile = manager.active_profile_mut();

        uuids
            .iter()
            .filter_map(|uuid| profile.update_available(uuid, &thunderstore).transpose())
            .map_ok(|update| update.into())
            .collect::<Result<Vec<ModInstall>>>()?
    };

    install_with_deps(
        to_update,
        InstallOptions::default().before_install(|install, manager, thunderstore| {
            let profile = manager.active_profile_mut();
            profile.force_remove_mod(install.uuid(), thunderstore).ok();
        }),
        true,
        app,
    )
    .await
}
