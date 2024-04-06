use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::Mutex,
};

use anyhow::{bail, ensure, Context, Result};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use uuid::Uuid;

use crate::{
    prefs::Prefs,
    thunderstore::{
        query::{self, QueryModsArgs}, BorrowedMod, ModRef, OwnedMod, Thunderstore
    }, util::IoResultExt,
};
use tauri::{AppHandle, Manager};

pub mod commands;
pub mod config;
pub mod downloader;
pub mod importer;

pub fn setup(app: &AppHandle) -> Result<()> {
    let prefs = app.state::<Mutex<Prefs>>();
    let prefs = prefs.lock().unwrap();

    let manager = ModManager::create(&prefs)?;
    app.manage(Mutex::new(manager));

    importer::setup(app).context("failed to initialize importer")?;
    downloader::setup(app).context("failed to initialize downloader")?;
    config::setup(app).context("failed to initialize mod config")?;

    Ok(())
}

pub struct ModManager {
    profiles: Vec<Profile>,
    active_profile_index: usize,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManagerSaveData {
    active_profile_index: usize,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileManifest {
    name: String,
    mods: Vec<ModRef>,
}

struct Profile {
    name: String,
    path: PathBuf,
    mods: Vec<ModRef>,
    config: Vec<config::LoadedFile>,
}

impl Profile {
    fn query_mods(
        &self,
        args: &QueryModsArgs,
        thunderstore: &Thunderstore,
    ) -> Result<Vec<OwnedMod>> {
        let mods = self
            .mods
            .iter()
            .map(|p| p.borrow(thunderstore))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(query::query_mods(args, mods.into_iter()))
    }

    fn get_mod<'a>(&'a self, package_uuid: &Uuid) -> Option<&'a ModRef> {
        self.mods.iter().find(|p| p.package_uuid == *package_uuid)
    }

    fn has_package(&self, package_uuid: &Uuid) -> bool {
        self.get_mod(package_uuid).is_some()
    }

    fn manifest(&self) -> ProfileManifest {
        ProfileManifest {
            name: self.name.clone(),
            mods: self.mods.clone(),
        }
    }

    fn dependants<'a>(
        &self,
        package_uuid: &Uuid,
        thunderstore: &'a Thunderstore,
    ) -> Result<Vec<BorrowedMod<'a>>> {
        let target_mod = self
            .get_mod(package_uuid)
            .context("mod not found in profile")?;

        let target_package = target_mod.borrow(thunderstore)?.package;

        self.mods
            .iter()
            .filter(|other| other.package_uuid != *package_uuid)
            .map(|other| other.borrow(thunderstore))
            .filter_map(|other| match other {
                Ok(other) => {
                    let deps = thunderstore
                        .dependencies(other.version)
                        .collect::<Result<Vec<_>>>()
                        .with_context(|| {
                            format!(
                                "failed to resolve dependencies of {}",
                                other.package.full_name
                            )
                        });

                    match deps {
                        Ok(deps) => match deps
                            .into_iter()
                            .any(|dep| dep.package.uuid4 == target_package.uuid4)
                        {
                            true => Some(Ok(other)),
                            false => None,
                        },
                        Err(err) => Some(Err(err)),
                    }
                }
                Err(_) => Some(other),
            }) // filter out packages that do not depend on the target one, while keeping errors
            .collect()
    }

    const GAME_ID: u32 = 1966720;

    fn run_game(&self, prefs: &Prefs) -> Result<()> {
        let steam_path = prefs
            .steam_exe_path
            .as_ref()
            .context("steam exe path not set")?;

        let steam_path = resolve_path(steam_path, "steam")?;

        let mut preloader_path = self.path.join("BepInEx");
        preloader_path.push("core");
        preloader_path.push("BepInEx.Preloader.dll");

        let preloader_path = resolve_path(&preloader_path, "preloader")?;

        Command::new(steam_path)
            .arg("-applaunch")
            .arg(Self::GAME_ID.to_string())
            .arg("--doorstop-enable")
            .arg("true")
            .arg("--doorstop-target")
            .arg(preloader_path)
            .spawn()?;

        return Ok(());

        fn resolve_path<'a>(path: &'a Path, name: &'static str) -> Result<&'a str> {
            let str = path.to_str();
            if !path.try_exists()? || str.is_none() {
                bail!("{} path could not be resolved", name);
            }
            Ok(str.unwrap())
        }
    }
}

#[typeshare]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Dependant {
    pub name: String,
    pub uuid: Uuid,
}

#[typeshare]
#[derive(Serialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "content")]
pub enum RemoveModResponse {
    Removed,
    HasDependants(Vec<Dependant>),
}

impl Profile {
    fn remove_mod(
        &mut self,
        package_uuid: &Uuid,
        thunderstore: &Thunderstore,
    ) -> Result<RemoveModResponse> {
        let dependants = self.dependants(package_uuid, thunderstore)?;

        if !dependants.is_empty() {
            let response = dependants
                .iter()
                .map(|m| Dependant {
                    name: m.package.name.clone(),
                    uuid: m.package.uuid4,
                })
                .collect();

            return Ok(RemoveModResponse::HasDependants(response));
        }

        self.force_remove_mod(package_uuid, thunderstore)?;

        Ok(RemoveModResponse::Removed)
    }

