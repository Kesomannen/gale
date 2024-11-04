use std::{
    collections::{HashSet, VecDeque},
    iter::FusedIterator,
    path::PathBuf,
    str::{self},
    sync::Mutex,
    time::Instant,
};

use anyhow::{Context, Result};
use indexmap::IndexMap;
use log::debug;
use serde::{Deserialize, Serialize};
use tauri::{async_runtime::JoinHandle, AppHandle, Manager};
use uuid::Uuid;

use crate::{
    game::Game,
    profile::ModManager,
    util::{self, fs::JsonStyle},
};

pub mod commands;
pub mod query;
pub mod token;

mod fetch;

mod models;
pub use models::*;

mod ident;
pub use ident::*;

pub fn setup(app: &AppHandle) {
    let manager = app.state::<Mutex<ModManager>>();
    let manager = manager.lock().unwrap();

    let mut thunderstore = Thunderstore::default();
    thunderstore.switch_game(manager.active_game, app.clone());

    app.manage(Mutex::new(thunderstore));

    query::setup(app);
}

/// A pair of a package and one of its versions.
///
/// This is tied to the lifetime of the `Thunderstore` struct and thus
/// can only be held when its Mutex is locked. To avoid that limitation,
/// use [`ModId`] instead.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, Hash)]
pub struct BorrowedMod<'a> {
    pub package: &'a PackageListing,
    pub version: &'a PackageVersion,
}

impl<'a> BorrowedMod<'a> {
    pub fn ident(&self) -> &'a VersionIdent {
        &self.version.ident
    }

    pub fn owner(&self) -> &'a str {
        self.ident().owner()
    }

    pub fn name(&self) -> &'a str {
        self.ident().name()
    }

    pub fn version(&self) -> &'a str {
        self.ident().version()
    }

    pub fn split(self) -> (&'a PackageListing, &'a PackageVersion) {
        (self.package, self.version)
    }

    pub fn deps(&self) -> impl Iterator<Item = &'a VersionIdent> + 'a {
        self.version.dependencies.iter()
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

/// A pair of a package uuid and the uuid of one of its versions.
///
/// This is a "persistent" version of [`BorrowedMod`] which can be held
/// without locking [`Thunderstore`] as well as (de)serialized.
///
/// To convert it back into a [`BorrowedMod`], use [`ModId::borrow`].
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModId {
    #[serde(alias = "package")]
    pub package_uuid: Uuid,
    #[serde(alias = "version")]
    pub version_uuid: Uuid,
}

impl From<BorrowedMod<'_>> for ModId {
    fn from(borrowed: BorrowedMod<'_>) -> Self {
        Self {
            package_uuid: borrowed.package.uuid,
            version_uuid: borrowed.version.uuid,
        }
    }
}

impl ModId {
    /// Borrows the mod from [`Thunderstore`].
    pub fn borrow<'a>(&self, thunderstore: &'a Thunderstore) -> Result<BorrowedMod<'a>> {
        thunderstore.get_mod(self.package_uuid, self.version_uuid)
    }
}

/// Registry of Thunderstore mods for the active game.
#[derive(Default)]
pub struct Thunderstore {
    /// A handle to the current [`fetch_package_loop`] task.
    fetch_loop_handle: Option<JoinHandle<()>>,
    /// Whether packages have been succesfully fetched at least one since
    /// the last call to [`Thunderstore::switch_game`].
    packages_fetched: bool,
    /// Whether a [`fetch_mods`] task i currently running.
    is_fetching: bool,
    // IndexMap is not used for ordering here, but for fast iteration,
    // since we iterate over all mods when resolving identifiers and querying.
    packages: IndexMap<Uuid, PackageListing>,
}

impl Thunderstore {
    /// Whether packages have been succesfully fetched at least one since
    /// the last call to [`Thunderstore::switch_game`].
    pub fn packages_fetched(&self) -> bool {
        self.packages_fetched
    }

