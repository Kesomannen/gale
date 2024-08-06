use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    str::{self, Split},
    sync::Mutex,
    time::{Duration, Instant},
};

use anyhow::{anyhow, Context, Result};
use indexmap::IndexMap;
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use tauri::{async_runtime::JoinHandle, AppHandle, Emitter, Manager};
use uuid::Uuid;

use crate::{
    games::Game,
    logger,
    manager::ModManager,
    prefs::Prefs,
    util::{self, fs::JsonStyle},
    NetworkClient,
};

use self::models::{PackageListing, PackageVersion};

pub mod commands;
pub mod models;
pub mod query;
pub mod token;

pub fn setup(app: &AppHandle) {
    let manager = app.state::<Mutex<ModManager>>();
    let manager = manager.lock().unwrap();

    let mut thunderstore = Thunderstore::default();
    thunderstore.switch_game(manager.active_game, app.clone());

    app.manage(Mutex::new(thunderstore));

    query::setup(app);
}

#[derive(Serialize, Debug, Clone)]
pub struct OwnedMod {
    pub package: PackageListing,
    pub version: PackageVersion,
}

impl From<BorrowedMod<'_>> for OwnedMod {
    fn from(borrowed_mod: BorrowedMod) -> Self {
        Self {
            package: borrowed_mod.package.clone(),
            version: borrowed_mod.version.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, Hash)]
pub struct BorrowedMod<'a> {
    pub package: &'a PackageListing,
    pub version: &'a PackageVersion,
}

impl<'a> BorrowedMod<'a> {
    pub fn reference(self) -> ModRef {
        self.into()
    }
}

impl<'a> From<BorrowedMod<'a>> for (&'a PackageListing, &'a PackageVersion) {
    fn from(borrowed_mod: BorrowedMod<'a>) -> Self {
        (borrowed_mod.package, borrowed_mod.version)
    }
}

impl<'a> From<(&'a PackageListing, &'a PackageVersion)> for BorrowedMod<'a> {
    fn from((package, version): (&'a PackageListing, &'a PackageVersion)) -> Self {
        Self { package, version }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModRef {
    pub package_uuid: Uuid,
    pub version_uuid: Uuid,
}

impl From<BorrowedMod<'_>> for ModRef {
    fn from(borrowed_mod: BorrowedMod<'_>) -> Self {
        Self {
            package_uuid: borrowed_mod.package.uuid4,
            version_uuid: borrowed_mod.version.uuid4,
        }
    }
}

impl ModRef {
    pub fn borrow<'a>(&self, thunderstore: &'a Thunderstore) -> Result<BorrowedMod<'a>> {
        thunderstore.get_mod(&self.package_uuid, &self.version_uuid)
    }
}

pub fn parse_mod_ident(identifier: &str, delimeter: char) -> Result<(String, &str)> {
    let (author, name, version) = parts(identifier.split(delimeter))
        .with_context(|| format!("invalid dependency string format {}", identifier))?;

    let full_name = format!("{}-{}", author, name);
    return Ok((full_name, version));

    fn parts(mut s: Split<'_, char>) -> Option<(&str, &str, &str)> {
        let author = s.next()?;
        let name = s.next()?;
        let version = s.next()?;

        Some((author, name, version))
    }
}

#[derive(Default)]
pub struct Thunderstore {
    pub load_mods_handle: Option<JoinHandle<()>>,
    pub packages_fetched: bool,
    pub is_fetching: bool,
    // IndexMap is not used for ordering here, but for fast iteration,
    // since we iterate over all mods when querying and resolving dependencies
    pub packages: IndexMap<Uuid, PackageListing>,
}

impl Thunderstore {
    pub fn switch_game(&mut self, game: &'static Game, app: AppHandle) {
        if let Some(handle) = self.load_mods_handle.take() {
            handle.abort();
        }

        self.packages_fetched = false;
        self.packages = IndexMap::new();

        let load_mods_handle = tauri::async_runtime::spawn(load_mods_loop(app, game));
        self.load_mods_handle = Some(load_mods_handle);
    }

