use std::{cmp::Ordering, collections::HashSet, time::Duration};

use eyre::Result;
use indexmap::IndexMap;
use internment::Intern;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tracing::info;
use uuid::Uuid;

use super::{
    Backend, BorrowedMod, DeduplicatedMod, Thunderstore,
    models::{FrontendMod, FrontendModKind, FrontendVersion},
};
use crate::{
    profile::{LocalMod, ModManager},
    state::ManagerExt,
    util,
};

pub fn setup(app: &AppHandle) {
    tauri::async_runtime::spawn(query_loop(app.clone()));
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum SortBy {
    Newest,
    Name,
    Author,
    LastUpdated,
    Downloads,
    Rating,
    InstallDate,
    Custom,
    DiskSpace,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QueryModsArgs {
    pub max_count: Option<usize>,
    pub search_term: Option<String>,
    pub include_categories: HashSet<Intern<String>>,
    pub exclude_categories: HashSet<Intern<String>>,
    pub include_nsfw: bool,
    pub include_deprecated: bool,
    pub include_disabled: bool,
    pub include_enabled: bool,
    pub sort_by: SortBy,
    pub sort_order: SortOrder,
}

pub async fn query_loop(app: AppHandle) -> Result<()> {
    const INTERVAL: Duration = Duration::from_millis(500);

    loop {
        {
            let mut thunderstore = app.lock_thunderstore();

            if let Some(args) = &thunderstore.current_query {
                let manager = app.lock_manager();

                let mods = thunderstore.query_mods(args, &manager).collect_vec();
                app.emit_buffered("mod_query_result", &mods);

                if thunderstore.packages_fetched(&app, manager.active_game) {
                    info!("all packages fetched, pausing query loop");
                    thunderstore.current_query = None;
                }
            }
        };

        tokio::time::sleep(INTERVAL).await;
    }
}

/// Abstracts logic needed for `query_mods`, allowing it to be reused
/// for both Thunderstore and profile querying.
pub trait Queryable {
    /// The package's UUID.
    ///
    /// This is not unique across backends, but should be unique for every package within a backend.
    fn uuid(&self) -> uuid::Uuid;

    /// The package's full name, including the author.
    fn full_name(&self) -> &str;

    /// The packages latest version.
    fn version(&self) -> Option<semver::Version>;

    /// Whether the package should be included in the given query.
    fn matches(&self, args: &QueryModsArgs) -> bool;

    /// Whether the package should rank higher than `other` in the given query.
    fn cmp(&self, other: &Self, args: &QueryModsArgs) -> Ordering;

    /// The backend the package belongs to.
    fn backend(&self) -> Backend;

    /// A longer description of the package.
    fn description(&self) -> Option<&str> {
        None
    }

    /// Whether the package is deprecated.
    fn is_deprecated(&self) -> bool {
        false
    }
}

impl Queryable for BorrowedMod<'_> {
    fn uuid(&self) -> uuid::Uuid {
        self.package.uuid
    }

    fn full_name(&self) -> &str {
        self.package.ident.as_str()
    }

    fn version(&self) -> Option<semver::Version> {
        Some(self.package.latest().parsed_version())
    }

    fn description(&self) -> Option<&str> {
        Some(&self.version.description)
    }

    fn matches(&self, args: &QueryModsArgs) -> bool {
        let pkg = self.package;

        if !args.include_nsfw && pkg.has_nsfw_content
            || !args.include_deprecated && pkg.is_deprecated
        {
            return false;
        }

        if !args.include_categories.is_empty()
            && args.include_categories.is_disjoint(&pkg.categories)
        {
            return false;
        }

        if !args.exclude_categories.is_empty()
            && !args.exclude_categories.is_disjoint(&pkg.categories)
        {
            return false;
        }

        true
    }

    fn cmp(&self, other: &Self, args: &QueryModsArgs) -> Ordering {
        let (a, b) = (self.package, other.package);

        b.is_pinned.cmp(&a.is_pinned).then_with(|| {
            let order = match args.sort_by {
                SortBy::Newest => a.date_created.cmp(&b.date_created),
                SortBy::Name => util::cmp_ignore_case(a.name(), b.name()),
                SortBy::Author => util::cmp_ignore_case(&a.ident, &b.ident),
                SortBy::LastUpdated => a.date_updated.cmp(&b.date_updated),
                SortBy::Downloads => a.total_downloads().cmp(&b.total_downloads()),
                SortBy::Rating => a.rating_score.cmp(&b.rating_score),
                SortBy::DiskSpace => self.version.file_size.cmp(&other.version.file_size),
                SortBy::InstallDate => Ordering::Equal,
                SortBy::Custom => Ordering::Equal,
            };

            match args.sort_order {
                SortOrder::Ascending => order,
                SortOrder::Descending => order.reverse(),
            }
        })
    }

    fn backend(&self) -> Backend {
        self.package.backend
    }
}

impl From<BorrowedMod<'_>> for FrontendMod {
    fn from(borrowed_mod: BorrowedMod<'_>) -> FrontendMod {
        let pkg = borrowed_mod.package;
        let vers = pkg.get_version(borrowed_mod.version.uuid).unwrap();
        FrontendMod {
            name: pkg.name().to_owned(),
            description: Some(vers.description.to_string()),
            version: Some(vers.parsed_version()),
            categories: Some(
                pkg.categories
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect(),
            ),
            author: Some(pkg.owner().to_owned()),
            rating: Some(pkg.rating_score),
            downloads: Some(pkg.total_downloads()),
            file_size: vers.file_size,
            website_url: if vers.website_url.is_empty() {
                None
            } else {
                Some(vers.website_url.to_string())
            },
            donate_url: pkg.donation_link.clone(),
            dependencies: Some(vers.dependencies.clone()),
            suggestions: Some(vers.suggestions.clone()),
            is_pinned: pkg.is_pinned,
            is_deprecated: pkg.is_deprecated,
            contains_nsfw: pkg.has_nsfw_content,
            uuid: pkg.uuid,
            version_uuid: vers.uuid,
            last_updated: Some(pkg.versions[0].date_created.to_rfc3339()),
            versions: pkg
                .versions
                .iter()
                .map(|v| FrontendVersion {
                    name: v.parsed_version(),
                    uuid: v.uuid,
                })
                .collect(),
            kind: FrontendModKind::Remote,
            icon: None,
            backend: pkg.backend,
        }
    }
}

