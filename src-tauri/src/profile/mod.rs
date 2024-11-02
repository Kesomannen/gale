use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

use anyhow::{bail, ensure, Context, Result};
use chrono::{DateTime, Utc};
use commands::save;
use export::modpack::ModpackArgs;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Listener, Manager};
use uuid::Uuid;
use walkdir::WalkDir;

use crate::{
    config,
    games::{self, Game},
    logger,
    prefs::Prefs,
    thunderstore::{self, BorrowedMod, ModId, Thunderstore, VersionIdent},
    util::{
        self,
        error::IoResultExt,
        fs::{JsonStyle, Overwrite, PathExt},
    },
};

pub mod commands;
pub mod export;
pub mod import;
pub mod install;
pub mod launch;
pub mod update;

mod query;

mod local;
pub use local::*;

pub fn setup(app: &AppHandle) -> Result<()> {
    {
        let prefs = app.state::<Mutex<Prefs>>();
        let prefs = prefs.lock().unwrap();

        let manager = ModManager::create(&prefs)?;
        app.manage(Mutex::new(manager));
    }

    import::setup(app).context("failed to initialize importer")?;
    install::setup(app).context("failed to initialize downloader")?;

    let handle = app.to_owned();
    app.listen("reorder_mod", move |event| {
        if let Err(err) = handle_reorder_event(event, &handle) {
            logger::log_js_err("Failed to reorder mod", &err, &handle);
        }
    });

    Ok(())
}

/// The main struct of the app.
pub struct ModManager {
    /// Holds all the currently managed games, indexed by their slug.
    ///
    /// Note that this only contains entries for `Game`s which the user has selected at least once.
    pub games: HashMap<&'static String, ManagerGame>,
    pub active_game: &'static Game,
}

/// Persistent data for ModManager
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManagerSaveData {
    active_game: String,
}

/// Stores information and profiles about one game.
pub struct ManagerGame {
    pub game: &'static Game,
    pub profiles: Vec<Profile>,
    pub path: PathBuf,
    pub favorite: bool,
    pub active_profile_index: usize,
}

/// Persistent data for ManagerGame
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManagerGameSaveData {
    favorite: bool,
    active_profile_index: usize,
}

pub struct Profile {
    pub name: String,
    pub path: PathBuf,
    pub mods: Vec<ProfileMod>,
    pub ignored_updates: HashSet<Uuid>,
    pub config: Vec<config::LoadFileResult>,
    pub linked_config: HashMap<Uuid, PathBuf>,
    pub modpack: Option<ModpackArgs>,
}

