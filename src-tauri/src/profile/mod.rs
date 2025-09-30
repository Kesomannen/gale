use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};
use export::modpack::ModpackArgs;
use eyre::{anyhow, ensure, eyre, Context, ContextCompat, OptionExt, Result};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tracing::{info, warn};
use uuid::Uuid;

use crate::{
    config::ConfigCache,
    db::{self, Db},
    game::{self, mod_loader::ModLoader, Game},
    prefs::Prefs,
    state::ManagerExt,
    thunderstore::{self, BorrowedMod, ModId, Thunderstore, VersionIdent},
    util::fs::PathExt,
};

pub mod commands;
pub mod export;
pub mod import;
pub mod install;
pub mod launch;
pub mod sync;
pub mod update;

mod actions;
mod query;

pub fn setup(data: db::SaveData, prefs: &Prefs, db: &Db, app: &AppHandle) -> Result<ModManager> {
    actions::setup(app)?;

    ModManager::create(data, prefs, db)
}

/// The main state of the app.
#[derive(Debug)]
pub struct ModManager {
    /// Holds all the currently managed games.
    ///
    /// Note that this only contains entries for `Game`s
    /// which the user has selected at least once.
    pub games: HashMap<Game, ManagedGame>,
    pub active_game: Game,
}

/// Stores profiles and other state for one game.
#[derive(Debug)]
pub struct ManagedGame {
    pub id: i64,
    pub game: Game,
    pub path: PathBuf,
    pub profiles: Vec<Profile>,
    pub favorite: bool,
    pub active_profile_id: i64,
}

