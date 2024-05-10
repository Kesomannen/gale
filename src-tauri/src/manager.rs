use std::{
    collections::HashMap, fs, path::{Path, PathBuf}, sync::Mutex
};

use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use uuid::Uuid;
use walkdir::WalkDir;

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
pub mod launcher;

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
    games: HashMap<&'static String, ManagerGame>,
    pub active_game: &'static Game,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManagerSaveData {
    active_game: String
}

pub struct ManagerGame {
    profiles: Vec<Profile>,
    path: PathBuf,
    favorite: bool,
    active_profile_index: usize,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManagerGameSaveData {
    favorite: bool,
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
    Local {
        #[serde(default="default_true")]
        enabled: bool,
        #[serde(flatten)]
        data: Box<LocalMod>,
    },
    Remote {
        #[serde(default="default_true")]
        enabled: bool,
        #[serde(flatten)]
        mod_ref: ModRef,
    },
}

fn default_true() -> bool {
    true
}

impl ProfileMod {
    fn local(data: LocalMod) -> Self {
        ProfileMod::Local { enabled: true, data: Box::new(data) }
    }

    fn remote(mod_ref: ModRef) -> Self {
        ProfileMod::Remote { enabled: true, mod_ref }
    }

    fn enabled(&self) -> bool {
        match self {
            ProfileMod::Local { enabled, .. } | ProfileMod::Remote { enabled, .. } => *enabled,
        }
    }

    fn enabled_mut(&mut self) -> &mut bool {
        match self {
            ProfileMod::Local { enabled, .. } | ProfileMod::Remote { enabled, .. } => enabled,
        }
    }

    fn uuid(&self) -> &Uuid {
        match self {
            ProfileMod::Local { data, .. } => &data.uuid,
            ProfileMod::Remote { mod_ref, ..} => &mod_ref.package_uuid,
        }
    }

    fn as_remote(&self) -> Option<(&ModRef, bool)> {
        match self {
            ProfileMod::Remote { mod_ref, enabled } => Some((mod_ref, *enabled)),
            _ => None,
        }
    }

    fn as_local(&self) -> Option<(&LocalMod, bool)> {
        match self {
            ProfileMod::Local { data, enabled } => Some((data, *enabled)),
            _ => None,
        }
    }

    fn full_name<'a>(&'a self, thunderstore: &'a Thunderstore) -> Result<&'a str> {
        match self {
            ProfileMod::Local { data, .. } => Ok(&data.name),
            ProfileMod::Remote { mod_ref, .. } => {
                let package = thunderstore.get_package(&mod_ref.package_uuid)?;
                Ok(&package.full_name)
            }
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
    fn get_mod<'a>(&'a self, uuid: &Uuid) -> Result<&'a ProfileMod> {
        self.mods.iter().find(|p| p.uuid() == uuid).context("mod not found in profile")
    }

