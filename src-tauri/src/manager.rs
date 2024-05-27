use std::{
    cmp::Ordering,
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use uuid::Uuid;
use walkdir::WalkDir;

use crate::{
    config, fs_util, games::{self, Game}, prefs::Prefs, thunderstore::{
        models::FrontendProfileMod,
        query::{self, QueryModsArgs, Queryable, SortBy},
        BorrowedMod, ModRef, Thunderstore,
    }, util::IoResultExt
};
use chrono::{DateTime, Utc};
use itertools::Itertools;
use tauri::{AppHandle, Manager};

pub mod commands;
pub mod downloader;
pub mod exporter;
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
    enabled: bool,
    #[serde(default = "Utc::now")]
    install_time: DateTime<Utc>,
    #[serde(flatten)]
    kind: ProfileModKind,
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

    fn uuid(&self) -> &Uuid {
        self.kind.uuid()
    }

    fn into_remote(self) -> Option<(ModRef, bool)> {
        self.kind.into_remote().map(|remote| (remote, self.enabled))
    }

    fn as_remote(&self) -> Option<(&ModRef, bool)> {
        self.kind.as_remote().map(|remote| (remote, self.enabled))
    }

    fn as_local(&self) -> Option<(&LocalMod, bool)> {
        self.kind.as_local().map(|local| (local, self.enabled))
    }

    fn queryable<'a>(&'a self, thunderstore: &'a Thunderstore) -> Result<QueryableProfileMod<'a>> {
        let kind = match &self.kind {
            ProfileModKind::Local(local) => QueryableProfileModKind::Local(local),
            ProfileModKind::Remote(mod_ref) => {
                let borrow = mod_ref.borrow(thunderstore)?;
                QueryableProfileModKind::Remote(borrow)
            }
        };

        Ok(QueryableProfileMod {
            kind,
            install_time: self.install_time,
            enabled: self.enabled,
        })
    }
}

#[typeshare]
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", untagged)]
enum ProfileModKind {
    Local(Box<LocalMod>),
    Remote(ModRef),
}

fn default_true() -> bool {
    true
}

impl ProfileModKind {
    fn uuid(&self) -> &Uuid {
        match self {
            ProfileModKind::Local(local_mod) => &local_mod.uuid,
            ProfileModKind::Remote(mod_ref) => &mod_ref.package_uuid,
        }
    }

    fn into_remote(self) -> Option<ModRef> {
        match self {
            ProfileModKind::Remote(mod_ref) => Some(mod_ref),
            _ => None,
        }
    }

    fn as_remote(&self) -> Option<&ModRef> {
        match self {
            ProfileModKind::Remote(mod_ref) => Some(mod_ref),
            _ => None,
        }
    }

    fn as_local(&self) -> Option<&LocalMod> {
        match self {
            ProfileModKind::Local(local) => Some(local),
            _ => None,
        }
    }

    fn full_name<'a>(&'a self, thunderstore: &'a Thunderstore) -> Result<&'a str> {
        match self {
            ProfileModKind::Local(local) => Ok(&local.name),
            ProfileModKind::Remote(mod_ref) => {
                let package = thunderstore.get_package(&mod_ref.package_uuid)?;
                Ok(&package.full_name)
            }
        }
    }
}

struct QueryableProfileMod<'a> {
    enabled: bool,
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

        if let SortBy::InstallDate = args.sort_by {
            return self.install_time.cmp(&other.install_time);
        }

        match (&self.kind, &other.kind) {
            (Kind::Remote(a), Kind::Remote(b)) => a.cmp(b, args),
            (Kind::Local(a), Kind::Local(b)) => a.cmp(b, args),
            (Kind::Local(_), _) => Ordering::Greater,
            (_, Kind::Local(_)) => Ordering::Less,
        }
    }
}

impl From<QueryableProfileMod<'_>> for FrontendProfileMod {
    fn from(value: QueryableProfileMod<'_>) -> Self {
        let data = match value.kind {
            QueryableProfileModKind::Local(local) => local.clone().into(),
            QueryableProfileModKind::Remote(remote) => remote.into(),
        };

        FrontendProfileMod {
            data,
            enabled: value.enabled,
        }
    }
}

