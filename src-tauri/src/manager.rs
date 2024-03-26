use std::{fs, path::PathBuf, process::Command, sync::Mutex};

use anyhow::{anyhow, Context, Result};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    prefs::Prefs,
    thunderstore::{
        self,
        models::PackageListing,
        query::{self, QueryModsArgs},
        BorrowedMod, OwnedMod,
    },
};

pub mod commands;
pub mod downloader;
pub mod importer;

pub struct ModManager {
    profiles: Mutex<Vec<Profile>>,
    active_profile_index: Mutex<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ManagerSaveData {
    active_profile_index: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ProfileMod {
    package_uuid: Uuid,
    version_uuid: Uuid
}

impl ProfileMod {
    fn get<'a>(&self, packages: &'a IndexMap<Uuid, PackageListing>) -> Result<BorrowedMod<'a>> {
        let package = packages
            .get(&self.package_uuid)
            .with_context(|| format!("package with id {} not found", self.package_uuid))?;
        let version = package.get_version(&self.version_uuid).with_context(|| {
            format!(
                "version with id {} not found in package {}",
                self.version_uuid, &package.full_name
            )
        })?;

        Ok((package, version).into())
    }
}

#[derive(Debug, Clone)]
struct Profile {
    name: String,
    path: PathBuf,
    mods: Vec<ProfileMod>,
}

impl Profile {
    fn query_mods(
        &self,
        args: QueryModsArgs,
        packages: &IndexMap<Uuid, PackageListing>,
    ) -> Result<Vec<OwnedMod>> {
        let mods = self
            .mods
            .iter()
            .map(|p| p.get(&packages))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(query::query_mods(&args, mods.into_iter()))
    }

    fn get_mod<'a>(&'a self, package_uuid: Uuid) -> Option<&'a ProfileMod> {
        self.mods.iter().find(|p| p.package_uuid == package_uuid)
    }

    fn has_mod(&self, package_uuid: Uuid) -> bool {
        self.get_mod(package_uuid).is_some()
    }

    fn dependants_of<'a>(
        &'a self,
        package_uuid: Uuid,
        packages: &IndexMap<Uuid, PackageListing>,
    ) -> Result<Vec<&'a ProfileMod>> {
        let target_mod = self.get_mod(package_uuid).context("mod not found")?;

        self.mods
            .iter()
            .filter_map(|other| {
                if other.package_uuid == package_uuid {
                    return None;
                }

                match other.get(packages) {
                    Ok(borrowed_other) => {
                        match thunderstore::resolve_deps(
                            &borrowed_other.version.dependencies,
                            packages,
                        )
                        .any(|dep| match dep {
                            Ok(dep) => dep.package.uuid4 == target_mod.package_uuid,
                            Err(_) => false,
                        }) {
                            true => Some(Ok(other)),
                            false => None,
                        }
                    }
                    Err(e) => Some(Err(e)),
                }
            })
            .collect()
    }
}

impl Profile {
    const GAME_ID: u32 = 1966720;

    fn run_game(&self, config: &Prefs) -> Result<()> {
        let steam_path = config
            .steam_exe_path
            .as_ref()
            .context("steam exe path not set")?;

        let steam_path = resolve_path(&steam_path, "steam")?;

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

        fn resolve_path<'a>(path: &'a PathBuf, name: &'static str) -> Result<&'a str> {
            let str = path.to_str();
            if !path.try_exists()? || str.is_none() {
                return Err(anyhow!("{} path could not be resolved", name));
            }
            Ok(str.unwrap())
        }
    }
}

impl ModManager {
    pub fn init(options: &Prefs) -> Result<Self> {
        println!("initiating manager");

        let save_path = options.data_path.join("manager.json");
        let save_data = match save_path.try_exists()? {
            true => {
                let data = fs::read_to_string(save_path)?;
                serde_json::from_str(&data).context("failed to parse manager save data")
            }
            false => Ok(ManagerSaveData {
                active_profile_index: 0,
            }),
        }?;

        let profiles_path = options.data_path.join("profiles");
        fs::create_dir_all(&profiles_path)?;

        let mut profiles = Vec::new();
        for entry in profiles_path.read_dir()? {
            let path = entry?.path();
            if path.is_dir() {
                let name = path.file_name().unwrap().to_str().unwrap().to_string();
                let mods = fs::read_to_string(path.join("profile.json"))?;
                let mods: Vec<ProfileMod> =
                    serde_json::from_str(&mods).context("failed to parse profile mods")?;
                profiles.push(Profile { name, path, mods });
            }
        }

        println!(
            "loaded profiles: {:?}",
            profiles.iter().map(|p| &p.name).collect::<Vec<_>>()
        );

        let is_empty = profiles.is_empty();

        let manager = Self {
            profiles: Mutex::new(profiles),
            active_profile_index: Mutex::new(save_data.active_profile_index),
        };

        if is_empty {
            manager.create_profile("Default".to_string(), options)?;
            manager.save(options)?;
        }

        Ok(manager)
    }

    fn save(&self, prefs: &Prefs) -> Result<()> {
        let manager_save_data = ManagerSaveData {
            active_profile_index: *self.active_profile_index.lock().unwrap(),
        };

        let json = serde_json::to_string(&manager_save_data)?;
        let save_path = prefs.data_path.join("manager.json");
        fs::write(save_path, json)?;

        let profiles = self.profiles.lock().unwrap();
        for profile in profiles.iter() {
            let json = serde_json::to_string(&profile.mods)?;
            let mut path = prefs.data_path.join("profiles");
            path.push(&profile.name);
            path.push("profile.json");

            fs::write(path, json)?;
        }

        Ok(())
    }

    fn create_profile(&self, name: String, options: &Prefs) -> Result<usize> {
        let mut profiles = self.profiles.lock().unwrap();
        if profiles.iter().any(|p| p.name == name) {
            return Err(anyhow!("profile with name {} already exists", name));
        }

        let mut path = options.data_path.join("profiles");
        path.push(&name);
        fs::create_dir_all(&path)?;

        let profile = Profile {
            name,
            path,
            mods: Vec::new(),
        };
        profiles.push(profile);

        Ok(profiles.len() - 1)
    }

    fn delete_profile(&self, index: usize) -> Result<()> {
        let mut profiles = self.profiles.lock().unwrap();
        let profile = profiles.get(index).context("profile not found")?;

        if profiles.len() == 1 {
            return Err(anyhow!("cannot delete last profile"));
        }

        let mut active_profile_index = self.active_profile_index.lock().unwrap();
        if *active_profile_index == index {
            *active_profile_index = 0;
        }

        fs::remove_dir_all(&profile.path)?;
        profiles.remove(index);

        Ok(())
    }

    fn set_active_profile(&self, index: usize) -> Result<()> {
        let mut active_profile = self.active_profile_index.lock().unwrap();
        let profiles = self.profiles.lock().unwrap();

        if index >= profiles.len() {
            return Err(anyhow!("profile index {} out of bounds", index));
        }

        *active_profile = index;

        Ok(())
    }
}

fn get_active_profile<'a>(
    profiles: &'a mut Vec<Profile>,
    manager: &ModManager,
) -> Result<&'a mut Profile> {
    let active_profile_index = *manager.active_profile_index.lock().unwrap();
    profiles
        .get_mut(active_profile_index)
        .context("active profile not found")
}
