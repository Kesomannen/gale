use std::{
    collections::HashSet,
    fs,
    io::{self, Write},
    path::PathBuf,
    str::{self, Split},
    sync::Mutex,
    time::{Duration, Instant},
};

use anyhow::{anyhow, Context, Result};
use indexmap::IndexMap;
use log::debug;
use serde::{Deserialize, Serialize};
use tauri::{async_runtime::JoinHandle, AppHandle, Manager};
use uuid::Uuid;

use crate::{games::Game, manager::ModManager, util, NetworkClient};

use self::models::{PackageListing, PackageVersion};

pub mod commands;
pub mod models;
pub mod query;

pub fn setup(app: &AppHandle) -> Result<()> {
    let manager = app.state::<Mutex<ModManager>>();
    let manager = manager.lock().unwrap();

    let mut thunderstore = Thunderstore::default();
    thunderstore.switch_game(manager.active_game, app.clone());

    app.manage(Mutex::new(thunderstore));

    query::setup(app).context("failed to initialize query")?;

    Ok(())
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

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
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

#[derive(Serialize, Deserialize, Clone)]
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
    pub finished_loading: bool,
    // IndexMap is not used for ordering here, but for fast iteration,
    // since we iterate over all mods when querying and resolving dependencies
    pub packages: IndexMap<Uuid, PackageListing>,
}

impl Thunderstore {
    pub fn switch_game(&mut self, game: &'static Game, app: AppHandle) {
        if let Some(handle) = self.load_mods_handle.take() {
            handle.abort();
        }

        self.finished_loading = false;
        self.packages.clear();

        let load_mods_handle = tauri::async_runtime::spawn(load_mods_loop(app, game));

        self.load_mods_handle = Some(load_mods_handle);
    }

    pub fn latest(&self) -> impl Iterator<Item = BorrowedMod<'_>> {
        self.packages.values().map(move |package| BorrowedMod {
            package,
            version: &package.versions[0],
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
    ) -> Result<HashSet<BorrowedMod<'a>>> {
        let mut result = HashSet::new();
        let mut stack = dependency_strings.map(String::as_str).collect::<Vec<_>>();
        let mut visited = stack
            .iter()
            .map(|s| parse_author_name(s))
            .collect::<HashSet<_>>();

        while let Some(id) = stack.pop() {
            let dependency = self.find_mod(id, '-')?;

            for dep in &dependency.version.dependencies {
                let (author, name) = parse_author_name(dep);

                if !visited.insert((author, name)) {
                    continue;
                }

                stack.push(dep.as_str());
            }

            result.insert(dependency);
        }

        return Ok(result);

        fn parse_author_name(s: &str) -> (&str, &str) {
            let mut split = s.split('-');
            (split.next().unwrap(), split.next().unwrap())
        }
    }

    pub fn dependencies<'a>(
        &'a self,
        version: &'a PackageVersion,
    ) -> Result<HashSet<BorrowedMod<'a>>> {
        self.resolve_deps(version.dependencies.iter())
    }
}

const TIME_BETWEEN_LOADS: Duration = Duration::from_secs(60 * 15);

fn load_from_cache(app: &AppHandle) -> Result<bool> {
    let start = Instant::now();

    let manager = app.state::<Mutex<ModManager>>();
    let manager = manager.lock().unwrap();

    let path = cache_path(&manager);

    if !path.exists() {
        debug!("No cache file found at {:?}", path);
        return Ok(false);
    }

    let file = fs::File::open(path).context("failed to open cache file")?;
    let reader = io::BufReader::new(file);

    let thunderstore = app.state::<Mutex<Thunderstore>>();
    let mut thunderstore = thunderstore.lock().unwrap();

    let packages: Vec<PackageListing> =
        serde_json::from_reader(reader).context("failed to deserialize cache")?;

    thunderstore.packages = packages
        .into_iter()
        .map(|package| (package.uuid4, package))
        .collect();

    debug!(
        "Loaded {} mods from cache in {:?}",
        thunderstore.packages.len(),
        start.elapsed()
    );

    Ok(true)
}

async fn load_mods_loop(app: AppHandle, game: &'static Game) {
    /* 
    match load_from_cache(&app) {
        Ok(true) => {
            tokio::time::sleep(TIME_BETWEEN_LOADS / 2).await;
        }
        Ok(false) => {}
        Err(err) => {
            util::print_err("error while loading mods from cache", &err, &app);
        }
    }
    */

    let mut is_first = true;
    loop {
        if let Err(err) = load_mods(&app, game, is_first).await {
            util::print_err("error while loading mods from Thunderstore", &err, &app);
        } else {
            is_first = false;
        }

        tokio::time::sleep(TIME_BETWEEN_LOADS).await;
    }
}

const IGNORED_NAMES: [&str; 2] = ["r2modman", "GaleModManager"];

async fn load_mods(app: &AppHandle, game: &'static Game, write_directly: bool) -> Result<()> {
    let state = app.state::<Mutex<Thunderstore>>();
    let client = &app.state::<NetworkClient>().0;

    let mut response = client.get(&game.url).send().await?.error_for_status()?;

    let mut is_first_chunk = true;
    let mut text_index = 0;
    let mut text = String::new();
    let mut byte_buffer = Vec::new();
    let mut package_buffer = match write_directly {
        true => None,
        false => Some(IndexMap::new()),
    };
    let mut i = 0;

    let start_time = Instant::now();

    while let Some(chunk) = response.chunk().await? {
        byte_buffer.extend_from_slice(&chunk);
        let chunk = str::from_utf8(&byte_buffer);

        if chunk.is_err() {
            continue;
        }

        let chunk = chunk.unwrap();
        match is_first_chunk {
            true => {
                is_first_chunk = false;
                text.extend(chunk.chars().skip(1)); // remove leading [
            }
            false => text.push_str(chunk),
        };

        byte_buffer.clear();

        {
            let mut state = state.lock().unwrap();

            let map = match package_buffer {
                Some(ref mut map) => map,
                None => &mut state.packages,
            };

            while let Some(index) = text[text_index..].find("}]},") {
                let (json, _) = text[text_index..].split_at(index + 3);

                match serde_json::from_str::<PackageListing>(json) {
                    Ok(package) => {
                        if !IGNORED_NAMES.contains(&package.name.as_str()) {
                            map.insert(package.uuid4, package);
                        }
                    }
                    Err(err) => {
                        debug!("{}", json);
                        util::print_err("failed to load mod", &anyhow!(err), app)
                    }
                }

                text_index += index + 4;
            }

            if i % 200 == 0 && start_time.elapsed().as_secs() > 1 {
                let _ = app.emit_all(
                    "status_update",
                    Some(format!("Fetching mods from Thunderstore... {}", map.len())),
                );
            }
        }

        i += 1;
    }

    let mut state = state.lock().unwrap();
    if let Some(package_buffer) = package_buffer {
        state.packages = package_buffer;
    }

    state.finished_loading = true;

    debug!(
        "Loaded {} mods in {:?}",
        state.packages.len(),
        start_time.elapsed()
    );

    let _ = app.emit_all("status_update", None::<String>);

    /* 
    let manager = app.state::<Mutex<ModManager>>();
    let manager = manager.lock().unwrap();
    
    let mut file = fs::File::create(cache_path(&manager)).context("failed to create cache file")?;
    file.write_all(b"[")?;
    file.write_all(text.as_bytes())?;
    */

    Ok(())
}

fn cache_path(manager: &ModManager) -> PathBuf {
    manager.active_game().path().join("thunderstore_cache.json")
}
