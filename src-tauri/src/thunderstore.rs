use std::{
    collections::HashSet, iter, str::{self, Split}, sync::Mutex, time::{Duration, Instant}
};

use anyhow::{anyhow, Context, Result};
use indexmap::IndexMap;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tauri::{async_runtime::JoinHandle, AppHandle, Manager};
use uuid::Uuid;

use crate::{
    games::Game, manager::ModManager, util, NetworkClient
};

use self::{
    models::{PackageListing, PackageVersion},
    query::Queryable,
};

pub mod commands;
pub mod models;
pub mod query;

pub fn setup(app: &AppHandle) -> Result<()> {
    let manager = app.state::<Mutex<ModManager>>();
    let manager = manager.lock().unwrap();

    let mut thunderstore = Thunderstore::new();
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

#[derive(Debug, Clone, Serialize)]
pub struct BorrowedMod<'a> {
    pub package: &'a PackageListing,
    pub version: &'a PackageVersion,
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

impl From<&BorrowedMod<'_>> for ModRef {
    fn from(borrowed_mod: &BorrowedMod<'_>) -> Self {
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

pub struct Thunderstore {
    pub load_mods_handle: Option<JoinHandle<()>>,
    pub finished_loading: bool,
    // IndexMap is not used for ordering here, but for fast iteration,
    // since we iterate over all mods when querying and resolving dependencies
    pub packages: IndexMap<Uuid, PackageListing>,
}

impl Thunderstore {
    pub fn new() -> Self {
        Self {
            load_mods_handle: None,
            finished_loading: false,
            packages: IndexMap::new(),
        }
    }

    pub fn switch_game(&mut self, game: &'static Game, app: AppHandle) {
        if let Some(handle) = self.load_mods_handle.take() {
            handle.abort();
        }

        self.finished_loading = false;
        self.packages.clear();

        let load_mods_handle = tauri::async_runtime::spawn(load_mods_loop(app, game));

        self.load_mods_handle = Some(load_mods_handle);
    }

    pub fn queryable(&self) -> impl Iterator<Item = Queryable<'_>> {
        self.packages.values().map(move |package| {
            Queryable::Online(BorrowedMod {
                package,
                version: &package.versions[0],
            })
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
        let split = identifier.split(delimeter);

        let (author, name, version) = parts(split)
            .with_context(|| format!("invalid dependency string format {}", identifier))?;

        let full_name = format!("{}-{}", author, name);
        let version = semver::Version::parse(version)
            .with_context(|| format!("invalid version format {}", version))?;

        let package = self.find_package(&full_name)?;
        let version = package
            .get_version_with_num(&version)
            .with_context(|| format!("version {} not found in package {}", version, full_name))?;

        return Ok((package, version).into());

        fn parts(mut s: Split<'_, char>) -> Option<(&str, &str, &str)> {
            let author = s.next()?;
            let name = s.next()?;
            let version = s.next()?;

            Some((author, name, version))
        }
    }

    pub fn resolve_deps<'a>(
        &'a self,
        dependency_strings: &'a [String],
    ) -> impl Iterator<Item = Result<BorrowedMod<'a>>> + 'a {
        let mut unique_map = HashSet::new();
        return inner(self, dependency_strings)
            .filter_ok(move |dep| unique_map.insert(dep.package.uuid4));

        fn inner<'a>(
            this: &'a Thunderstore,
            dependency_strings: &'a [String],
        ) -> Box<dyn Iterator<Item = Result<BorrowedMod<'a>>> + 'a> {
            Box::new(
                dependency_strings
                    .iter()
                    .map(move |dependency| {
                        let dep = this.find_mod(dependency, '-');

                        dep.map(|dep| {
                            inner(this, &dep.version.dependencies).chain(iter::once(Ok(dep)))
                        })
                    })
                    .flatten_ok()
                    .flatten_ok(),
            )
        }
    }

    pub fn dependencies<'a>(
        &'a self,
        version: &'a PackageVersion,
    ) -> impl Iterator<Item = Result<BorrowedMod<'a>>> {
        self.resolve_deps(&version.dependencies)
    }
}

const TIME_BETWEEN_LOADS: Duration = Duration::from_secs(60 * 10);

async fn load_mods_loop(app: AppHandle, game: &'static Game) {
    let mut is_first = true;
    loop {
        if let Err(err) = load_mods(&app, game, is_first).await {
            util::print_err("error while loading mods from Thunderstore", &err, &app);
        }

        is_first = false;
        tokio::time::sleep(TIME_BETWEEN_LOADS).await;
    }
}

const IGNORED_NAMES: [&str; 2] = ["r2modman", "GaleModManager"];

async fn load_mods(app: &AppHandle, game: &'static Game, write_directly: bool) -> Result<()> {
    let state = app.state::<Mutex<Thunderstore>>();
    let client = &app.state::<NetworkClient>().0;

    let mut response = client.get(&game.url)
        .send().await?
        .error_for_status()?;

    let mut is_first_chunk = true;
    let mut buffer = String::new();
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
                buffer.extend(chunk.chars().skip(1)); // remove leading [
            }
            false => buffer.push_str(chunk),
        };

        byte_buffer.clear();

        {
            let mut state = state.lock().unwrap();

            let map = match package_buffer {
                Some(ref mut map) => map,
                None => &mut state.packages,
            };

            while let Some(index) = buffer.find("}]},") {
                let (json, _) = buffer.split_at(index + 3);

                let package = serde_json::from_str::<PackageListing>(json);

                match package {
                    Ok(package) => {
                        if !IGNORED_NAMES.contains(&package.name.as_str()) {
                            map.insert(package.uuid4, package);
                        }
                    }
                    Err(err) => {
                        println!("{}", json);
                        util::print_err("Failed to load mod", &anyhow!(err), app)
                    }
                }

                buffer.replace_range(..index + 4, "");
            }

            if i % 200 == 0 && start_time.elapsed().as_secs() > 1 {
                let _ = app.emit_all(
                    "status_update",
                    Some(format!(
                        "Fetching mods from Thunderstore... {}",
                        map.len()
                    )),
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

    println!(
        "loaded {} mods in {:?}",
        state.packages.len(),
        start_time.elapsed()
    );

    let _ = app.emit_all("status_update", None::<String>);
    Ok(())
}