#[derive(Debug)]
pub struct Profile {
    pub id: i64,
    pub name: String,
    pub path: PathBuf,
    pub mods: Vec<ProfileMod>,
    pub game: Game,
    pub ignored_updates: HashSet<Uuid>,
    pub config_cache: ConfigCache,
    pub linked_config: HashMap<Uuid, PathBuf>,
    pub modpack: Option<ModpackArgs>,
    pub sync: Option<sync::SyncProfileData>,
    pub custom_args: Vec<String>,
    pub custom_args_enabled: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProfileMod {
    pub enabled: bool,

    #[serde(default = "Utc::now")]
    pub install_time: DateTime<Utc>,

    #[serde(flatten)]
    pub kind: ProfileModKind,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", untagged)]
pub enum ProfileModKind {
    Thunderstore(ThunderstoreMod),
    // Box to decrease size of enum, since this variant is rare and much larger
    Local(Box<LocalMod>),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ThunderstoreMod {
    #[serde(rename = "fullName")]
    pub ident: VersionIdent,

    #[serde(flatten)]
    pub id: ModId,
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

    fn get_mod_ok(&self, uuid: Uuid) -> Option<&ProfileMod> {
        self.mods.iter().find(|p| p.uuid() == uuid)
    }

    fn get_mod(&self, uuid: Uuid) -> Result<&ProfileMod> {
        self.get_mod_ok(uuid).ok_or_eyre("mod not found in profile")
    }

    fn get_mod_ok_mut(&mut self, uuid: Uuid) -> Option<&mut ProfileMod> {
        self.mods.iter_mut().find(|p| p.uuid() == uuid)
    }

    fn get_mod_mut(&mut self, uuid: Uuid) -> Result<&mut ProfileMod> {
        self.get_mod_ok_mut(uuid)
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
    ) -> impl Iterator<Item = &'a ProfileMod> + 'a {
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
        let mod_loader = &self.game.mod_loader;
        let relative = mod_loader
            .log_path()
            .ok_or_else(|| eyre!("log file is unsupported for {}", mod_loader.as_str()))?;

        self.path
            .join(relative)
            .exists_or_none()
            .ok_or_eyre("no log file found")
    }

    fn to_frontend(&self) -> FrontendProfile {
        FrontendProfile {
            id: self.id,
            name: self.name.clone(),
            mod_count: self.mods.len(),
            sync: self.sync.clone(),
            custom_args: self.custom_args.clone(),
            custom_args_enabled: self.custom_args_enabled,
        }
    }

    pub fn save(&self, app: &AppHandle, notify_frontend: bool) -> Result<()> {
        if notify_frontend {
            self.notify_frontend(app)?;
        }

        app.db().save_profile(self)
    }

    pub fn notify_frontend(&self, app: &AppHandle) -> Result<()> {
        app.emit("profile_changed", self.to_frontend())?;
        Ok(())
    }
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FrontendManagedGame {
    active_id: i64,
    profiles: Vec<FrontendProfile>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FrontendProfile {
    id: i64,
    name: String,
    mod_count: usize,
    sync: Option<sync::SyncProfileData>,
    custom_args: Vec<String>,
    custom_args_enabled: bool,
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
    pub readme: Option<String>,
    pub changelog: Option<String>,
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
    pub fn find_profile_index(&self, name: &str) -> Option<usize> {
        self.profiles
            .iter()
            .position(|profile| profile.name == name)
    }

    pub fn index_of(&self, profile_id: i64) -> Result<usize> {
        self.profiles
            .iter()
            .position(|profile| profile.id == profile_id)
            .ok_or_else(|| eyre!("profile with id {} not found", profile_id))
    }

    fn profile_at(&self, index: usize) -> Result<&Profile> {
        self.profiles
            .get(index)
            .ok_or_else(|| anyhow!("profile index {} is out of bounds", index))
    }

    fn profile_ok(&self, id: i64) -> Option<&Profile> {
        self.profiles.iter().find(|profile| profile.id == id)
    }

    fn profile(&self, id: i64) -> Result<&Profile> {
        self.profile_ok(id)
            .with_context(|| format!("profile with id {id} not found"))
    }

    fn profile_ok_mut(&mut self, id: i64) -> Option<&mut Profile> {
        self.profiles.iter_mut().find(|profile| profile.id == id)
    }

    /*
    fn profile_mut(&mut self, id: i64) -> Result<&mut Profile> {
        self.profile_ok_mut(id)
            .with_context(|| format!("profile with id {id} not found"))
    }
    */

    fn active_profile(&self) -> &Profile {
        self.profile(self.active_profile_id).unwrap()
    }

    fn active_profile_mut(&mut self) -> &mut Profile {
        self.profiles
            .iter_mut()
            .find(|profile| profile.id == self.active_profile_id)
            .expect("active profile not found")
    }

    pub fn set_active_profile(&mut self, index: usize) -> Result<&mut Profile> {
        ensure!(
            index < self.profiles.len(),
            "profile index {} is out of bounds",
            index
        );

        let profile = &mut self.profiles[index];
        self.active_profile_id = profile.id;

        Ok(profile)
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

    pub fn update_window_title(&self, app: &AppHandle) -> Result<()> {
        let title = format!("{} | {} - Gale", self.active_profile().name, self.game.name);
        app.get_webview_window("main")
            .unwrap()
            .set_title(&title)
            .ok();

        Ok(())
    }

    pub fn save(&self, app: &AppHandle) -> Result<()> {
        app.emit("game_changed", self.to_frontend())?;

        app.db().save_game(self)
    }

    fn to_frontend(&self) -> FrontendManagedGame {
        FrontendManagedGame {
            active_id: self.active_profile_id,
            profiles: self
                .profiles
                .iter()
                .map(|profile| profile.to_frontend())
                .collect(),
        }
    }
}

impl ModManager {
    pub fn create(data: db::SaveData, prefs: &Prefs, db: &Db) -> Result<Self> {
        const DEFAULT_GAME_SLUG: &str = "among-us";

        let db::SaveData {
            manager,
            games,
            profiles,
        } = data;

        let active_game = manager
            .active_game_slug
            .and_then(|slug| game::from_slug(&slug))
            .unwrap_or_else(|| game::from_slug(DEFAULT_GAME_SLUG).unwrap());

        let mut manager = Self {
            games: HashMap::new(),
            active_game,
        };

        let path = prefs.data_dir.to_path_buf();

        for saved_game in games {
            manager.add_saved_game(&path, saved_game)?;
        }

        for saved_profile in profiles {
            let path = PathBuf::from(saved_profile.path);

            if !path.exists() {
                warn!(
                    "profile {} at {} does not exist anymore",
                    saved_profile.name,
                    path.display()
                );
                if let Err(err) = db.delete_profile(saved_profile.id) {
                    warn!("failed to delete missing profile from database: {:#}", err);
                }
                continue;
            }

            let game = game::from_slug(&saved_profile.game_slug).ok_or_else(|| {
                eyre!(
                    "profile {} is in unknown game: {}",
                    saved_profile.name,
                    saved_profile.game_slug
                )
            })?;

            let profile = Profile {
                path,
                game,
                id: saved_profile.id,
                name: saved_profile.name,
                mods: saved_profile.mods,
                modpack: saved_profile.modpack,
                ignored_updates: saved_profile.ignored_updates.unwrap_or_default(),
                config_cache: ConfigCache::default(),
                linked_config: HashMap::new(),
                sync: saved_profile.sync_data,
                custom_args: saved_profile.custom_args.unwrap_or_default(),
                custom_args_enabled: saved_profile.custom_args_enabled.unwrap_or(false),
            };

            manager
                .ensure_game(game, false, prefs, db)?
                .profiles
                .push(profile);
        }

        manager.ensure_game(manager.active_game, true, prefs, db)?;
        db.save_all(&manager)?;

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

    pub fn profile_by_id(&self, id: i64) -> Result<(Game, &Profile)> {
        self.games
            .iter()
            .find_map(|(game, managed_game)| {
                managed_game.profile_ok(id).map(|profile| (*game, profile))
            })
            .ok_or_else(|| eyre!("profile with id {} not found", id))
    }

    pub fn profile_by_id_mut(&mut self, id: i64) -> Result<(Game, &mut Profile)> {
        self.games
            .iter_mut()
            .find_map(|(game, managed_game)| {
                managed_game
                    .profile_ok_mut(id)
                    .map(|profile| (*game, profile))
            })
            .ok_or_else(|| eyre!("profile with id {} not found", id))
    }

    pub fn set_active_game(&mut self, game: Game, app: &AppHandle) -> Result<&ManagedGame> {
        self.ensure_game(game, true, &app.lock_prefs(), app.db())?;

        if self.active_game != game {
            self.active_game = game;

            let mut thunderstore = app.lock_thunderstore();
            thunderstore.switch_game(game, app.clone());
        }

        Ok(self.active_game())
    }

    fn ensure_game<'a>(
        &'a mut self,
        game: Game,
        verify_profiles: bool,
        prefs: &Prefs,
        db: &Db,
    ) -> Result<&'a mut ManagedGame> {
        if !self.games.contains_key(game) {
            self.manage_game(game, prefs, db)?;
        }

        let managed = self
            .games
            .get_mut(game)
            .expect("newly managed game not found");

        if verify_profiles && managed.profile(managed.active_profile_id).is_err() {
            if managed.profiles.is_empty() {
                warn!("game {} has no profiles", game.slug);
                managed.create_default_profile(db).with_context(|| {
                    format!("failed to create default profile for {}", game.slug)
                })?;
            } else {
                warn!("active profile was out of bounds");
                managed.active_profile_id = managed.profiles[0].id;
            }
        }

        Ok(managed)
    }

    fn manage_game(&mut self, game: Game, prefs: &Prefs, db: &Db) -> Result<()> {
        info!("managing new game: {}", game.slug);

        let path = prefs.data_dir.join(&*game.slug);
        let id = self.games.values().map(|game| game.id).max().unwrap_or(0) + 1;

        let mut managed = ManagedGame {
            id,
            game,
            path,
            profiles: Vec::new(),
            favorite: false,
            active_profile_id: 0,
        };

        if let Err(err) = managed.create_default_profile(db) {
            warn!(
                "failed to create default profile for {}: {:#}",
                game.slug, err
            )
        }

        self.games.insert(game, managed);
        Ok(())
    }

    fn cache_mods(&self, thunderstore: &Thunderstore, prefs: &Prefs) -> Result<()> {
        let packages = self
            .active_game()
            .installed_mods(thunderstore)
            .map(|borrowed| borrowed.package)
            .unique()
            .collect_vec();

        thunderstore::cache::write_packages(&packages, self.active_game, prefs)
    }

    fn add_saved_game(&mut self, base_path: &Path, saved_game: db::ManagedGameData) -> Result<()> {
        let game = game::from_slug(&saved_game.slug).ok_or_else(|| {
            eyre!(
                "unknown game in save: {} (has Gale been downgraded?)",
                saved_game.slug
            )
        })?;

        let managed_game = ManagedGame {
            id: saved_game.id,
            game,
            profiles: Vec::new(),
            favorite: saved_game.favorite,
            active_profile_id: saved_game.active_profile_id,
            path: base_path.join(&*game.slug),
        };

        self.games.insert(game, managed_game);
        Ok(())
    }

    pub fn save_all(&self, app: &AppHandle) -> Result<()> {
        app.emit("game_changed", self.active_game().to_frontend())?;

        app.db().save_all(self)
    }

    pub fn save(&self, app: &AppHandle) -> Result<()> {
        app.db().save_manager(self)
    }

    pub fn save_active_game(&self, app: &AppHandle) -> Result<()> {
        self.active_game().save(app)
    }

    pub fn save_active_profile(&self, app: &AppHandle, notify_frontend: bool) -> Result<()> {
        self.active_profile().save(app, notify_frontend)
    }
}
