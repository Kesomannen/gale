use std::{
    collections::HashMap, fs, path::{Path, PathBuf}, process::Command, str::FromStr, sync::Mutex
};

use anyhow::{anyhow, bail, ensure, Context, Result};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use uuid::Uuid;

use crate::{
    fs_util, games::{self, Game}, prefs::Prefs, thunderstore::{
        models::FrontendMod,
        query::{self, QueryModsArgs, Queryable},
        BorrowedMod, ModRef, Thunderstore,
    }, util::IoResultExt
};
use tauri::{AppHandle, Manager};

pub mod commands;
pub mod config;
pub mod downloader;
pub mod importer;

pub fn setup(app: &AppHandle) -> Result<()> {
    {
        let prefs = app.state::<Mutex<Prefs>>();
        let prefs = prefs.lock().unwrap();

        let manager = ModManager::create(&prefs)?;
        app.manage(Mutex::new(manager));
    }

    importer::setup(app).context("failed to initialize importer")?;
    downloader::setup(app).context("failed to initialize downloader")?;
    config::setup(app).context("failed to initialize mod config")?;

    Ok(())
}

pub struct ModManager {
    games: HashMap<u32, ManagerGame>,
    pub active_game: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManagerSaveData {
    active_game: u32
}

pub struct ManagerGame {
    profiles: Vec<Profile>,
    path: PathBuf,
    active_profile_index: usize,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManagerGameSaveData {
    active_profile_index: usize,
}

#[typeshare]
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct LocalMod {
    pub name: String,
    pub icon: Option<PathBuf>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub version: Option<semver::Version>,
    pub dependencies: Option<Vec<String>>,
    pub uuid: Uuid,
}

#[typeshare]
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", untagged)]
enum ProfileMod {
    Local(LocalMod),
    Remote(ModRef),
}

impl ProfileMod {
    fn uuid(&self) -> &Uuid {
        match self {
            ProfileMod::Local(local) => &local.uuid,
            ProfileMod::Remote(mod_ref) => &mod_ref.package_uuid,
        }
    }

    fn as_remote(&self) -> Option<&ModRef> {
        match self {
            ProfileMod::Remote(mod_ref) => Some(mod_ref),
            _ => None,
        }
    }

    fn as_local(&self) -> Option<&LocalMod> {
        match self {
            ProfileMod::Local(local) => Some(local),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileManifest {
    name: String,
    mods: Vec<ProfileMod>,
}

struct Profile {
    name: String,
    path: PathBuf,
    mods: Vec<ProfileMod>,
    config: Vec<config::LoadedFile>,
}

impl Profile {
    fn get_mod<'a>(&'a self, uuid: &Uuid) -> Option<&'a ProfileMod> {
        self.mods.iter().find(|p| p.uuid() == uuid)
    }

    fn has_mod(&self, uuid: &Uuid) -> bool {
        self.get_mod(uuid).is_some()
    }

    fn remote_mods(&self) -> impl Iterator<Item = &'_ ModRef> {
        self.mods.iter().filter_map(ProfileMod::as_remote)
    }

    fn local_mods(&self) -> impl Iterator<Item = &'_ LocalMod> {
        self.mods.iter().filter_map(ProfileMod::as_local)
    }

    fn manifest(&self) -> ProfileManifest {
        ProfileManifest {
            name: self.name.clone(),
            mods: self.mods.clone(),
        }
    }

