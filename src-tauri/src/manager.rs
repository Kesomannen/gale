use std::{
    cmp::Ordering,
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

use anyhow::{ensure, Context, Result};
use chrono::{DateTime, Utc};
use exporter::modpack::ModpackArgs;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use typeshare::typeshare;
use uuid::Uuid;
use walkdir::WalkDir;

use crate::{
    config,
    games::{self, Game},
    prefs::Prefs,
    thunderstore::{
        self,
        models::FrontendProfileMod,
        query::{self, QueryModsArgs, Queryable, SortBy},
        BorrowedMod, ModRef, Thunderstore,
    },
    util::{
        self,
        error::IoResultExt,
        fs::{JsonStyle, Overwrite},
    },
};

pub mod commands;
pub mod downloader;
pub mod exporter;
pub mod importer;
pub mod installer;
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

    Ok(())
}

pub struct ModManager {
    pub games: HashMap<&'static String, ManagerGame>,
    pub active_game: &'static Game,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManagerSaveData {
    active_game: String,
}

pub struct ManagerGame {
    pub game: &'static Game,
    pub profiles: Vec<Profile>,
    pub path: PathBuf,
    pub favorite: bool,
    pub active_profile_index: usize,
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
#[serde(rename_all = "camelCase")]
pub struct ProfileMod {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "Utc::now")]
    pub install_time: DateTime<Utc>,
    #[serde(flatten)]
    pub kind: ProfileModKind,
}

impl ProfileMod {
    fn new(install_time: DateTime<Utc>, kind: ProfileModKind) -> Self {
        Self {
            kind,
            install_time,
            enabled: true,
        }
    }

    fn now(kind: ProfileModKind) -> Self {
        Self::new(Utc::now(), kind)
    }

    fn local_now(data: LocalMod) -> Self {
        Self::now(ProfileModKind::Local(Box::new(data)))
    }

    fn remote_now(mod_ref: ModRef) -> Self {
        Self::now(ProfileModKind::Remote(mod_ref))
    }

    pub fn uuid(&self) -> &Uuid {
        self.kind.uuid()
    }

    fn as_remote(&self) -> Option<(&ModRef, bool)> {
        self.kind.as_remote().map(|remote| (remote, self.enabled))
    }

    fn as_local(&self) -> Option<(&LocalMod, bool)> {
        self.kind.as_local().map(|local| (local, self.enabled))
    }

    fn queryable<'a>(
        &'a self,
        index: usize,
        thunderstore: &'a Thunderstore,
    ) -> Result<QueryableProfileMod<'a>> {
        let kind = match &self.kind {
            ProfileModKind::Local(local) => QueryableProfileModKind::Local(local),
            ProfileModKind::Remote(mod_ref) => {
                let borrow = mod_ref.borrow(thunderstore)?;
                QueryableProfileModKind::Remote(borrow)
            }
        };

        Ok(QueryableProfileMod {
            kind,
            index,
            install_time: self.install_time,
            enabled: self.enabled,
        })
    }
}

#[typeshare]
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", untagged)]
pub enum ProfileModKind {
    Local(Box<LocalMod>),
    Remote(ModRef),
}

fn default_true() -> bool {
    true
}

impl ProfileModKind {
    pub fn uuid(&self) -> &Uuid {
        match self {
            ProfileModKind::Local(local_mod) => &local_mod.uuid,
            ProfileModKind::Remote(mod_ref) => &mod_ref.package_uuid,
        }
    }

    pub fn as_remote(&self) -> Option<&ModRef> {
        match self {
            ProfileModKind::Remote(mod_ref) => Some(mod_ref),
            _ => None,
        }
    }

    pub fn as_local(&self) -> Option<&LocalMod> {
        match self {
            ProfileModKind::Local(local) => Some(local),
            _ => None,
        }
    }

    pub fn full_name<'a>(&'a self, thunderstore: &'a Thunderstore) -> Result<&'a str> {
        match self {
            ProfileModKind::Local(local) => Ok(&local.name),
            ProfileModKind::Remote(mod_ref) => {
                let package = thunderstore.get_package(&mod_ref.package_uuid)?;
                Ok(&package.full_name)
            }
        }
    }

