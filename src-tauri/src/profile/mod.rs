use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::Mutex,
    time::Instant,
};

use chrono::{DateTime, Utc};
use export::modpack::ModpackArgs;
use eyre::{anyhow, ensure, Context, OptionExt, Result};
use itertools::Itertools;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Listener, Manager};
use uuid::Uuid;

use crate::{
    config::ConfigCache,
    game::{self, Game, ModLoader},
    logger,
    prefs::Prefs,
    thunderstore::{self, BorrowedMod, ModId, Thunderstore, VersionIdent},
    util::{
        self,
        error::IoResultExt,
        fs::{JsonStyle, PathExt},
    },
};

pub mod commands;
pub mod export;
pub mod import;
pub mod install;
pub mod launch;
pub mod update;

mod actions;
mod query;

pub fn setup(app: &AppHandle) -> Result<()> {
    {
        let prefs = app.state::<Mutex<Prefs>>();
        let prefs = prefs.lock().unwrap();

        let manager = ModManager::create(&prefs)?;
        app.manage(Mutex::new(manager));
    }

    install::setup(app).context("failed to initialize downloader")?;

    let handle = app.to_owned();
    app.listen("reorder_mod", move |event| {
        if let Err(err) = actions::handle_reorder_event(event, &handle) {
            logger::log_webview_err("Failed to reorder mod", err, &handle);
        }
    });

    Ok(())
}

/// The main state of the app.
pub struct ModManager {
    /// Holds all the currently managed games.
    ///
    /// Note that this only contains entries for `Game`s
    /// which the user has selected at least once.
    pub games: HashMap<Game, ManagedGame>,
    pub active_game: Game,
}

/// Persistent data for ModManager
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManagerSaveData {
    active_game: String,
}

/// Stores profiles and other state for one game.
pub struct ManagedGame {
    pub game: Game,
    pub profiles: Vec<Profile>,
    pub path: PathBuf,
    pub favorite: bool,
    pub active_profile_index: usize,
}

/// Persistent data for ManagerGame
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManagedGameSaveData {
    favorite: bool,
    active_profile_index: usize,
}

pub struct Profile {
    pub name: String,
    pub path: PathBuf,
    pub mods: Vec<ProfileMod>,
    pub game: Game,
    pub ignored_updates: HashSet<Uuid>,
    pub config_cache: ConfigCache,
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
    #[serde(rename = "fullName")]
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

    pub fn full_name(&self) -> Cow<'_, str> {
        self.kind.full_name()
    }

    fn as_thunderstore(&self) -> Option<(&ThunderstoreMod, bool)> {
        self.kind
            .as_thunderstore()
            .map(|remote| (remote, self.enabled))
    }

    fn as_local(&self) -> Option<(&LocalMod, bool)> {
        self.kind.as_local().map(|local| (local, self.enabled))
    }

    /// Finds all dependencies of this mod.
    ///
    /// See [`Thunderstore::dependencies`] for more information.
    pub fn dependencies<'a>(
        &'a self,
        thunderstore: &'a Thunderstore,
    ) -> impl Iterator<Item = BorrowedMod<'a>> {
        self.kind.dependencies(thunderstore)
    }
}

impl ProfileModKind {
    /// A unique ID for this mod in its profile - **not** unique across profiles.
    pub fn uuid(&self) -> Uuid {
        match self {
            ProfileModKind::Local(local_mod) => local_mod.uuid,
            ProfileModKind::Thunderstore(ts_mod) => ts_mod.id.package_uuid,
        }
    }