/// Persistent data for Profile
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileSaveData {
    mods: Vec<ProfileMod>,

    #[serde(default)]
    modpack: Option<ModpackArgs>,

    #[serde(default)]
    ignored_updates: HashSet<Uuid>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProfileMod {
    pub enabled: bool,

    #[serde(default = "Utc::now")]
    pub install_time: DateTime<Utc>,

    #[serde(flatten)]
    pub kind: ProfileModKind,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", untagged)]
pub enum ProfileModKind {
    Thunderstore(ThunderstoreMod),
    // Box to decrease size of enum, since this variant is rare and much larger
    Local(Box<LocalMod>),
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ThunderstoreMod {
    #[serde(alias = "fullName")]
    ident: VersionIdent,

    #[serde(flatten)]
    id: ModId,
}

impl ProfileMod {
    fn new(kind: ProfileModKind) -> Self {
        Self {
            kind,
            install_time: Utc::now(),
            enabled: true,
        }
    }

    fn new_at(install_time: DateTime<Utc>, kind: ProfileModKind) -> Self {
        Self {
            install_time,
            ..Self::new(kind)
        }
    }

    fn new_local(local_mod: LocalMod) -> Self {
        Self::new(ProfileModKind::Local(Box::new(local_mod)))
    }

    /// See [`ProfileModKind::uuid`]
    pub fn uuid(&self) -> Uuid {
        self.kind.uuid()
    }

    pub fn ident(&self) -> Cow<'_, VersionIdent> {
        self.kind.ident()
    }

    fn as_thunderstore(&self) -> Option<(&ThunderstoreMod, bool)> {
        self.kind
            .as_thunderstore()
            .map(|remote| (remote, self.enabled))
    }

    fn as_local(&self) -> Option<(&LocalMod, bool)> {
        self.kind.as_local().map(|local| (local, self.enabled))
    }
}

impl ProfileModKind {
    /// A unique ID for this mod in its profile - **not** unique across profiles.
    pub fn uuid(&self) -> Uuid {
        match self {
            ProfileModKind::Local(local_mod) => local_mod.uuid,
            ProfileModKind::Thunderstore(ts_mod) => ts_mod.id.package,
        }
    }

    pub fn ident(&self) -> Cow<'_, VersionIdent> {
        match self {
            ProfileModKind::Thunderstore(ts_mod) => Cow::Borrowed(&ts_mod.ident),
            ProfileModKind::Local(local_mod) => Cow::Owned(local_mod.ident()),
        }
    }

    pub fn as_thunderstore(&self) -> Option<&ThunderstoreMod> {
        match self {
            ProfileModKind::Thunderstore(ts_mod) => Some(ts_mod),
            _ => None,
        }
    }

    pub fn as_local(&self) -> Option<&LocalMod> {
        match self {
            ProfileModKind::Local(local) => Some(local),
            _ => None,
        }
    }

    /// Finds **all** dependencies of this mod.
    pub fn deps<'a>(&'a self, thunderstore: &'a Thunderstore) -> Vec<BorrowedMod<'a>> {
        let deps = match self {
            ProfileModKind::Local(local_mod) => local_mod.dependencies.as_ref(),
            ProfileModKind::Thunderstore(ts_mod) => ts_mod
                .id
                .borrow(thunderstore)
                .map(|borrowed| &borrowed.version.dependencies)
                .ok(),
        };

        match deps {
            Some(deps) => thunderstore.resolve_deps(deps.iter()).0,
            None => Vec::new(),
        }
    }
}

impl Profile {
    fn is_valid_name(name: &str) -> bool {
        const FORBIDDEN: &[char] = &['\\', '/', ':', '*', '?', '"', '<', '>', '|'];

        !name.is_empty()
            && !name.chars().all(char::is_whitespace)
            && name.chars().all(|c| !FORBIDDEN.contains(&c))
    }

    fn index_of(&self, uuid: Uuid) -> Result<usize> {
        self.mods
            .iter()
            .position(|p| p.uuid() == uuid)
            .context("mod not found in profile")
    }

    fn get_mod(&self, uuid: Uuid) -> Result<&ProfileMod> {
        self.mods
            .iter()
            .find(|p| p.uuid() == uuid)
            .context("mod not found in profile")
    }

    fn get_mod_mut(&mut self, uuid: Uuid) -> Result<&mut ProfileMod> {
        self.mods
            .iter_mut()
            .find(|p| p.uuid() == uuid)
            .context("mod not found in profile")
    }

    pub fn has_mod(&self, uuid: Uuid) -> bool {
        self.get_mod(uuid).is_ok()
    }

    fn thunderstore_mods(&self) -> impl Iterator<Item = (&ThunderstoreMod, bool)> {
        self.mods.iter().filter_map(ProfileMod::as_thunderstore)
    }

    fn local_mods(&self) -> impl Iterator<Item = (&LocalMod, bool)> {
        self.mods.iter().filter_map(ProfileMod::as_local)
    }

    fn save_data(&self) -> ProfileSaveData {
        ProfileSaveData {
            modpack: self.modpack.clone(),
            mods: self.mods.clone(),
            ignored_updates: self.ignored_updates.clone(),
        }
    }

    /// Finds the dependants of a mod in this profile.
    ///
    /// This is an expensive operation, use with care!
    fn dependants<'a>(
        &'a self,
        uuid: Uuid,
        thunderstore: &'a Thunderstore,
    ) -> impl Iterator<Item = &ProfileMod> + 'a {
        self.mods
            .iter()
            .filter(move |other| other.uuid() != uuid)
            .filter(move |other| {
                other
                    .kind
                    .deps(thunderstore)
                    .iter()
                    .any(|dep| dep.package.uuid4 == uuid)
            })
    }

    fn bepinex_log_path(&self) -> Result<PathBuf> {
        let path = self.path.join("BepInEx").join("LogOutput.log");

        if !path.exists() {
            bail!("no log file found");
        }

        Ok(path)
    }

    fn load(mut path: PathBuf) -> Result<Option<Self>> {
        path.push("profile.json");

        if !path.exists() {
            return Ok(None);
        }

        let manifest: ProfileSaveData = util::fs::read_json(&path).with_context(|| {
            format!(
                "failed to read profile manifest for '{}'",
                path.parent()
                    .unwrap()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
            )
        })?;

        path.pop();

        let profile = Profile {
            modpack: manifest.modpack,
            name: util::fs::file_name_owned(&path),
            mods: manifest.mods,
            linked_config: HashMap::new(),
            config: Vec::new(),
            ignored_updates: manifest.ignored_updates,
            path,
        };

        Ok(Some(profile))
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Dependant {
    pub full_name: VersionIdent,
    pub uuid: Uuid,
}

impl From<BorrowedMod<'_>> for Dependant {
    fn from(value: BorrowedMod) -> Self {
        Self {
            full_name: value.version.ident.clone(),
            uuid: value.package.uuid4,
        }
    }
}

