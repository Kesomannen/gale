use eyre::Result;
use query::QueryModsArgs;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::{HashSet, VecDeque},
    hash::Hash,
    iter::FusedIterator,
    str::{self},
};
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;
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
            version: package.latest_released(),
        }
    }

    pub fn ident(&self) -> &'a VersionIdent {
        &self.version.ident
    }

    pub fn dependencies(&self) -> impl Iterator<Item = (&'a VersionIdent, Backend)> + 'a + use<'a> {
        self.version
            .dependencies
            .iter()
            .map(|ident| (ident, self.package.backend))
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
        thunderstore.get_mod(
            self.package_uuid,
            self.version_uuid,
            FromBackend::Only(self.backend),
        )
    }
}

#[derive(Debug, Serialize)]
pub struct DeduplicatedMod<T> {
    pub thunderstore: Option<T>,
    pub hexium: Option<T>,
}

impl<T> DeduplicatedMod<T> {
    fn set(&mut self, backend: Backend, value: T) {
        match backend {
            Backend::Thunderstore => self.thunderstore = Some(value),
            Backend::Hexium => self.hexium = Some(value),
        }
    }

    pub fn map<U>(self, mut f: impl FnMut(T) -> U) -> DeduplicatedMod<U> {
        DeduplicatedMod {
            thunderstore: self.thunderstore.map(&mut f),
            hexium: self.hexium.map(f),
        }
    }
}

impl<T> Default for DeduplicatedMod<T> {
    fn default() -> Self {
        Self {
            thunderstore: None,
            hexium: None,
        }
    }
}

/// Registry of Thunderstore mods for the active game.
pub struct Thunderstore {
    game: Option<Game>,
    fetch_cancel_token: CancellationToken,
    /// Whether a [`fetch_mods`] task is currently running.
    is_fetching: bool,
    current_query: Option<QueryModsArgs>,
    thunderstore_backend: ThunderstoreBackend,
    hexium_backend: ThunderstoreBackend,
}

/// Specifies which backend to use when searching for a mod using a uuid or identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FromBackend {
    /// Use any backend that has the mod with no preference. This currently prefers Thunderstore,
    /// but that may change at any time.
    Any,
    /// Always prefer the given backend, but fall back to the other if the mod is not found.
    Prefer(Backend),
    /// Check both backends for the mod. If both backends have the mod, prefer the one with the higher version
    /// and deprecation status. If only one backend has the mod, use that one.
    PreferIfEqual(Backend),
    /// Only use the given backend. If the mod is not found there, return an error.
    Only(Backend),
}

impl From<Backend> for FromBackend {
    fn from(backend: Backend) -> Self {
        Self::Only(backend)
    }
}

impl From<Option<Backend>> for FromBackend {
    fn from(option: Option<Backend>) -> Self {
        match option {
            Some(backend) => Self::Only(backend),
            None => Self::Any,
        }
    }
}

impl Thunderstore {
    pub fn new() -> Self {
        Self {
            game: None,
            fetch_cancel_token: CancellationToken::new(),
            is_fetching: false,
            current_query: None,
            thunderstore_backend: ThunderstoreBackend::new(Backend::Thunderstore),
            hexium_backend: ThunderstoreBackend::new(Backend::Hexium),
        }
    }

    /// Whether packages have been succesfully fetched at least one since
    /// the last call to [`Thunderstore::switch_game`].
    pub fn packages_fetched(&self, app: &AppHandle, game: Game) -> bool {
        let backends = app.lock_prefs().enabled_backends(game);
        backends
            .iter()
            .all(|backend| self.backend(backend).packages_fetched())
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
        from: FromBackend,
    ) -> Result<R> {
        let (preferred, fallback) = match from {
            FromBackend::Prefer(Backend::Hexium) | FromBackend::PreferIfEqual(Backend::Hexium) => {
                (&self.hexium_backend, &self.thunderstore_backend)
            }
            FromBackend::Any
            | FromBackend::Prefer(Backend::Thunderstore)
            | FromBackend::PreferIfEqual(Backend::Thunderstore) => {
                (&self.thunderstore_backend, &self.hexium_backend)
            }
            FromBackend::Only(backend) => return f(self.backend(backend)),
        };

        match (f(preferred), f(fallback)) {
            (Ok(preferred), Ok(fallback)) if matches!(from, FromBackend::PreferIfEqual(_)) => {
                Ok(cmp(preferred, fallback))
            }
            (Ok(preferred), Ok(_)) => Ok(preferred),
            (Ok(preferred), Err(_)) => Ok(preferred),
            (Err(_), Ok(fallback)) => Ok(fallback),
            (Err(e), Err(_)) => Err(e),
        }
    }