    pub fn name<'a>(&'a self, thunderstore: &'a Thunderstore) -> Result<&'a str> {
        match self {
            ProfileModKind::Local(local) => Ok(&local.name),
            ProfileModKind::Remote(mod_ref) => {
                let package = thunderstore.get_package(&mod_ref.package_uuid)?;
                Ok(&package.name)
            }
        }
    }
}

struct QueryableProfileMod<'a> {
    enabled: bool,
    index: usize,
    install_time: DateTime<Utc>,
    kind: QueryableProfileModKind<'a>,
}

impl<'a> Queryable for QueryableProfileMod<'a> {
    fn full_name(&self) -> &str {
        use QueryableProfileModKind as Kind;

        match &self.kind {
            Kind::Local(local) => &local.name,
            Kind::Remote(remote) => &remote.package.full_name,
        }
    }

    fn matches(&self, args: &QueryModsArgs) -> bool {
        use QueryableProfileModKind as Kind;

        if !args.include_disabled && !self.enabled {
            return false;
        }

        match &self.kind {
            Kind::Local(local) => local.matches(args),
            Kind::Remote(remote) => remote.matches(args),
        }
    }

    fn cmp(&self, other: &Self, args: &QueryModsArgs) -> Ordering {
        use QueryableProfileModKind as Kind;

        match args.sort_by {
            SortBy::InstallDate => return self.install_time.cmp(&other.install_time),
            SortBy::Custom => return self.index.cmp(&other.index),
            _ => (),
        };

        match (&self.kind, &other.kind) {
            (Kind::Remote(a), Kind::Remote(b)) => a.cmp(b, args),
            (Kind::Local(a), Kind::Local(b)) => a.cmp(b, args),
            (Kind::Local(_), _) => Ordering::Greater,
            (_, Kind::Local(_)) => Ordering::Less,
        }
    }
}

enum QueryableProfileModKind<'a> {
    Local(&'a LocalMod),
    Remote(BorrowedMod<'a>),
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileManifest {
    mods: Vec<ProfileMod>,
    #[serde(default)]
    modpack: Option<ModpackArgs>,
}

pub struct Profile {
    pub name: String,
    pub path: PathBuf,
    pub mods: Vec<ProfileMod>,
    pub config: Vec<config::LoadFileResult>,
    pub linked_config: HashMap<Uuid, String>,
    pub modpack: Option<ModpackArgs>,
}

impl Profile {
    fn is_valid_name(name: &str) -> bool {
        name.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ' ')
    }

    fn index_of(&self, uuid: &Uuid) -> Result<usize> {
        self.mods
            .iter()
            .position(|p| p.uuid() == uuid)
            .context("mod not found in profile")
    }

    fn get_mod<'a>(&'a self, uuid: &Uuid) -> Result<&'a ProfileMod> {
        self.mods
            .iter()
            .find(|p| p.uuid() == uuid)
            .context("mod not found in profile")
    }

    fn get_mod_mut<'a>(&'a mut self, uuid: &Uuid) -> Result<&'a mut ProfileMod> {
        self.mods
            .iter_mut()
            .find(|p| p.uuid() == uuid)
            .context("mod not found in profile")
    }

    fn has_mod(&self, uuid: &Uuid) -> bool {
        self.get_mod(uuid).is_ok()
    }

    fn remote_mods(&self) -> impl Iterator<Item = (&ModRef, bool)> {
        self.mods.iter().filter_map(ProfileMod::as_remote)
    }

    fn local_mods(&self) -> impl Iterator<Item = (&LocalMod, bool)> {
        self.mods.iter().filter_map(ProfileMod::as_local)
    }

    fn manifest(&self) -> ProfileManifest {
        ProfileManifest {
            modpack: self.modpack.clone(),
            mods: self.mods.clone(),
        }
    }

    fn query_mods(
        &self,
        args: &QueryModsArgs,
        thunderstore: &Thunderstore,
    ) -> Result<Vec<FrontendProfileMod>> {
        let queryables = self
            .mods
            .iter()
            .enumerate()
            .map(|(i, p)| p.queryable(i, thunderstore))
            .collect::<Result<Vec<_>>>()?;

        Ok(query::query_mods(args, queryables.into_iter())
            .map(|queryable| {
                let (data, uuid) = match queryable.kind {
                    QueryableProfileModKind::Local(local) => (local.clone().into(), local.uuid),
                    QueryableProfileModKind::Remote(remote) => {
                        (remote.into(), remote.package.uuid4)
                    }
                };

                FrontendProfileMod {
                    data,
                    enabled: queryable.enabled,
                    config_file: self.linked_config.get(&uuid).cloned(),
                }
            })
            .collect())
    }

    fn dependants<'a>(
        &'a self,
        target_mod: BorrowedMod<'a>,
        thunderstore: &'a Thunderstore,
    ) -> impl Iterator<Item = Result<BorrowedMod<'a>>> + 'a {
        self.remote_mods()
            .filter(|(other, _)| other.package_uuid != target_mod.package.uuid4)
            .map(|(other, _)| other.borrow(thunderstore))
            .filter_map_ok(move |other| {
                let deps = thunderstore.dependencies(other.version).0;
                match deps.iter().any(|dep| dep.package == target_mod.package) {
                    true => Some(other),
                    false => None,
                }
            })
    }

    fn load(mut path: PathBuf) -> Result<Self> {
        path.push("profile.json");

        let manifest: ProfileManifest =
            util::fs::read_json(&path).context("failed to read profile manifest")?;

        path.pop();

        let profile = Profile {
            modpack: manifest.modpack,
            name: util::fs::file_name_lossy(&path),
            mods: manifest.mods,
            linked_config: HashMap::new(),
            config: Vec::new(),
            path,
        };

        Ok(profile)
    }
}