    pub fn latest(&self) -> impl Iterator<Item = BorrowedMod<'_>> {
        self.packages.values().map(move |package| BorrowedMod {
            package,
            version: package.latest(),
        })
    }

    pub fn get_package<'a>(&'a self, uuid: &Uuid) -> Result<&'a PackageListing> {
        self.packages
            .get(uuid)
            .with_context(|| format!("package with id {} not found", uuid))
    }

    pub fn find_package<'a>(&'a self, full_name: &str) -> Result<&'a PackageListing> {
        self.packages
            .values()
            .find(|mod_listing| mod_listing.full_name == full_name)
            .with_context(|| format!("package {} not found", full_name))
    }

    pub fn get_mod<'a>(
        &'a self,
        package_uuid: &Uuid,
        version_uuid: &Uuid,
    ) -> Result<BorrowedMod<'a>> {
        let package = self.get_package(package_uuid)?;
        let version = package.get_version(version_uuid).with_context(|| {
            format!(
                "version with id {} not found in package {}",
                version_uuid, package.full_name
            )
        })?;

        Ok((package, version).into())
    }

    pub fn find_mod<'a>(&'a self, identifier: &str, delimeter: char) -> Result<BorrowedMod<'a>> {
        let (full_name, version) = parse_mod_ident(identifier, delimeter)?;

        let version = semver::Version::parse(version)
            .with_context(|| format!("invalid version format {}", version))?;

        let package = self.find_package(&full_name)?;
        let version = package
            .get_version_with_num(&version)
            .with_context(|| format!("version {} not found in package {}", version, full_name))?;

        Ok((package, version).into())
    }

    pub fn resolve_deps<'a>(
        &'a self,
        dependency_strings: impl Iterator<Item = &'a String>,
    ) -> (HashSet<BorrowedMod<'a>>, Vec<&'a str>) {
        let mut result = HashSet::new();
        let mut errors = Vec::new();
        let mut stack = dependency_strings.map(String::as_str).collect::<Vec<_>>();
        let mut visited = stack
            .iter()
            .map(|s| parse_author_name(s))
            .collect::<HashSet<_>>();

        while let Some(id) = stack.pop() {
            match self.find_mod(id, '-') {
                Ok(dependency) => {
                    for dep in &dependency.version.dependencies {
                        let (author, name) = parse_author_name(dep);

                        if !visited.insert((author, name)) {
                            continue;
                        }

                        stack.push(dep.as_str());
                    }

                    result.insert(dependency);
                }
                Err(_) => {
                    errors.push(id);
                }
            }
        }

        return (result, errors);

        fn parse_author_name(s: &str) -> (&str, &str) {
            let mut split = s.split('-');
            (split.next().unwrap(), split.next().unwrap())
        }
    }

    pub fn dependencies<'a>(
        &'a self,
        version: &'a PackageVersion,
    ) -> (HashSet<BorrowedMod<'a>>, Vec<&'a str>) {
        self.resolve_deps(version.dependencies.iter())
    }
}

const TIME_BETWEEN_LOADS: Duration = Duration::from_secs(60 * 15);

async fn load_mods_loop(app: AppHandle, game: &'static Game) {
    let manager = app.state::<Mutex<ModManager>>();
    let thunderstore = app.state::<Mutex<Thunderstore>>();
    let prefs = app.state::<Mutex<Prefs>>();

    {
        let manager = manager.lock().unwrap();

        match read_cache(&cache_path(&manager)) {
            Ok(Some(mods)) => {
                let mut thunderstore = thunderstore.lock().unwrap();

                for package in mods {
                    thunderstore.packages.insert(package.uuid4, package);
                }
            }
            Ok(None) => (),
            Err(err) => warn!("failed to read cache: {}", err),
        }
    }

    let mut is_first = true;
    loop {
        let fetch_automatically = prefs.lock().unwrap().fetch_mods_automatically();
        // this could happen if the user manually triggers a fetch, 
        // or the fetch is still ongoing from the last loop
        let is_fetching = thunderstore.lock().unwrap().is_fetching; 

        if fetch_automatically && !is_fetching {
            if let Err(err) = fetch_mods(&app, game, is_first).await {
                logger::log_js_err("error while fetching mods from Thunderstore", &err, &app);
            } else {
                is_first = false;

                // REMOVE IN THE FUTURE!
                let mut manager = manager.lock().unwrap();
                let thunderstore = thunderstore.lock().unwrap();

                manager.fill_profile_mod_names(&thunderstore);
            }
        }

        tokio::time::sleep(TIME_BETWEEN_LOADS).await;
    }
}