    fn cmp_package_listing<'a>(
        preferred: &'a PackageListing,
        fallback: &'a PackageListing,
    ) -> &'a PackageListing {
        match Self::cmp_packages(
            preferred.is_deprecated,
            Some(&preferred.latest().parsed_version()),
            fallback.is_deprecated,
            Some(&fallback.latest().parsed_version()),
        ) {
            Ordering::Less => fallback,
            _ => preferred,
        }
    }

    fn cmp_queryable<T: Queryable>(preferred: T, fallback: T) -> T {
        match Self::cmp_packages(
            preferred.is_deprecated(),
            preferred.version().as_ref(),
            fallback.is_deprecated(),
            fallback.version().as_ref(),
        ) {
            Ordering::Less => fallback,
            _ => preferred,
        }
    }

    fn cmp_packages(
        a_deprecated: bool,
        a_version: Option<&semver::Version>,
        b_deprecated: bool,
        b_version: Option<&semver::Version>,
    ) -> Ordering {
        a_deprecated
            .cmp(&b_deprecated)
            .reverse()
            .then_with(|| a_version.cmp(&b_version))
    }

    pub fn get_package(&self, uuid: Uuid, from: impl Into<FromBackend>) -> Result<&PackageListing> {
        self.resolve_thunderstore_vs_hexium(
            |b| b.get_package(uuid),
            Self::cmp_package_listing,
            from.into(),
        )
    }

    /// Finds a package with the given `full_name` (formatted as `owner-name`).
    pub fn find_package(
        &self,
        full_name: &str,
        from: impl Into<FromBackend>,
    ) -> Result<&PackageListing> {
        self.resolve_thunderstore_vs_hexium(
            |b| b.find_package(full_name),
            Self::cmp_package_listing,
            from.into(),
        )
    }

    pub fn get_mod(
        &self,
        package_uuid: Uuid,
        version_uuid: Uuid,
        from: impl Into<FromBackend>,
    ) -> Result<BorrowedMod<'_>> {
        self.resolve_thunderstore_vs_hexium(
            |b| b.get_mod(package_uuid, version_uuid),
            Self::cmp_queryable,
            from.into(),
        )
    }

    pub fn find_ident(
        &self,
        ident: &VersionIdent,
        from: impl Into<FromBackend>,
    ) -> Result<BorrowedMod<'_>> {
        self.find_mod(ident.owner(), ident.name(), ident.version(), from.into())
    }

    pub fn find_mod<'a>(
        &'a self,
        owner: &str,
        name: &str,
        version: &str,
        from: impl Into<FromBackend>,
    ) -> Result<BorrowedMod<'a>> {
        self.resolve_thunderstore_vs_hexium(
            |b| b.find_mod(owner, name, version),
            Self::cmp_queryable,
            from.into(),
        )
    }

    /// Switches the active game, clearing the package map and aborting ongoing fetch tasks.
    pub fn switch_game(&mut self, game: Game, app: AppHandle) {
        self.fetch_cancel_token.cancel();

        self.is_fetching = false;

        {
            self.thunderstore_backend.clear_packages();
            self.hexium_backend.clear_packages();

            let prefs = app.lock_prefs();

            for backend in prefs.enabled_backends(game).iter() {
                let backend = self.backend_mut(backend);
                backend.read_and_insert_cache(game, &prefs);
            }
        }

        self.game = Some(game);

        self.fetch_cancel_token = CancellationToken::new();

        tauri::async_runtime::spawn(fetch::fetch_package_loop(
            game,
            app,
            self.fetch_cancel_token.clone(),
        ));
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
    queue: VecDeque<(&'a VersionIdent, Backend)>,
    visited: HashSet<&'a str>,
    thunderstore: &'a Thunderstore,
}

impl<'a> Iterator for Dependencies<'a> {
    type Item = BorrowedMod<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (current_ident, current_backend) = self.queue.pop_front()?;
            let Ok(current) = self.thunderstore.find_ident(current_ident, current_backend) else {
                continue;
            };

            for dependency in &current.version.dependencies {
                if !self.visited.insert(dependency.full_name()) {
                    continue;
                }

                self.queue.push_back((dependency, current_backend));
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
        dependencies: impl IntoIterator<Item = (&'a VersionIdent, Backend)>,
    ) -> Dependencies<'a> {
        let queue = dependencies.into_iter().collect::<VecDeque<_>>();
        let mut visited = HashSet::with_capacity(queue.len());
        for (ident, _) in &queue {
            visited.insert(ident.full_name());
        }

        Dependencies {
            queue,
            visited,
            thunderstore: self,
        }
    }
}

async fn get_categories(
    backend: Backend,
    game: Game,
    app: &AppHandle,
) -> Result<Vec<PackageCategory>> {
    let url = backend.category_url(game);
    let response: CategoryResponse = app
        .http()
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(response.results)
}