#[typeshare]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Dependant {
    pub name: String,
    pub uuid: Uuid,
}

impl From<BorrowedMod<'_>> for Dependant {
    fn from(value: BorrowedMod) -> Self {
        Self {
            name: value.package.name.clone(),
            uuid: value.package.uuid4,
        }
    }
}

#[typeshare]
#[derive(Serialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "content")]
pub enum ModActionResponse {
    Done,
    HasDependants(Vec<Dependant>),
    HasDependencies(Vec<Dependant>),
}

impl Profile {
    fn rename(&mut self, name: String) -> Result<()> {
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

        self.name = name;
        self.path = new_path;

        Ok(())
    }

    fn remove_mod(
        &mut self,
        uuid: &Uuid,
        thunderstore: &Thunderstore,
    ) -> Result<ModActionResponse> {
        let profile_mod = self.get_mod(uuid)?;

        if profile_mod.enabled {
            if let Some((mod_ref, _)) = profile_mod.as_remote() {
                let borrowed = mod_ref.borrow(thunderstore)?;

                if let Some(dependants) = self.check_dependants(borrowed, thunderstore) {
                    return Ok(ModActionResponse::HasDependants(dependants));
                }
            }
        }

        self.force_remove_mod(uuid, thunderstore)?;
        Ok(ModActionResponse::Done)
    }

    fn force_remove_mod(&mut self, uuid: &Uuid, thunderstore: &Thunderstore) -> Result<()> {
        let index = self.index_of(uuid)?;

        self.scan_mod(&self.mods[index].kind, thunderstore, |dir| {
            fs::remove_dir_all(dir).fs_context("removing mod directory", dir)
        })?;

        self.mods.remove(index);

        Ok(())
    }

    fn toggle_mod(
        &mut self,
        uuid: &Uuid,
        thunderstore: &Thunderstore,
    ) -> Result<ModActionResponse> {
        let profile_mod = self.get_mod(uuid)?;

        if let Some((mod_ref, _)) = profile_mod.as_remote() {
            let borrowed = mod_ref.borrow(thunderstore)?;

            if profile_mod.enabled {
                if let Some(dependants) = self.check_dependants(borrowed, thunderstore) {
                    return Ok(ModActionResponse::HasDependants(dependants));
                }
            } else if let Some(deps) = self.check_deps(borrowed, thunderstore) {
                return Ok(ModActionResponse::HasDependencies(deps));
            }
        }

        self.force_toggle_mod(uuid, thunderstore)?;
        Ok(ModActionResponse::Done)
    }