    fn query_mods(
        &self,
        args: &QueryModsArgs,
        thunderstore: &Thunderstore,
    ) -> Result<Vec<FrontendMod>> {
        let queryables = self
            .mods
            .iter()
            .map(|p| match p {
                ProfileMod::Local(local) => Ok(Queryable::Local(local)),
                ProfileMod::Remote(mod_ref) => mod_ref.borrow(thunderstore).map(Queryable::Online),
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(query::query_mods(args, queryables.into_iter()))
    }

    fn dependants<'a>(
        &self,
        target_mod: BorrowedMod<'a>,
        thunderstore: &'a Thunderstore,
    ) -> Result<Vec<BorrowedMod<'a>>> {
        self.remote_mods()
            .filter(|other| other.package_uuid != target_mod.package.uuid4)
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
                            .any(|dep| dep.package.uuid4 == target_mod.package.uuid4)
                        {
                            true => Some(Ok(other)),
                            false => None,
                        },
                        Err(e) => Some(Err(e)),
                    }
                }
                Err(_) => Some(other),
            }) // filter out packages that do not depend on the target one, while keeping errors
            .collect()
    }

    fn load(mut path: PathBuf) -> Result<Self> {
        path.push("profile.json");
    
        let manifest = fs::read_to_string(&path).fs_context("read profile manifest", &path)?;
    
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
        uuid: &Uuid,
        thunderstore: &Thunderstore,
    ) -> Result<RemoveModResponse> {
        let profile_mod = self.get_mod(uuid).context("mod not found in profile")?;

        if let ProfileMod::Remote(mod_ref) = profile_mod {
            let borrowed_mod = mod_ref.borrow(thunderstore)?;
            let dependants = self.dependants(borrowed_mod, thunderstore)?;

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
        };

        self.force_remove_mod(uuid, thunderstore)?;

        Ok(RemoveModResponse::Removed)
    }

    fn force_remove_mod(&mut self, uuid: &Uuid, thunderstore: &Thunderstore) -> Result<()> {
        let index = self
            .mods
            .iter()
            .position(|m| m.uuid() == uuid)
            .context("mod not found in profile")?;

        let name = match &self.mods[index] {
            ProfileMod::Local(local) => &local.name,
            ProfileMod::Remote(mod_ref) => {
                let package = thunderstore.get_package(&mod_ref.package_uuid)?;
                &package.full_name
            }
        };

        let mut path = self.path.join("BepInEx");
        for dir in ["core", "patchers", "plugins"].iter() {
            path.push(dir);
            path.push(name);

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

impl ManagerGame {
    fn create_profile(&mut self, name: String) -> Result<&Profile> {
        ensure!(
            !self.profiles.iter().any(|p| p.name == name),
            "profile with name '{}' already exists",
            name
        );

        let mut path = self.path.join("profiles");
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

        let profile = self
            .profiles
            .get(index)
            .with_context(|| format!("profile index {} is out of bounds", index))?;

        fs::remove_dir_all(&profile.path)?;
        self.profiles.remove(index);

        self.active_profile_index = 0;

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

    fn load(mut path: PathBuf) -> Result<(u32, Self)> {
        let file_name = fs_util::file_name(&path);
        let game = games::from_name(&file_name)
            .ok_or_else(|| anyhow!("invalid game directory name: {}", file_name))?;
        
        path.push("game.json");

        let json = fs::read_to_string(&path).fs_context("reading game save data", &path)?;
        let data: ManagerGameSaveData = serde_json::from_str(&json)
            .context("failed to parse game save data")?;

        path.pop();

        let mut profiles = Vec::new();
        path.push("profiles");

        for entry in path.read_dir()? {
            let path = entry?.path();

            if path.is_dir() {
                let profile = Profile::load(path.clone())?;
                profiles.push(profile);
            }
        }

        path.pop();

        Ok((game.steam_id, Self {
            profiles,
            path,
            active_profile_index: data.active_profile_index,
        }))
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
                active_game: games::from_name("LethalCompany").unwrap().steam_id,
            },
        };

        let mut games = HashMap::new();

        for entry in prefs.data_path.read_dir()? {
            let path = entry?.path();

            if path.is_dir() {
                let (game, game_data) = ManagerGame::load(path)?;
                games.insert(game, game_data);
            }
        }

        let mut manager = Self {
            games,
            active_game: save_data.active_game,
        };

        manager.ensure_game(manager.active_game, prefs)?;
        manager.save(prefs)?;

        Ok(manager)
    }

    fn active_game(&self) -> &ManagerGame {
        self.games
            .get(&self.active_game)
            .expect("active game not found")
    }

    fn active_game_mut(&mut self) -> &mut ManagerGame {
        self.games
            .get_mut(&self.active_game)
            .expect("active game not found")
    }

    fn active_profile(&self) -> &Profile {
        self.active_game().active_profile()
    }

    fn active_profile_mut(&mut self) -> &mut Profile {
        self.active_game_mut().active_profile_mut()
    }

    fn ensure_game(&mut self, steam_id: u32, prefs: &Prefs) -> Result<()> {
        if self.games.get(&steam_id).is_some() {
            return Ok(());
        }

        let game = games::from_steam_id(steam_id)
            .ok_or_else(|| anyhow!("invalid game steam id: {}", steam_id))?;

        let mut new_game = ManagerGame {
            profiles: Vec::new(),
            path: prefs.data_path.join(game.name),
            active_profile_index: 0,
        };

        new_game.create_profile("Default".to_owned())?;
        self.games.insert(steam_id, new_game);

        Ok(())
    }

    fn save(&self, prefs: &Prefs) -> Result<()> {
        let mut path = prefs.data_path.clone();

        let data = ManagerSaveData {
            active_game: self.active_game,
        };

        let json = serde_json::to_string_pretty(&data)?;

        path.push("manager.json");
        fs::write(&path, json)?;
        path.pop();

        for (game_id, manager_game) in &self.games {
            let game = games::from_steam_id(*game_id)
                .ok_or_else(|| anyhow!("invalid game steam id: {}", game_id))?;

            path.push(game.name);

            let data = ManagerGameSaveData {
                active_profile_index: manager_game.active_profile_index,
            };

            let json = serde_json::to_string_pretty(&data)?;

            path.push("game.json");
            fs::write(&path, json)?;
            path.pop();

            path.push("profiles");
            for profile in &manager_game.profiles {
                path.push(&profile.name);
                path.push("profile.json");

                let content = serde_json::to_string_pretty(&profile.manifest())?;
                fs::write(&path, content)?;

                path.pop();
                path.pop();
            }
            path.pop();

            path.pop();
        }

        Ok(())
    }

    fn run_game(&self, prefs: &Prefs) -> Result<()> {
        let steam_path = prefs
            .steam_exe_path
            .as_ref()
            .context("steam exe path not set")?;

        let steam_path = resolve_path(steam_path, "steam")?;

        let mut preloader_path = self.active_profile().path.join("BepInEx");
        preloader_path.push("core");
        preloader_path.push("BepInEx.Preloader.dll");

        let preloader_path = resolve_path(&preloader_path, "preloader")?;

        Command::new(steam_path)
            .arg("-applaunch")
            .arg(self.active_game.to_string())
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