impl From<&ProfileMod> for Dependant {
    fn from(value: &ProfileMod) -> Self {
        Self {
            full_name: value.ident().into_owned(),
            uuid: value.uuid(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "dependants")]
pub enum ModActionResponse {
    Done,
    HasDependants(Vec<Dependant>),
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

    fn remove_mod(&mut self, uuid: Uuid, thunderstore: &Thunderstore) -> Result<ModActionResponse> {
        if self.get_mod(uuid)?.enabled {
            if let Some(dependants) = self.check_dependants(uuid, thunderstore) {
                return Ok(ModActionResponse::HasDependants(dependants));
            }
        }

        self.force_remove_mod(uuid)?;
        Ok(ModActionResponse::Done)
    }

    fn force_remove_mod(&mut self, uuid: Uuid) -> Result<()> {
        let index = self.index_of(uuid)?;

        self.scan_mod(&self.mods[index].kind, |dir| {
            fs::remove_dir_all(dir).fs_context("removing mod directory", dir)
        })?;

        self.mods.remove(index);

        Ok(())
    }

    fn toggle_mod(&mut self, uuid: Uuid, thunderstore: &Thunderstore) -> Result<ModActionResponse> {
        let dependants = match self.get_mod(uuid)?.enabled {
            true => self.check_dependants(uuid, thunderstore),
            false => self.check_deps(uuid, thunderstore),
        };

        match dependants {
            Some(dependants) => Ok(ModActionResponse::HasDependants(dependants)),
            None => {
                self.force_toggle_mod(uuid)?;
                Ok(ModActionResponse::Done)
            }
        }
    }

    fn force_toggle_mod(&mut self, uuid: Uuid) -> Result<()> {
        let profile_mod = self.get_mod(uuid)?;
        let state = profile_mod.enabled;
        let new_state = !state;

        self.scan_mod(&profile_mod.kind, |dir| {
            let files = WalkDir::new(dir)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|entry| {
                    let file_type = entry.file_type();
                    file_type.is_file() || file_type.is_symlink()
                });

            for file in files {
                let path = file.path();
                if new_state {
                    // remove any .old extensions
                    let mut new = path.to_owned();
                    while let Some("old") = new.extension().and_then(|ext| ext.to_str()) {
                        new.set_extension("");
                    }

                    fs::rename(path, &new).fs_context("removing .old extension", path)?;
                } else {
                    let mut new = path.to_path_buf();
                    new.add_ext("old");
                    fs::rename(path, &new).fs_context("adding .old extension", path)?;
                }
            }

            Ok(())
        })?;

        self.get_mod_mut(uuid)
            .expect("profile mod should already be validated through get_mod")
            .enabled = new_state;

        Ok(())
    }

    fn check_dependants(&self, uuid: Uuid, thunderstore: &Thunderstore) -> Option<Vec<Dependant>> {
        let dependants = self
            .dependants(uuid, thunderstore)
            .filter(|profile_mod| {
                if !profile_mod.enabled {
                    return false;
                }

                match &profile_mod.kind {
                    ProfileModKind::Local(_) => true,
                    ProfileModKind::Thunderstore(ts_mod) => {
                        match ts_mod.id.borrow(thunderstore) {
                            Ok(borrowed) => !borrowed.package.is_modpack(), // ignore modpacks
                            Err(_) => false,
                        }
                    }
                }
            })
            .map_into()
            .collect_vec();

        match dependants.is_empty() {
            true => None,
            false => Some(dependants),
        }
    }

    fn check_deps(&self, uuid: Uuid, thunderstore: &Thunderstore) -> Option<Vec<Dependant>> {
        let profile_mod = self.get_mod(uuid).ok()?;

        let disabled_deps = profile_mod
            .kind
            .deps(thunderstore)
            .into_iter()
            .filter(|dep| {
                self.get_mod(dep.package.uuid4)
                    .is_ok_and(|profile_mod| !profile_mod.enabled)
            })
            .map_into()
            .collect_vec();

        match disabled_deps.is_empty() {
            true => None,
            false => Some(disabled_deps),
        }
    }

    fn scan_mod<F>(&self, profile_mod: &ProfileModKind, scan_dir: F) -> Result<()>
    where
        F: Fn(&Path) -> Result<()>,
    {
        let mut path = self.path.join("BepInEx");

        let ident = profile_mod.ident();

        for dir in ["core", "patchers", "plugins"].into_iter() {
            path.push(dir);
            path.push(ident.full_name());

            if path.exists() {
                scan_dir(&path)?;
            }

            path.pop();
            path.pop();
        }

        Ok(())
    }

    fn reorder_mod(&mut self, uuid: Uuid, delta: i32) -> Result<()> {
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
    fn create_profile(&mut self, name: String) -> Result<&mut Profile> {
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
            ignored_updates: HashSet::new(),
            modpack: None,
        };
        self.profiles.push(profile);

        let index = self.profiles.len() - 1;
        self.active_profile_index = index;
        Ok(&mut self.profiles[index])
    }

    fn delete_profile(&mut self, index: usize, allow_delete_last: bool) -> Result<()> {
        ensure!(
            allow_delete_last || self.profiles.len() > 1,
            "cannot delete last profile"
        );

        let profile = self.profile(index)?;

        fs::remove_dir_all(&profile.path)?;
        self.profiles.remove(index);

        self.active_profile_index = 0;

        Ok(())
    }

    fn duplicate_profile(&mut self, duplicate_name: String, index: usize) -> Result<()> {
        self.create_profile(duplicate_name)?;
        let profile = self.profile(index)?;
        let new_profile = self.active_profile();

        util::fs::copy_dir(&profile.path, &new_profile.path, Overwrite::Yes)?;

        let mods = profile.mods.clone();

        let new_profile = self.active_profile_mut();

        new_profile.mods = mods;

        Ok(())
    }

    fn profile_index(&self, name: &str) -> Option<usize> {
        self.profiles
            .iter()
            .position(|profile| profile.name == name)
    }

    fn profile(&self, index: usize) -> Result<&Profile> {
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

    pub fn set_active_profile(
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
                .thunderstore_mods()
                .map(|(ts_mod, _)| ts_mod.id.borrow(thunderstore))
        })
    }