enum QueryableProfileModKind<'a> {
    Local(&'a LocalMod),
    Remote(BorrowedMod<'a>),
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileManifest<'a> {
    name: &'a str,
    mods: Vec<ProfileMod>,
}

pub struct Profile {
    pub name: String,
    pub path: PathBuf,
    pub mods: Vec<ProfileMod>,
    pub config: Vec<config::LoadFileResult>,
}

impl Profile {
    fn is_valid_name(name: &str) -> bool {
        name.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ' ')
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
            name: &self.name,
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
            .map(|p| p.queryable(thunderstore))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(query::query_mods(args, queryables.into_iter())
            .map(|queryable| queryable.into())
            .collect())
    }

    fn dependants<'a>(
        &self,
        target_mod: BorrowedMod<'a>,
        thunderstore: &'a Thunderstore,
    ) -> Result<Vec<BorrowedMod<'a>>> {
        self.remote_mods()
            .filter(|(other, _)| other.package_uuid != target_mod.package.uuid4)
            .map(|(other, _)| other.borrow(thunderstore))
            .filter_map_ok(|other| {
                let deps = thunderstore.dependencies(other.version).0;
                match deps.iter().any(|dep| dep.package == target_mod.package) {
                    true => Some(other),
                    false => None,
                }
            })
            .collect()
    }

    fn load(mut path: PathBuf) -> Result<Self> {
        path.push("profile.json");

        let manifest = fs::read_to_string(&path).fs_context("read profile manifest", &path)?;

        path.pop();

        let manifest: ProfileManifest =
            serde_json::from_str(&manifest).context("failed to parse profile manifest")?;

        let mut config = Vec::new();
        config::load_config(path.clone(), &mut config);

        Ok(Profile {
            name: manifest.name.to_owned(),
            mods: manifest.mods,
            config,
            path,
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
    fn remove_mod(
        &mut self,
        uuid: &Uuid,
        thunderstore: &Thunderstore,
    ) -> Result<ModActionResponse> {
        let profile_mod = self.get_mod(uuid)?;

        if profile_mod.enabled {
            if let Some((mod_ref, _)) = profile_mod.as_remote() {
                let borrowed = mod_ref.borrow(thunderstore)?;

                if let Some(dependants) = self.check_dependants(borrowed, thunderstore)? {
                    return Ok(ModActionResponse::HasDependants(dependants));
                }
            }
        }

        self.force_remove_mod(uuid, thunderstore)?;
        Ok(ModActionResponse::Done)
    }

    fn force_remove_mod(&mut self, uuid: &Uuid, thunderstore: &Thunderstore) -> Result<()> {
        let index = self
            .mods
            .iter()
            .position(|m| m.uuid() == uuid)
            .context("mod not found in profile")?;

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
                if let Some(dependants) = self.check_dependants(borrowed.clone(), thunderstore)? {
                    return Ok(ModActionResponse::HasDependants(dependants));
                }
            } else if let Some(deps) = self.check_deps(borrowed, thunderstore)? {
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
                    fs_util::add_extension(&mut new, "old");
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
    ) -> Result<Option<Vec<Dependant>>> {
        let ignored_category = "Modpacks".to_string();
        let dependants = self
            .dependants(borrowed_mod, thunderstore)?
            .into_iter()
            // filter out modpacks and disabled mods
            .filter(|m| {
                !m.package.categories.contains(&ignored_category)
                    && self.get_mod(&m.package.uuid4).unwrap().enabled
            })
            .map(Dependant::from)
            .collect::<Vec<_>>();

        Ok(match dependants.is_empty() {
            true => None,
            false => Some(dependants),
        })
    }

    fn check_deps(
        &self,
        borrowed_mod: BorrowedMod,
        thunderstore: &Thunderstore,
    ) -> Result<Option<Vec<Dependant>>> {
        let disabled_deps = thunderstore
            .dependencies(borrowed_mod.version)
            .0
            .into_iter()
            .filter_map(|dep| match self.get_mod(&dep.package.uuid4) {
                Ok(profile_mod) => match profile_mod.enabled {
                    true => None,
                    false => Some(Ok(dep)),
                },
                Err(e) => Some(Err(e)),
            })
            .map_ok(Dependant::from)
            .collect::<Result<Vec<_>>>()?;

        Ok(match disabled_deps.is_empty() {
            true => None,
            false => Some(disabled_deps),
        })
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
        let file_name = fs_util::file_name(&path);
        let game = match games::from_name(&file_name) {
            Some(game) => game,
            None => return Ok(None),
        };

        path.push("game.json");

        let json = fs::read_to_string(&path).fs_context("reading game save data", &path)?;
        let data: ManagerGameSaveData =
            serde_json::from_str(&json).context("failed to parse game save data")?;

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
                profiles,
                path,
                favorite: data.favorite,
                active_profile_index,
            },
        )))
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
