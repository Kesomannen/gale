use eyre::Result;
use itertools::Itertools;
use query::QueryModsArgs;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashSet, VecDeque},
    hash::Hash,
    iter::FusedIterator,
    str::{self},
};
use tauri::{AppHandle, async_runtime::JoinHandle};
use uuid::Uuid;

use crate::{game::Game, state::ManagerExt, thunderstore::query::Queryable};

pub mod cache;
pub mod commands;
pub mod query;
pub mod token;

mod fetch;
pub use fetch::wait_for_fetch;

mod models;
pub use models::*;

mod ident;
pub use ident::*;

mod backend;
pub use backend::Backend;
use backend::ThunderstoreBackend;

pub fn start(app: &AppHandle) {
    query::setup(app);
    app.lock_thunderstore()
        .switch_game(app.lock_manager().active_game, app.clone());
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
    pub fn latest(package: &'a PackageListing) -> Self {
        Self {
            package,
            version: package.latest(),
        }
    }

    pub fn ident(&self) -> &'a VersionIdent {
        &self.version.ident
    }

    pub fn dependencies(&self) -> impl Iterator<Item = &'a VersionIdent> + 'a + use<'a> {
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
#[derive(Serialize, Deserialize, Clone, Debug, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ModId {
    pub package_uuid: Uuid,
    pub version_uuid: Uuid,
    #[serde(default)]
    pub backend: Backend,
}

impl From<BorrowedMod<'_>> for ModId {
    fn from(borrowed: BorrowedMod<'_>) -> Self {
        Self {
            package_uuid: borrowed.package.uuid,
            version_uuid: borrowed.version.uuid,
            backend: borrowed.package.backend,
        }
    }
}

impl PartialEq for ModId {
    fn eq(&self, other: &Self) -> bool {
        self.version_uuid == other.version_uuid
    }
}

impl Hash for ModId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.version_uuid.hash(state);
    }
}

impl ModId {
    /// Borrows the mod from [`Thunderstore`].
    pub fn borrow<'a>(&self, thunderstore: &'a Thunderstore) -> Result<BorrowedMod<'a>> {
        thunderstore.get_mod(self.package_uuid, self.version_uuid)
    }
}

/// Registry of Thunderstore mods for the active game.
pub struct Thunderstore {
    /// A handle to the current [`fetch_package_loop`] task.
    fetch_loop_handle: Option<JoinHandle<()>>,
    /// Whether a [`fetch_mods`] task is currently running.
    is_fetching: bool,
    current_query: Option<QueryModsArgs>,
    thunderstore_backend: ThunderstoreBackend,
    hexium_backend: ThunderstoreBackend,
}

impl Thunderstore {
    pub fn new() -> Self {
        Self {
            fetch_loop_handle: None,
            is_fetching: false,
            current_query: None,
            thunderstore_backend: ThunderstoreBackend::new(Backend::Thunderstore),
            hexium_backend: ThunderstoreBackend::new(Backend::Hexium),
        }
    }

    /// Whether packages have been succesfully fetched at least one since
    /// the last call to [`Thunderstore::switch_game`].
    pub fn packages_fetched(&self, app: &AppHandle, game: Game) -> bool {
        let backends = app.lock_prefs().backends(game);
        backends
            .into_backend_slice()
            .iter()
            .all(|b| self.backend(*b).packages_fetched())
    }

    pub fn deduplicate<T: Queryable>(mods: impl Iterator<Item = T>) -> impl Iterator<Item = T> {
        mods.sorted_by(|a, b| a.full_name().cmp(&b.full_name()))
            .coalesce(|a, b| {
                if a.full_name() == b.full_name() {
                    Ok(Self::cmp_borrowed_mod(a, b))
                } else {
                    Err((a, b))
                }
            })
    }