    fn load(mut path: PathBuf) -> Result<Option<(&'static Game, Self)>> {
        let file_name = util::fs::file_name_owned(&path);
        let Some(game) = games::from_slug(&file_name) else {
            return Ok(None);
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
                if let Some(profile) = Profile::load(path)? {
                    profiles.push(profile);
                }
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
                active_profile_index,
                favorite: data.favorite,
            },
        )))
    }
}

const DEFAULT_GAME_SLUG: &str = "among-us";

impl ModManager {
    pub fn create(prefs: &Prefs) -> Result<Self> {
        let save_path = prefs.data_dir.join("manager.json");
        let save_data = match save_path.try_exists()? {
            true => util::fs::read_json(&save_path).context("failed to read manager save data")?,
            false => ManagerSaveData {
                active_game: DEFAULT_GAME_SLUG.to_owned(),
            },
        };

        let mut games = HashMap::new();

        for entry in prefs.data_dir.read_dir()? {
            let path = entry?.path();

            if path.is_dir() {
                if let Some((game, game_data)) = ManagerGame::load(path)? {
                    games.insert(&game.slug, game_data);
                }
            }
        }

        let active_game = games::from_slug(&save_data.active_game).unwrap_or_else(|| {
            games::from_slug(DEFAULT_GAME_SLUG).expect("default game should be valid")
        });

        let mut manager = Self { games, active_game };

        manager.ensure_game(manager.active_game, prefs)?;
        manager.save(prefs)?;

        Ok(manager)
    }