    fn force_remove_mod(&mut self, package_uuid: &Uuid, thunderstore: &Thunderstore) -> Result<()> {
        let index = self
            .mods
            .iter()
            .position(|m| m.package_uuid == *package_uuid)
            .context("mod not found in profile")?;

        let package = thunderstore.get_package(package_uuid)?;

        let mut path = self.path.join("BepInEx");
        for dir in ["core", "patchers", "plugins"].iter() {
            path.push(dir);
            path.push(&package.full_name);

            if path.try_exists().unwrap_or(false) {
                fs::remove_dir_all(&path).with_context(|| {
                    format!("failed to remove mod directory at {}", path.display())
                })?;
            }

            path.pop();
            path.pop();
        }

        self.mods.remove(index);

        Ok(())
    }
}

impl ModManager {
    pub fn create(prefs: &Prefs) -> Result<Self> {
        let save_path = prefs.data_path.join("manager.json");
        let save_data = match save_path.try_exists()? {
            true => {
                let json = fs::read_to_string(&save_path)
                    .fs_context("read manager save data", &save_path)?;

                serde_json::from_str(&json).context("failed to parse manager save data")?
            }
            false => ManagerSaveData {
                active_profile_index: 0,
            },
        };

        let profiles_path = prefs.data_path.join("profiles");
        fs::create_dir_all(&profiles_path)
            .fs_context("create profiles directory", &profiles_path)?;

        let mut profiles = Vec::new();
        for entry in profiles_path.read_dir()? {
            let path = entry?.path();
            if path.is_dir() {
                let profile = load_profile(path.clone())
                    .with_context(|| format!("failed to load profile at {}", path.display()))?;

                profiles.push(profile);
            }
        }

        let is_empty = profiles.is_empty();

        let mut manager = Self {
            profiles,
            active_profile_index: save_data.active_profile_index,
        };

        if is_empty {
            manager.create_profile("Default".to_owned(), prefs)?;
            manager.save(prefs)?;
        }

        Ok(manager)
    }

    fn save(&self, prefs: &Prefs) -> Result<()> {
        let manager_save_data = ManagerSaveData {
            active_profile_index: self.active_profile_index,
        };

        let json = serde_json::to_string_pretty(&manager_save_data)?;
        let save_path = prefs.data_path.join("manager.json");
        fs::write(save_path, json)?;

        let mut path = prefs.data_path.join("profiles");
        for profile in &self.profiles {
            path.push(&profile.name);
            path.push("manifest.json");

            let manifest = profile.manifest();
            let content = serde_json::to_string_pretty(&manifest)?;
            fs::write(&path, content)?;

            path.pop();
            path.pop();
        }

        Ok(())
    }

    fn create_profile<'a>(&'a mut self, name: String, prefs: &Prefs) -> Result<&'a Profile> {
        ensure!(
            !self.profiles.iter().any(|p| p.name == name),
            "profile with name '{}' already exists",
            name
        );

        let mut path = prefs.data_path.join("profiles");
        path.push(&name);
        fs::create_dir_all(&path)?;

        let profile = Profile {
            name,
            path,
            mods: Vec::new(),
            config: Vec::new(),
        };
        self.profiles.push(profile);

        let index = self.profiles.len() - 1;
        self.active_profile_index = index;
        Ok(&self.profiles[index])
    }

    fn delete_profile(&mut self, index: usize) -> Result<()> {
        ensure!(self.profiles.len() > 1, "cannot delete last profile");

        let profile = self.profiles.get(index)
            .with_context(|| format!("profile index {} is out of bounds", index))?;

        fs::remove_dir_all(&profile.path)?;
        self.profiles.remove(index); 

        if self.active_profile_index == index {
            self.active_profile_index = 0;
        }

        Ok(())
    }

    fn active_profile(&self) -> &Profile {
        &self.profiles[self.active_profile_index]
    }

    fn active_profile_mut(&mut self) -> &mut Profile {
        &mut self.profiles[self.active_profile_index]
    }

    fn set_active_profile(&mut self, index: usize) -> Result<()> {
        ensure!(
            index < self.profiles.len(),
            "profile index {} is out of bounds",
            index
        );

        self.active_profile_index = index;

        Ok(())
    }
}

fn load_profile(mut path: PathBuf) -> Result<Profile> {
    path.push("manifest.json");

    let manifest = fs::read_to_string(&path)
        .fs_context("read profile manifest", &path)?;

    path.pop();

    let manifest: ProfileManifest =
        serde_json::from_str(&manifest).context("failed to parse profile manifest")?;

    let config = config::load_config(path.clone()).collect();

    Ok(Profile {
        name: manifest.name,
        path,
        mods: manifest.mods,
        config,
    })
}