    pub fn ident(&self) -> Cow<'_, VersionIdent> {
        match self {
            ProfileModKind::Thunderstore(ts_mod) => Cow::Borrowed(&ts_mod.ident),
            ProfileModKind::Local(local_mod) => Cow::Owned(local_mod.ident()),
        }
    }

    pub fn full_name(&self) -> Cow<'_, str> {
        match self.ident() {
            Cow::Borrowed(borrow) => Cow::Borrowed(borrow.full_name()),
            Cow::Owned(owned) => Cow::Owned(owned.name().to_owned()),
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

    /// Finds all dependencies of this mod.
    ///
    /// See [`Thunderstore::dependencies`] for more information.
    pub fn dependencies<'a>(
        &'a self,
        thunderstore: &'a Thunderstore,
    ) -> impl Iterator<Item = BorrowedMod<'a>> {
        let idents = match self {
            ProfileModKind::Local(local_mod) => local_mod.dependencies.as_ref(),
            ProfileModKind::Thunderstore(ts_mod) => ts_mod
                .id
                .borrow(thunderstore)
                .map(|borrowed| &borrowed.version.dependencies)
                .ok(),
        };

        idents
            .into_iter()
            .flat_map(|deps| thunderstore.dependencies(deps))
    }
}

impl Profile {
    fn new(name: String, path: PathBuf, game: Game) -> Self {
        Self {
            name,
            path,
            game,
            mods: Vec::new(),
            ignored_updates: HashSet::new(),
            config_cache: ConfigCache::default(),
            linked_config: HashMap::new(),
            modpack: None,
        }
    }

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
            .ok_or_eyre("mod not found in profile")
    }

    fn get_mod(&self, uuid: Uuid) -> Result<&ProfileMod> {
        self.mods
            .iter()
            .find(|p| p.uuid() == uuid)
            .ok_or_eyre("mod not found in profile")
    }

    fn get_mod_mut(&mut self, uuid: Uuid) -> Result<&mut ProfileMod> {
        self.mods
            .iter_mut()
            .find(|p| p.uuid() == uuid)
            .ok_or_eyre("mod not found in profile")
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

    /// Finds all the dependants of a mod in this profile.
    ///
    /// This includes both direct and indirect dependencies.
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
                    .dependencies(thunderstore)
                    .any(|dep| dep.package.uuid == uuid)
            })
    }

    /// Recursively finds the dependencies of the given mods and filters
    /// out those already installed.
    fn missing_deps<'a>(
        &'a self,
        idents: impl IntoIterator<Item = &'a VersionIdent>,
        thunderstore: &'a Thunderstore,
    ) -> impl Iterator<Item = BorrowedMod<'a>> + 'a {
        thunderstore
            .dependencies(idents)
            .filter(|dep| !self.has_mod(dep.package.uuid))
    }

    fn log_path(&self) -> Result<PathBuf> {
        self.path
            .join(self.game.mod_loader.log_path())
            .exists_or_none()
            .ok_or_eyre("no log file found")
    }

    fn load(mut path: PathBuf, game: Game) -> Result<Option<Self>> {
        path.push("profile.json");

        if !path.exists() {
            warn!(
                "profile directory at {} does not contain a manifest, skipping",
                path.display()
            );
            return Ok(None);
        }

        let manifest: ProfileSaveData = util::fs::read_json(&path)
            .with_context(|| format!("failed to read profile manifest"))?;

        path.pop();

        let name = util::fs::file_name_owned(&path);

        let profile = Self {
            modpack: manifest.modpack,
            mods: manifest.mods,
            ignored_updates: manifest.ignored_updates,
            ..Self::new(name, path, game)
        };

        Ok(Some(profile))
    }

    fn save_data(&self) -> ProfileSaveData {
        ProfileSaveData {
            modpack: self.modpack.clone(),
            mods: self.mods.clone(),
            ignored_updates: self.ignored_updates.clone(),
        }
    }

    fn save(&self, path: &mut PathBuf) -> Result<()> {
        path.push("profile.json");
        util::fs::write_json(&path, &self.save_data(), JsonStyle::Pretty)?;
        path.pop();

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct LocalMod {
    pub name: String,
    pub icon: Option<PathBuf>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub version: Option<semver::Version>,
    pub dependencies: Option<Vec<VersionIdent>>,
    pub uuid: Uuid,
    #[serde(default)]
    pub file_size: u64,
}

impl LocalMod {
    pub fn ident(&self) -> VersionIdent {
        let version = self.version.as_ref().map(|vers| vers.to_string());

        VersionIdent::new(
            self.author.as_deref().unwrap_or(""),
            &self.name,
            version.as_deref().unwrap_or(""),
        )
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Dependant {
    #[serde(rename = "fullName")]
    ident: VersionIdent,
    uuid: Uuid,
}

impl From<BorrowedMod<'_>> for Dependant {
    fn from(value: BorrowedMod) -> Self {
        Self {
            ident: value.version.ident.clone(),
            uuid: value.package.uuid,
        }
    }
}

impl From<&ProfileMod> for Dependant {
    fn from(value: &ProfileMod) -> Self {
        Self {
            ident: value.ident().into_owned(),
            uuid: value.uuid(),
        }
    }
}

impl ManagedGame {
    fn new(path: PathBuf, game: Game) -> Self {
        Self {
            game,
            path,
            profiles: Vec::new(),
            favorite: false,
            active_profile_index: 0,
        }
    }

    pub fn profile_index(&self, name: &str) -> Option<usize> {
        self.profiles
            .iter()
            .position(|profile| profile.name == name)
    }

    fn profile(&self, index: usize) -> Result<&Profile> {
        self.profiles
            .get(index)
            .ok_or_else(|| anyhow!("profile index {} is out of bounds", index))
    }

    fn active_profile(&self) -> &Profile {
        &self.profiles[self.active_profile_index]
    }

    fn active_profile_mut(&mut self) -> &mut Profile {
        &mut self.profiles[self.active_profile_index]
    }

    pub fn set_active_profile(&mut self, index: usize) -> Result<()> {
        ensure!(
            index < self.profiles.len(),
            "profile index {} is out of bounds",
            index
        );

        self.active_profile_index = index;

        info!(
            "set active profile for game {} to {} (index {})",
            self.game.slug, self.profiles[index].name, index
        );

        Ok(())
    }

    /// Returns an iterator over all installed thunderstore mods across all of the game's profiles.
    ///
    /// May contain duplicates.
    fn installed_mods<'a>(
        &'a self,
        thunderstore: &'a Thunderstore,
    ) -> impl Iterator<Item = BorrowedMod<'a>> + 'a {
        self.profiles.iter().flat_map(|profile| {
            profile
                .thunderstore_mods()
                .filter_map(|(ts_mod, _)| ts_mod.id.borrow(thunderstore).ok())
        })
    }

    fn load(mut path: PathBuf) -> Result<Option<(Game, Self)>> {
        let file_name = util::fs::file_name_owned(&path);

        let Some(game) = game::from_slug(&file_name) else {
            info!(
                "directory '{}' does not match any game, skipping",
                file_name
            );
            return Ok(None);
        };

        path.push("game.json");

        let data = util::fs::read_json::<ManagedGameSaveData>(&path)
            .context("failed to read game save data")?;

        path.pop();

        path.push("profiles");

        let mut profiles = Vec::new();

        for entry in path
            .read_dir()
            .context("failed to read profiles directory")?
        {
            let path = entry.context("failed to read profile directory")?.path();

            if path.is_dir() {
                let result = Profile::load(path.clone(), game).with_context(|| {
                    format!(
                        "failed to load profile {}",
                        path.file_name().unwrap().to_string_lossy()
                    )
                })?;

                if let Some(profile) = result {
                    profiles.push(profile);
                }
            }
        }

        path.pop();

        let active_profile_index = data
            .active_profile_index
            .min(profiles.len().saturating_sub(1));

        let result = Self {
            game,
            profiles,
            path,
            active_profile_index,
            favorite: data.favorite,
        };

        Ok(Some((game, result)))
    }

    fn save_data(&self) -> ManagedGameSaveData {
        ManagedGameSaveData {
            favorite: self.favorite,
            active_profile_index: self.active_profile_index,
        }
    }

    fn save(&self, path: &mut PathBuf) -> Result<()> {
        path.push("game.json");
        util::fs::write_json(&path, &self.save_data(), JsonStyle::Pretty)?;
        path.pop();

        path.push("profiles");

        for profile in &self.profiles {
            path.push(&profile.name);
            profile.save(path)?;
            path.pop();
        }

        path.pop();

        Ok(())
    }
}

impl ModManager {
    pub fn create(prefs: &Prefs) -> Result<Self> {
        const DEFAULT_GAME_SLUG: &str = "among-us";

        let path = prefs.data_dir.join("manager.json");
        let save = match path.exists_or_none() {
            Some(path) => util::fs::read_json(path).context("failed to read manager save data")?,
            None => ManagerSaveData {
                active_game: DEFAULT_GAME_SLUG.to_owned(),
            },
        };

        let mut games = HashMap::new();

        for entry in prefs
            .data_dir
            .read_dir()
            .fs_context("reading data directory", &prefs.data_dir)?
        {
            let path = entry.context("failed to read data directory entry")?.path();

            if !path.is_dir() {
                continue;
            }

            let result = ManagedGame::load(path.clone()).with_context(|| {
                format!(
                    "failed to load game {}",
                    path.file_name().unwrap().to_string_lossy()
                )
            })?;

            if let Some((game, manager_game)) = result {
                debug!(
                    "loaded game {} with {} profiles",
                    game.slug,
                    manager_game.profiles.len()
                );
                games.insert(game, manager_game);
            }
        }

        let active_game = game::from_slug(&save.active_game)
            .unwrap_or_else(|| game::from_slug(DEFAULT_GAME_SLUG).unwrap());

        let mut manager = Self { games, active_game };

        manager.ensure_game(manager.active_game, prefs)?;
        manager.save(prefs)?;

        Ok(manager)
    }

    pub fn active_mod_loader(&self) -> &'static ModLoader<'static> {
        &self.active_game.mod_loader
    }

    pub fn active_game(&self) -> &ManagedGame {
        self.games
            .get(&self.active_game)
            .expect("active game not found")
    }

    pub fn active_game_mut(&mut self) -> &mut ManagedGame {
        self.games
            .get_mut(&self.active_game)
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
        game: Game,
        thunderstore: &mut Thunderstore,
        prefs: &Prefs,
        app: AppHandle,
    ) -> Result<()> {
        self.ensure_game(game, prefs)?;

        if self.active_game != game {
            self.active_game = game;
            thunderstore.switch_game(game, app);
        }

        info!("set active game to {}", game.slug);

        Ok(())
    }

    fn ensure_game<'a>(&'a mut self, game: Game, prefs: &Prefs) -> Result<&'a mut ManagedGame> {
        const DEFAULT_PROFILE_NAME: &str = "Default";

        if self.games.contains_key(game) {
            debug!("{} is already managed", game.slug);
        } else {
            info!("managing new game: {}", game.slug);
            let path = prefs.data_dir.join(&*game.slug);

            let mut managed_game = ManagedGame::new(path, game);
            managed_game.create_profile(DEFAULT_PROFILE_NAME.to_owned())?;

            self.games.insert(game, managed_game);
        }

        Ok(self.games.get_mut(game).unwrap())
    }

    fn cache_mods(&self, thunderstore: &Thunderstore) -> Result<()> {
        let packages = self
            .active_game()
            .installed_mods(thunderstore)
            .map(|borrowed| borrowed.package)
            .unique()
            .collect_vec();

        thunderstore::write_cache(&packages, self)
    }

    fn save_data(&self) -> ManagerSaveData {
        ManagerSaveData {
            active_game: self.active_game.slug.to_string(),
        }
    }

    fn save(&self, prefs: &Prefs) -> Result<()> {
        let start = Instant::now();
        let mut path = prefs.data_dir.get().to_path_buf();

        path.push("manager.json");
        util::fs::write_json(&path, &self.save_data(), JsonStyle::Pretty)?;
        path.pop();

        for (game, managed_game) in &self.games {
            path.push(&*game.slug);
            managed_game.save(&mut path)?;
            path.pop();
        }

        debug!(
            "saved manager data to {} in {:?}",
            path.display(),
            start.elapsed()
        );

        Ok(())
    }
}