    fn get_mod_mut<'a>(&'a mut self, uuid: &Uuid) -> Result<&'a mut ProfileMod> {
        self.mods.iter_mut().find(|p| p.uuid() == uuid).context("mod not found in profile")
    }

    fn has_mod(&self, uuid: &Uuid) -> bool {
        self.get_mod(uuid).is_ok()
    }

    fn remote_mods(&self) -> impl Iterator<Item = (&'_ ModRef, bool)> {
        self.mods.iter().filter_map(ProfileMod::as_remote)
    }

    fn local_mods(&self) -> impl Iterator<Item = (&'_ LocalMod, bool)> {
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
                ProfileMod::Local { data, .. } => Ok(Queryable::Local(data)),
                ProfileMod::Remote { mod_ref, .. } => mod_ref.borrow(thunderstore).map(Queryable::Online),
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
            .filter(|(other, _)| other.package_uuid != target_mod.package.uuid4)
            .map(|(other, _)| other.borrow(thunderstore))
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
        let profile_mod = self.get_mod(uuid)?;

        if let Some((mod_ref, _)) = profile_mod.as_remote() {
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

        self.scan_mod(&self.mods[index], thunderstore, |dir| {
            fs::remove_dir_all(dir).fs_context("removing mod directory", dir)
        })?;

        self.mods.remove(index);

        Ok(())
    }

    fn toggle_mod(&mut self, uuid: &Uuid, thunderstore: &Thunderstore) -> Result<()> {
        let profile_mod = self.get_mod(uuid)?;
        let state = profile_mod.enabled();
        let new_state = !state;

        self.scan_mod(profile_mod, thunderstore, |dir| {
            let files = WalkDir::new(dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file());
            
            for file in files {
                let path = file.path();
                if new_state {
                    // remove ".old" extension
                    if let Some(ext) = path.extension() {
                        if ext == "old" {
                            fs::rename(path, path.with_extension(""))?;
                        }
                    }
                } else {
                    let mut new = path.to_path_buf();
                    fs_util::add_extension(&mut new, "old");
                    fs::rename(path, &new)?;
                }
            }

            Ok(())
        })?;

        *self.get_mod_mut(uuid).unwrap().enabled_mut() = new_state;

        Ok(())
    }

    fn scan_mod<'a, F>(&'a self, profile_mod: &'a ProfileMod, thunderstore: &'a Thunderstore, scan_dir: F) -> Result<()>
    where F: Fn(&Path) -> Result<()> 
    {
        let name = profile_mod.full_name(thunderstore)?;
        let mut path = self.path.join("BepInEx");

        for dir in ["core", "patchers", "plugins"].into_iter() {
            path.push(dir);
            path.push(name);

            if path.exists() {
                scan_dir(&path)?;
            }

            path.pop();
            path.pop();
        }

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

    fn load(mut path: PathBuf) -> Result<Option<(&'static Game, Self)>> {
        let file_name = fs_util::file_name(&path);
        let game = match games::from_name(&file_name) {
            Some(game) => game,
            None => return Ok(None),
        };
        
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

        Ok(Some((game, Self {
            profiles,
            path,
            favorite: data.favorite,
            active_profile_index: data.active_profile_index,
        })))
    }
}

impl ModManager {
    pub fn create(prefs: &Prefs) -> Result<Self> {
        let save_path = prefs.get_path_or_err("data_dir")?.join("manager.json");
        let save_data = match save_path.try_exists()? {
            true => {
                let json = fs::read_to_string(&save_path)
                    .fs_context("read manager save data", &save_path)?;

                serde_json::from_str(&json).context("failed to parse manager save data")?
            }
            false => ManagerSaveData {
                active_game: games::from_name("lethal-company").unwrap().id.to_owned(),
            },
        };

        let mut games = HashMap::new();

        for entry in prefs.get_path_or_err("data_dir")?.read_dir()? {
            let path = entry?.path();

            if path.is_dir() {
                if let Some((game, game_data)) = ManagerGame::load(path)? {
                    games.insert(&game.id, game_data);
                }
            }
        }

        let active_game = games::from_name(&save_data.active_game)
            .unwrap_or_else(|| games::from_name("lethal-company").unwrap());

        let mut manager = Self { games, active_game };

        manager.ensure_game(manager.active_game, prefs)?;
        manager.save(prefs)?;

        Ok(manager)
    }

    fn active_game(&self) -> &ManagerGame {
        self.games
            .get(&self.active_game.id)
            .expect("active game not found")
    }

    fn active_game_mut(&mut self) -> &mut ManagerGame {
        self.games
            .get_mut(&self.active_game.id)
            .expect("active game not found")
    }

    fn active_profile(&self) -> &Profile {
        self.active_game().active_profile()
    }

    fn active_profile_mut(&mut self) -> &mut Profile {
        self.active_game_mut().active_profile_mut()
    }

    fn ensure_game(&mut self, game: &'static Game, prefs: &Prefs) -> Result<()> {
        if self.games.get(&game.id).is_some() {
            return Ok(());
        }

        let mut manager_game = ManagerGame {
            profiles: Vec::new(),
            path: prefs.get_path_or_err("data_dir")?.join(&game.id),
            favorite: false,
            active_profile_index: 0,
        };

        manager_game.create_profile("Default".to_owned())?;
        self.games.insert(&game.id, manager_game);

        Ok(())
    }

    fn save(&self, prefs: &Prefs) -> Result<()> {
        let mut path = prefs.get_path_or_err("data_dir")?.clone();

        let data = ManagerSaveData {
            active_game: self.active_game.id.to_owned(),
        };

        let json = serde_json::to_string_pretty(&data)?;

        path.push("manager.json");
        fs::write(&path, json)?;
        path.pop();

        for (game_id, manager_game) in &self.games {
            path.push(game_id);

            let data = ManagerGameSaveData {
                favorite: manager_game.favorite,
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
}
