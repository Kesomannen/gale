use crate::{
    game::Game,
    prefs::{Backends, Prefs},
    thunderstore::{BorrowedMod, PackageListing, VersionIdent, cache::MarkdownKind},
};
use eyre::eyre;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use uuid::Uuid;
use crate::thunderstore::PackageIdent;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Backend {
    #[default]
    Thunderstore,
    Hexium,
}

impl Display for Backend {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Backend::Thunderstore => "thunderstore",
            Backend::Hexium => "hexium",
        })
    }
}

impl Backend {
    pub fn index_url(self, game: Game) -> Option<String> {
        match self {
            Backend::Thunderstore => Some(format!(
                "https://thunderstore.io/c/{}/api/v1/package-listing-index/",
                game.slug
            )),
            Backend::Hexium => {
                if game.slug == "valheim" {
                    Some(
                        "https://mods.valtools.org/c/valheim/api/v1/package-listing-index/"
                            .to_string(),
                    )
                } else {
                    None
                }
            }
        }
    }

    pub fn markdown_url(self, ident: &VersionIdent, cache: MarkdownKind) -> String {
        match self {
            Backend::Thunderstore => format!(
                "https://thunderstore.io/api/experimental/package/{}/{}/{}/{}/",
                ident.owner(),
                ident.name(),
                ident.version(),
                cache
            ),
            Backend::Hexium => format!(
                "https://mods.valtools.org/api/experimental/package/{}/{}/{}/{}/",
                ident.owner(),
                ident.name(),
                ident.version(),
                cache
            ),
        }
    }

    pub fn owner_url(self, owner: &str, game: Game) -> String {
        match self {
            Backend::Thunderstore => {
                format!("https://thunderstore.io/c/{}/p/{}/", game.slug, owner)
            }
            Backend::Hexium => format!("https://mods.valtools.org/teams/{}", owner),
        }
    }

    pub fn mod_url(self, package: &PackageIdent, game: Game) -> String {
        match self {
            Backend::Thunderstore => {
                format!(
                    "https://thunderstore.io/c/{}/p/{}/{}/",
                    game.slug, package.name(), package.name()
                )
            }
            Backend::Hexium => format!(
                "https://mods.valtools.org/mods/1/{}/{}",
                package.name(), package.name()
            ),
        }
    }

    pub fn download_url(self, version: &VersionIdent) -> String {
        match self {
            Backend::Thunderstore => format!(
                "https://thunderstore.io/package/download/{}/{}/{}",
                version.owner(), version.name(), version.version()
            ),
            Backend::Hexium => format!(
                "https://mods.valtools.org/uploads/{}/{}/{}.zip",
                version.owner(), version.name(), version.version()
            ),
        }
    }

    pub fn profile_import(self, key: &str) -> String {
        match self {
            Backend::Thunderstore => {
                format!("https://thunderstore.io/api/experimental/legacyprofile/get/{key}/")
            }
            Backend::Hexium => {
                format!("https://mods.valtools.org/api/experimental/legacyprofile/get/{key}/")
            }
        }
    }

    pub fn profile_export(self) -> &'static str {
        match self {
            Backend::Thunderstore => {
                "https://thunderstore.io/api/experimental/legacyprofile/create/"
            }
            Backend::Hexium => "https://mods.valtools.org/api/experimental/legacyprofile/create/",
        }
    }

    pub fn modpack_upload_baseurl(self) -> &'static str {
        match self {
            Backend::Thunderstore => "https://thunderstore.io/api/experimental",
            Backend::Hexium => "https://mods.valtools.org/api/experimental",
        }
    }

    pub fn apply_all<R>(f: impl Fn(Self) -> R, backends: Backends) -> Vec<R> {
        match backends {
            Backends::All => vec![f(Backend::Thunderstore), f(Backend::Hexium)],
            Backends::Thunderstore => vec![f(Backend::Thunderstore)],
            Backends::Hexium => vec![f(Backend::Hexium)],
        }
    }
}

/// Registry for all Thunderstore-like mods for the active game (Hexium, Thunderstore).
pub struct ThunderstoreBackend {
    /// Whether packages have been succesfully fetched at least one since
    /// the last call to [`crate::thunderstore::Thunderstore::switch_game`].
    pub(super) packages_fetched: bool,
    /// Whether a [`fetch_mods`] task is currently running.
    is_fetching: bool,
    // IndexMap is not used for ordering here, but for fast iteration,
    // since we iterate over all mods when resolving identifiers and querying.
    pub(super) packages: IndexMap<Uuid, PackageListing>,
    backend: Backend,
}

impl ThunderstoreBackend {
    pub fn new(backend: Backend) -> Self {
        Self {
            packages_fetched: false,
            is_fetching: false,
            packages: IndexMap::new(),
            backend,
        }
    }

    /// Whether packages have been succesfully fetched at least one since
    /// the last call to [`crate::thunderstore::Thunderstore::switch_game`].
    pub fn packages_fetched(&self) -> bool {
        self.packages_fetched
    }

    /// Returns an iterator over the latest versions of every package.
    pub fn latest(&self) -> impl Iterator<Item = BorrowedMod<'_>> {
        self.packages.values().map(move |package| BorrowedMod {
            package,
            version: package.latest(),
        })
    }

    pub fn get_package(&self, uuid: Uuid) -> eyre::Result<&PackageListing> {
        self.packages
            .get(&uuid)
            .ok_or_else(|| eyre!("package with id {uuid} not found",))
    }

    /// Finds a package with the given `full_name` (formatted as `owner-name`).
    pub fn find_package(&self, full_name: &str) -> eyre::Result<&PackageListing> {
        self.packages
            .values()
            .find(|package| package.ident.as_str() == full_name)
            .ok_or_else(|| eyre!("package {full_name} not found",))
    }

    pub fn get_mod(&self, package_uuid: Uuid, version_uuid: Uuid) -> eyre::Result<BorrowedMod<'_>> {
        let package = self.get_package(package_uuid)?;
        let version = package.get_version(version_uuid).ok_or_else(|| {
            eyre!(
                "version with id {version_uuid} not found in package {}",
                package.ident
            )
        })?;

        Ok((package, version).into())
    }

    pub fn find_mod<'a>(
        &'a self,
        owner: &str,
        name: &str,
        version: &str,
    ) -> eyre::Result<BorrowedMod<'a>> {
        let package = self
            .packages
            .values()
            .find(|package| package.owner() == owner && package.name() == name)
            .ok_or_else(|| eyre!("package {}-{} not found", owner, name))?;

        let version = package.get_version_with_num(version).ok_or_else(|| {
            eyre!(
                "version {} not found in package {}-{}",
                version,
                owner,
                name
            )
        })?;

        Ok((package, version).into())
    }

    /// Clear the package map.
    pub fn clear_packages(&mut self, game: Game, prefs: &Prefs) {
        self.is_fetching = false;
        self.packages_fetched = false;
        self.packages = IndexMap::new();

        self.read_and_insert_cache(game, prefs, self.backend);
    }
}