impl From<LocalMod> for FrontendMod {
    fn from(local_mod: LocalMod) -> FrontendMod {
        let LocalMod {
            name,
            description,
            version,
            file_size,
            uuid,
            dependencies,
            icon,
            ..
        } = local_mod;

        FrontendMod {
            name,
            description,
            version,
            file_size,
            uuid,
            dependencies,
            icon,
            kind: FrontendModKind::Local,
            ..Default::default()
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModListQueryItem {
    pub is_installed: bool,
    pub data: DeduplicatedMod<FrontendMod>,
}

impl Thunderstore {
    /// Sorts and filters `mods` according to `args` and converts the
    /// results to [`FrontendMod`].
    pub(super) fn query_mods(
        &self,
        args: &QueryModsArgs,
        manager: &ModManager,
    ) -> impl Iterator<Item = ModListQueryItem> {
        let included_mods = self
            .latest()
            .filter(|borrowed| !manager.hidden_mods.contains(&borrowed.package.uuid));

        let results = query_mods(args, included_mods);

        let profile = manager.active_profile();

        deduplicate_results(results, args.max_count.unwrap_or(usize::MAX)).map(|deduplicated| {
            let deduplicated = deduplicated.map(|backend_mod| FrontendMod::from(backend_mod));
            let is_installed = deduplicated
                .first()
                .map_or(false, |m| profile.has_mod(m.uuid));

            ModListQueryItem {
                is_installed,
                data: deduplicated,
            }
        })
    }
}

/// Sorts and filters `mods` according to `args`.
/// Does **not** limit the number of results according to `args.max_count`.
/// Limiting is instead left up to the caller to handle.
pub fn query_mods<'a, T, I>(args: &QueryModsArgs, mods: I) -> Vec<T>
where
    T: Queryable + 'a,
    I: Iterator<Item = T> + 'a,
{
    let search_terms = args.search_term.as_ref().map(|str| {
        let description_query = str.to_lowercase().trim().to_owned();
        let package_query = description_query.replace(' ', "_");
        // search for packages with underscores and descriptions with spaces
        (description_query, package_query)
    });

    let mut results = mods
        .filter(|queryable| {
            if let Some((description_query, package_query)) = &search_terms {
                let name_match = queryable.full_name().to_lowercase().contains(package_query);

                let description_match = queryable.description().is_some_and(|description| {
                    description.to_lowercase().contains(description_query)
                });

                if !name_match && !description_match {
                    return false;
                }
            }

            queryable.matches(args)
        })
        .collect_vec();

    results.sort_by(|a, b| a.cmp(b, args));

    results
}

/// Combine query results from multiple backends into a single list of [`DeduplicatedMod`], merging by UUID.
fn deduplicate_results<T>(
    results: impl IntoIterator<Item = T>,
    max_count: usize,
) -> impl Iterator<Item = DeduplicatedMod<T>>
where
    T: Queryable,
{
    use indexmap::map::Entry;

    let mut deduped_mods: IndexMap<Uuid, DeduplicatedMod<T>> = IndexMap::new();

    // We need to loop through all results as packages from different sources
    // might be ranked differently, and we want to make sure we get every duplicate.
    for result in results {
        let accepting_new_mods = deduped_mods.len() < max_count;
        let entry = deduped_mods.entry(result.uuid());

        match entry {
            Entry::Occupied(mut occupied_entry) => {
                occupied_entry.get_mut().set(result.backend(), result);
            }
            Entry::Vacant(entry) if accepting_new_mods => {
                let mut deduplicated_mod = DeduplicatedMod::default();
                deduplicated_mod.set(result.backend(), result);
                entry.insert(deduplicated_mod);
            }
            Entry::Vacant(_) => (),
        }
    }

    deduped_mods.into_values()
}