    /// Returns an iterator over the lastest versions of every package.
    pub fn latest(&self) -> impl Iterator<Item = BorrowedMod<'_>> {
        self.packages.values().map(move |package| BorrowedMod {
            package,
            version: package.latest(),
        })
    }

    pub fn get_package(&self, uuid: Uuid) -> Result<&PackageListing> {
        self.packages
            .get(&uuid)
            .with_context(|| format!("package with id {} not found", uuid))
    }

    /// Finds a package with the given `full_name` (formatted as `owner-name`).
    pub fn find_package<'a>(&'a self, full_name: &str) -> Result<&'a PackageListing> {
        self.packages
            .values()
            .find(|package| package.ident.as_str() == full_name)
            .with_context(|| format!("package {} not found", full_name))
    }

    pub fn get_mod(&self, package_uuid: Uuid, version_uuid: Uuid) -> Result<BorrowedMod<'_>> {
        let package = self.get_package(package_uuid)?;
        let version = package.get_version(version_uuid).with_context(|| {
            format!(
                "version with id {} not found in package {}",
                version_uuid, package.ident
            )
        })?;

        Ok((package, version).into())
    }

    pub fn find_ident<'a>(&'a self, ident: &VersionIdent) -> Result<BorrowedMod<'a>> {
        self.find_mod(ident.owner(), ident.name(), ident.version())
    }

    pub fn find_mod<'a>(
        &'a self,
        owner: &str,
        name: &str,
        version: &str,
    ) -> Result<BorrowedMod<'a>> {
        let package = self
            .packages
            .values()
            .find(|package| package.owner() == owner && package.name() == name)
            .with_context(|| format!("package {}-{} not found", owner, name))?;

        let version = package.get_version_with_num(version).with_context(|| {
            format!(
                "version {} not found in package {}-{}",
                version, owner, name
            )
        })?;

        Ok((package, version).into())
    }

    /// Switches the active game, clearing the package map and aborting ongoing fetch tasks.
    pub fn switch_game(&mut self, game: Game, app: AppHandle) {
        if let Some(handle) = self.fetch_loop_handle.take() {
            handle.abort();
        }

        self.is_fetching = false;
        self.packages_fetched = false;
        self.packages = IndexMap::new();

        let load_mods_handle = tauri::async_runtime::spawn(fetch::fetch_package_loop(app, game));
        self.fetch_loop_handle = Some(load_mods_handle);
    }
}

/// See [`Thunderstore::dependencies`].
pub struct Dependencies<'a> {
    queue: VecDeque<&'a VersionIdent>,
    visited: HashSet<&'a str>,
    thunderstore: &'a Thunderstore,
}

impl<'a> Iterator for Dependencies<'a> {
    type Item = BorrowedMod<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let current = self.queue.pop_front()?;
            let Ok(current) = self.thunderstore.find_ident(&current) else {
                continue;
            };

            for dependency in &current.version.dependencies {
                if !self.visited.insert(dependency.full_name()) {
                    continue;
                }

                self.queue.push_back(dependency);
            }

            break Some(current);
        }
    }
}

impl FusedIterator for Dependencies<'_> {}

impl Thunderstore {
    /// Recursively finds the dependencies of the given mods,
    /// sorted by ascending depth.
    ///
    /// Duplicates of the same package are removed. The specific
    /// version of a package that is chosen depends on which
    /// is encountered first.
    pub fn dependencies<'a>(
        &'a self,
        idents: impl IntoIterator<Item = &'a VersionIdent>,
    ) -> Dependencies<'a> {
        Dependencies {
            queue: idents.into_iter().collect(),
            visited: HashSet::new(),
            thunderstore: self,
        }
    }
}

pub fn read_cache(manager: &ModManager) -> Result<Option<Vec<PackageListing>>> {
    let start = Instant::now();

    let path = cache_path(manager);

    if !path.exists() {
        debug!("no cache file found at {}", path.display());
        return Ok(None);
    }

    let result: Vec<PackageListing> =
        util::fs::read_json(path).context("failed to deserialize cache")?;

    debug!(
        "read {} mods from cache in {:?}",
        result.len(),
        start.elapsed()
    );

    Ok(Some(result))
}

pub fn write_cache(packages: &[&PackageListing], manager: &ModManager) -> Result<()> {
    if packages.is_empty() {
        return Ok(());
    }

    let start = Instant::now();

    util::fs::write_json(cache_path(manager), packages, JsonStyle::Compact)
        .context("failed to write mod cache")?;

    debug!(
        "wrote {} mods to cache in {:?}",
        packages.len(),
        start.elapsed()
    );

    Ok(())
}

fn cache_path(manager: &ModManager) -> PathBuf {
    manager.active_game().path().join("thunderstore_cache.json")
}