    fn force_toggle_mod(&mut self, uuid: &Uuid, thunderstore: &Thunderstore) -> Result<()> {
        let profile_mod = self.get_mod(uuid)?;
        let state = profile_mod.enabled;
        let new_state = !state;

        self.scan_mod(&profile_mod.kind, thunderstore, |dir| {
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
                    util::fs::add_extension(&mut new, "old");
                    fs::rename(path, &new)?;
                }
            }

            Ok(())
        })?;

        self.get_mod_mut(uuid).unwrap().enabled = new_state;

        Ok(())
    }

    fn check_dependants(
        &self,
        borrowed_mod: BorrowedMod,
        thunderstore: &Thunderstore,
    ) -> Option<Vec<Dependant>> {
        let dependants = self
            .dependants(borrowed_mod, thunderstore)
            .filter_map(Result::ok) // ignore any missing deps
            .filter(|borrowed| {
                log::debug!("{}", borrowed.package.name);
                !borrowed.package.is_modpack()
                    && self.get_mod(&borrowed.package.uuid4).unwrap().enabled
            })
            .map_into()
            .collect::<Vec<_>>();

        match dependants.is_empty() {
            true => None,
            false => Some(dependants),
        }
    }

    fn check_deps(
        &self,
        borrowed_mod: BorrowedMod,
        thunderstore: &Thunderstore,
    ) -> Option<Vec<Dependant>> {
        let disabled_deps = thunderstore
            .dependencies(borrowed_mod.version)
            .0
            .into_iter()
            .filter(|dep| {
                self.get_mod(&dep.package.uuid4)
                    .is_ok_and(|profile_mod| !profile_mod.enabled)
            })
            .map_into()
            .collect::<Vec<_>>();

        match disabled_deps.is_empty() {
            true => None,
            false => Some(disabled_deps),
        }
    }

    fn scan_mod<'a, F>(
        &'a self,
        profile_mod: &'a ProfileModKind,
        thunderstore: &'a Thunderstore,
        scan_dir: F,
    ) -> Result<()>
    where
        F: Fn(&Path) -> Result<()>,
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

    fn reorder_mod(&mut self, uuid: &Uuid, delta: i32) -> Result<()> {
        let index = self
            .mods
            .iter()
            .position(|m| m.uuid() == uuid)
            .context("mod not found in profile")?;

        let target = (index as i32 + delta).clamp(0, self.mods.len() as i32 - 1) as usize;
        let profile_mod = self.mods.remove(index);
        self.mods.insert(target, profile_mod);

        Ok(())
    }
}

impl ManagerGame {
    fn create_profile(&mut self, name: String) -> Result<&Profile> {
        ensure!(
            Profile::is_valid_name(&name),
            "profile name '{}' is invalid",
            name
        );

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
            linked_config: HashMap::new(),
            modpack: None,
        };
        self.profiles.push(profile);

        let index = self.profiles.len() - 1;
        self.active_profile_index = index;
        Ok(&self.profiles[index])
    }

    fn delete_profile(&mut self, index: usize, allow_delete_last: bool) -> Result<()> {
        ensure!(
            allow_delete_last || self.profiles.len() > 1,
            "cannot delete last profile"
        );

        let profile = self.get_profile(index)?;

        fs::remove_dir_all(&profile.path)?;
        self.profiles.remove(index);

        self.active_profile_index = 0;

        Ok(())
    }

    fn duplicate_profile(&mut self, duplicate_name: String, index: usize) -> Result<()> {
        self.create_profile(duplicate_name)?;
        let profile = self.get_profile(index)?;
        let new_profile = self.active_profile();

        util::fs::copy_dir(&profile.path, &new_profile.path, Overwrite::Yes)?;

        let mods = profile.mods.clone();

        let new_profile = self.active_profile_mut();

        new_profile.mods = mods;

        Ok(())
    }

    fn get_profile(&self, index: usize) -> Result<&Profile> {
        self.profiles
            .get(index)
            .with_context(|| format!("profile index {} is out of bounds", index))
    }

    fn active_profile(&self) -> &Profile {
        &self.profiles[self.active_profile_index]
    }

    fn active_profile_mut(&mut self) -> &mut Profile {
        &mut self.profiles[self.active_profile_index]
    }

    fn set_active_profile(
        &mut self,
        index: usize,
        _thunderstore: Option<&Thunderstore>,
    ) -> Result<()> {
        ensure!(
            index < self.profiles.len(),
            "profile index {} is out of bounds",
            index
        );

        self.active_profile_index = index;
        //self.active_profile_mut().refresh_config(thunderstore);

        Ok(())
    }

    fn installed_mods<'a>(
        &'a self,
        thunderstore: &'a Thunderstore,
    ) -> impl Iterator<Item = Result<BorrowedMod<'a>>> + 'a {
        self.profiles.iter().flat_map(|profile| {
            profile
                .remote_mods()
                .map(|(mod_ref, _)| mod_ref.borrow(thunderstore))
        })
    }

    fn load(mut path: PathBuf) -> Result<Option<(&'static Game, Self)>> {
        let file_name = util::fs::file_name_lossy(&path);
        let game = match games::from_id(&file_name) {
            Some(game) => game,
            None => return Ok(None),
        };

        path.push("game.json");

        let data: ManagerGameSaveData =
            util::fs::read_json(&path).context("failed to read game save data")?;

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

        let active_profile_index = data
            .active_profile_index
            .min(profiles.len().saturating_sub(1));

        Ok(Some((
            game,
            Self {
                game,
                profiles,
                path,
                favorite: data.favorite,
                active_profile_index,
            },
        )))
    }
}