    pub fn active_game(&self) -> &ManagerGame {
        self.games
            .get(&self.active_game.slug)
            .expect("active game not found")
    }

    pub fn active_game_mut(&mut self) -> &mut ManagerGame {
        self.games
            .get_mut(&self.active_game.slug)
            .expect("active game not found")
    }

    pub fn active_profile(&self) -> &Profile {
        self.active_game().active_profile()
    }

    pub fn active_profile_mut(&mut self) -> &mut Profile {
        self.active_game_mut().active_profile_mut()
    }

    pub fn set_active_game(
        &mut self,
        game: &'static Game,
        thunderstore: &mut Thunderstore,
        prefs: &Prefs,
        app: AppHandle,
    ) -> Result<()> {
        self.ensure_game(game, prefs)?;

        if self.active_game.slug != game.slug {
            self.active_game = game;
            thunderstore.switch_game(game, app);
        }

        Ok(())
    }

    fn ensure_game(&mut self, game: &'static Game, prefs: &Prefs) -> Result<()> {
        if self.games.contains_key(&game.slug) {
            return Ok(());
        }

        let mut manager_game = ManagerGame {
            game,
            profiles: Vec::new(),
            path: prefs.data_dir.join(&game.slug),
            favorite: false,
            active_profile_index: 0,
        };

        manager_game.create_profile("Default".to_owned())?;
        self.games.insert(&game.slug, manager_game);

        Ok(())
    }

    pub fn cache_mods(&self, thunderstore: &Thunderstore) -> Result<()> {
        let packages = self
            .active_game()
            .profiles
            .iter()
            .flat_map(|profile| {
                profile
                    .thunderstore_mods()
                    .map(|(ts_mod, _)| ts_mod.id.borrow(thunderstore))
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
                active_game: self.active_game.slug.to_owned(),
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

                util::fs::write_json(&path, &profile.save_data(), JsonStyle::Pretty)?;

                path.pop();
                path.pop();
            }
            path.pop();

            path.pop();
        }

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

    let manager = app.state::<Mutex<ModManager>>();
    let prefs = app.state::<Mutex<Prefs>>();

    let mut manager = manager.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    manager.active_profile_mut().reorder_mod(uuid, delta)?;

    save(&manager, &prefs)?;

    Ok(())
}