    /// Returns an iterator over the latest versions of every package.
    /// Without deduplication, usable for filtering. Call [`Thunderstore::deduplicate`] afterwards.
    pub fn latest(&self) -> impl Iterator<Item = BorrowedMod<'_>> {
        self.thunderstore_backend
            .latest()
            .chain(self.hexium_backend.latest())
    }

    fn resolve_thunderstore_vs_hexium<'a, R: 'a>(
        &'a self,
        f: impl Fn(&'a ThunderstoreBackend) -> Result<R>,
        cmp: impl Fn(R, R) -> R,
    ) -> Result<R> {
        let thunderstore = f(&self.thunderstore_backend);
        let hexium = f(&self.hexium_backend);
        match (thunderstore, hexium) {
            (Ok(thunderstore), Ok(hexium)) => Ok(cmp(thunderstore, hexium)),
            (Ok(thunderstore), Err(_)) => Ok(thunderstore),
            (Err(_), Ok(hexium)) => Ok(hexium),
            (Err(e), Err(_)) => Err(e),
        }
    }

    fn cmp_package_listing<'a>(
        thunderstore: &'a PackageListing,
        hexium: &'a PackageListing,
    ) -> &'a PackageListing {
        if thunderstore.latest().version() >= hexium.latest().version() {
            thunderstore
        } else {
            hexium
        }
    }

    pub fn get_package(&self, uuid: Uuid) -> Result<&PackageListing> {
        self.resolve_thunderstore_vs_hexium(|b| b.get_package(uuid), Self::cmp_package_listing)
    }

    /// Finds a package with the given `full_name` (formatted as `owner-name`).
    pub fn find_package(&self, full_name: &str) -> Result<&PackageListing> {
        self.resolve_thunderstore_vs_hexium(
            |b| b.find_package(full_name),
            Self::cmp_package_listing,
        )
    }

    fn cmp_borrowed_mod<T: Queryable>(thunderstore: T, hexium: T) -> T {
        if thunderstore.version() >= hexium.version() {
            thunderstore
        } else {
            hexium
        }
    }

    pub fn get_mod(&self, package_uuid: Uuid, version_uuid: Uuid) -> Result<BorrowedMod<'_>> {
        self.resolve_thunderstore_vs_hexium(
            |b| b.get_mod(package_uuid, version_uuid),
            Self::cmp_borrowed_mod,
        )
    }

    pub fn find_ident(&self, ident: &VersionIdent) -> Result<BorrowedMod<'_>> {
        self.find_mod(ident.owner(), ident.name(), ident.version())
    }

    pub fn find_mod<'a>(
        &'a self,
        owner: &str,
        name: &str,
        version: &str,
    ) -> Result<BorrowedMod<'a>> {
        self.resolve_thunderstore_vs_hexium(
            |b| b.find_mod(owner, name, version),
            Self::cmp_borrowed_mod,
        )
    }

    /// Switches the active game, clearing the package map and aborting ongoing fetch tasks.
    pub fn switch_game(&mut self, game: Game, app: AppHandle) {
        if let Some(handle) = self.fetch_loop_handle.take() {
            handle.abort();
        }

        self.is_fetching = false;

        self.thunderstore_backend
            .clear_packages(game, &app.lock_prefs());
        self.hexium_backend.clear_packages(game, &app.lock_prefs());

        let load_mods_handle = tauri::async_runtime::spawn(fetch::fetch_package_loop(game, app));
        self.fetch_loop_handle = Some(load_mods_handle);
    }

    pub fn backend(&self, backend: Backend) -> &ThunderstoreBackend {
        match backend {
            Backend::Thunderstore => &self.thunderstore_backend,
            Backend::Hexium => &self.hexium_backend,
        }
    }

    pub fn backend_mut(&mut self, backend: Backend) -> &mut ThunderstoreBackend {
        match backend {
            Backend::Thunderstore => &mut self.thunderstore_backend,
            Backend::Hexium => &mut self.hexium_backend,
        }
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
            let Ok(current) = self.thunderstore.find_ident(current) else {
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
        let queue = idents.into_iter().collect::<VecDeque<_>>();
        let mut visited = HashSet::with_capacity(queue.len());
        for item in &queue {
            visited.insert(item.full_name());
        }

        Dependencies {
            queue,
            visited,
            thunderstore: self,
        }
    }
}