const DEFAULT_GAME_ID: &str = "lethal-company";

impl ModManager {
    pub fn create(prefs: &Prefs) -> Result<Self> {
        let save_path = prefs.data_dir.join("manager.json");
        let save_data = match save_path.try_exists()? {
            true => util::fs::read_json(&save_path).context("failed to read manager save data")?,
            false => ManagerSaveData {
                active_game: DEFAULT_GAME_ID.to_owned(),
            },
        };

        let mut games = HashMap::new();

        for entry in prefs.data_dir.read_dir()? {
            let path = entry?.path();

            if path.is_dir() {
                if let Some((game, game_data)) = ManagerGame::load(path)? {
                    games.insert(&game.id, game_data);
                }
            }
        }

        let active_game = games::from_id(&save_data.active_game)
            .unwrap_or_else(|| games::from_id(DEFAULT_GAME_ID).unwrap());

        let mut manager = Self { games, active_game };

        manager.ensure_game(manager.active_game, prefs)?;
        manager.save(prefs)?;

        Ok(manager)
    }

    pub fn active_game(&self) -> &ManagerGame {
        self.games
            .get(&self.active_game.id)
            .expect("active game not found")
    }

    pub fn active_game_mut(&mut self) -> &mut ManagerGame {
        self.games
            .get_mut(&self.active_game.id)
            .expect("active game not found")
    }

    pub fn active_profile(&self) -> &Profile {
        self.active_game().active_profile()
    }

    pub fn active_profile_mut(&mut self) -> &mut Profile {
        self.active_game_mut().active_profile_mut()
    }

    fn ensure_game(&mut self, game: &'static Game, prefs: &Prefs) -> Result<()> {
        if self.games.contains_key(&game.id) {
            return Ok(());
        }

        let mut manager_game = ManagerGame {
            game,
            profiles: Vec::new(),
            path: prefs.data_dir.join(&game.id),
            favorite: false,
            active_profile_index: 0,
        };

        manager_game.create_profile("Default".to_owned())?;
        self.games.insert(&game.id, manager_game);

        Ok(())
    }

    pub fn cache_mods(&self, thunderstore: &Thunderstore) -> Result<()> {
        let packages = self
            .active_game()
            .profiles
            .iter()
            .flat_map(|profile| {
                profile
                    .remote_mods()
                    .map(|(mod_ref, _)| mod_ref.borrow(thunderstore))
            })
            .filter_map(Result::ok)
            .map(|borrowed| borrowed.package)
            .unique()
            .collect_vec();

        let path = thunderstore::cache_path(self);
        thunderstore::write_cache(&packages, &path)
    }

    fn save(&self, prefs: &Prefs) -> Result<()> {
        let mut path = prefs.data_dir.get().to_path_buf();

        path.push("manager.json");

        util::fs::write_json(
            &path,
            &ManagerSaveData {
                active_game: self.active_game.id.to_owned(),
            },
            JsonStyle::Pretty,
        )?;

        path.pop();

        for (game_id, manager_game) in &self.games {
            path.push(game_id);
            path.push("game.json");

            util::fs::write_json(
                &path,
                &ManagerGameSaveData {
                    favorite: manager_game.favorite,
                    active_profile_index: manager_game.active_profile_index,
                },
                JsonStyle::Pretty,
            )?;

            path.pop();

            path.push("profiles");
            for profile in &manager_game.profiles {
                path.push(&profile.name);
                path.push("profile.json");

                util::fs::write_json(&path, &profile.manifest(), JsonStyle::Pretty)?;

                path.pop();
                path.pop();
            }
            path.pop();

            path.pop();
        }

        Ok(())
    }
}