async fn fetch_mods(app: &AppHandle, game: &'static Game, write_directly: bool) -> Result<()> {
    const IGNORED_NAMES: [&str; 2] = ["r2modman", "GaleModManager"];
    const UPDATE_INTERVAL: Duration = Duration::from_millis(500);

    let state = app.state::<Mutex<Thunderstore>>();
    let client = &app.state::<NetworkClient>().0;

    state.lock().unwrap().is_fetching = true;

    let url = format!("https://thunderstore.io/c/{}/api/v1/package/", game.id);
    let mut response = client.get(url).send().await?.error_for_status()?;

    let mut is_first_chunk = true;
    let mut buffer = String::new();
    let mut byte_buffer = Vec::new();
    let mut package_buffer = match write_directly {
        true => None,
        false => Some(IndexMap::new()),
    };

    let start_time = Instant::now();
    let mut last_update = Instant::now();

    while let Some(chunk) = response.chunk().await? {
        byte_buffer.extend_from_slice(&chunk);
        let chunk = match str::from_utf8(&byte_buffer) {
            Ok(chunk) => chunk,
            Err(_) => continue,
        };

        if is_first_chunk {
            is_first_chunk = false;
            buffer.extend(chunk.chars().skip(1)); // remove leading [
        } else {
            buffer.push_str(chunk);
        }

        byte_buffer.clear();

        {
            let mut state = state.lock().unwrap();

            let map = match package_buffer {
                Some(ref mut map) => map,
                None => &mut state.packages,
            };

            while let Some(index) = buffer.find("}]},") {
                let (json, _) = buffer.split_at(index + 3);

                match serde_json::from_str::<PackageListing>(json) {
                    Ok(package) => {
                        if !IGNORED_NAMES.contains(&package.name.as_str()) {
                            map.insert(package.uuid4, package);
                        }
                    }
                    Err(err) => logger::log_js_err("failed to fetch mod", &anyhow!(err), app),
                }

                buffer.replace_range(..index + 4, "");
            }

            if last_update.elapsed() > UPDATE_INTERVAL {
                app.emit(
                    "status_update",
                    Some(format!("Fetching mods from Thunderstore... {}", map.len())),
                )
                .ok();

                last_update = Instant::now();
            }
        }
    }

    let mut state = state.lock().unwrap();
    if let Some(package_buffer) = package_buffer {
        state.packages = package_buffer;
    }

    state.packages_fetched = true;
    state.is_fetching = false;

    debug!(
        "loaded {} mods in {:?}",
        state.packages.len(),
        start_time.elapsed()
    );

    app.emit("status_update", None::<String>).ok();

    Ok(())
}

fn read_cache(path: &Path) -> Result<Option<Vec<PackageListing>>> {
    let start = Instant::now();

    if !path.exists() {
        debug!("no cache file found at {}", path.display());
        return Ok(None);
    }

    let result: Vec<PackageListing> =
        util::fs::read_json(path).context("failed to deserialize cache")?;

    debug!(
        "Read {} mods from cache in {:?}",
        result.len(),
        start.elapsed()
    );

    Ok(Some(result))
}

pub fn write_cache(packages: &[&PackageListing], path: &Path) -> Result<()> {
    if packages.is_empty() {
        return Ok(());
    }

    let start = Instant::now();

    util::fs::write_json(path, packages, JsonStyle::Compact)
        .context("failed to write mod cache")?;

    debug!(
        "wrote {} mods to cache in {:?}",
        packages.len(),
        start.elapsed()
    );

    Ok(())
}

pub fn cache_path(manager: &ModManager) -> PathBuf {
    manager.active_game().path.join("thunderstore_cache.json")
}
