use std::{cmp::Ordering, collections::HashSet, time::Duration};

use eyre::Result;
use internment::Intern;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tracing::info;

use super::{
    models::{FrontendMod, FrontendModKind, FrontendVersion, IntoFrontendMod},
    BorrowedMod,
};
use crate::{
    profile::{LocalMod, Profile},
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
    pub max_count: usize,
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

                let mods =
                    query_frontend_mods(args, thunderstore.latest(), manager.active_profile());
                app.emit("mod_query_result", &mods)?;

                if thunderstore.packages_fetched {
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
    /// The package's full name, including the author.
    fn full_name(&self) -> &str;

    /// Whether the package should be included in the given query.
    fn matches(&self, args: &QueryModsArgs) -> bool;

    /// Whether the package should rank higher than `other` in the given query.
    fn cmp(&self, other: &Self, args: &QueryModsArgs) -> Ordering;

    /// A longer description of the package.
    fn description(&self) -> Option<&str> {
        None
    }
}

impl Queryable for BorrowedMod<'_> {
    fn full_name(&self) -> &str {
        self.package.ident.as_str()
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
}

impl IntoFrontendMod for BorrowedMod<'_> {
    fn into_frontend(self, profile: Option<&Profile>) -> FrontendMod {
        let pkg = self.package;
        let vers = pkg.get_version(self.version.uuid).unwrap();
        FrontendMod {
            name: pkg.name().to_owned(),
            description: Some(vers.description.to_string()),
            version: Some(vers.parsed_version()),
            categories: Some(
                pkg.categories
                    .iter()
                    .map(|intern| intern.to_string())
                    .collect(),
            ),
            author: Some(pkg.owner().to_owned()),
            rating: Some(pkg.rating_score),
            downloads: Some(pkg.total_downloads()),
            file_size: vers.file_size,
            website_url: match vers.website_url.is_empty() {
                true => None,
                false => Some(vers.website_url.to_string()),
            },
            donate_url: pkg.donation_link.clone(),
            dependencies: Some(vers.dependencies.clone()),
            is_pinned: pkg.is_pinned,
            is_deprecated: pkg.is_deprecated,
            contains_nsfw: pkg.has_nsfw_content,
            uuid: pkg.uuid,
            version_uuid: vers.uuid,
            is_installed: profile
                .map(|profile| profile.has_mod(pkg.uuid))
                .unwrap_or(false),
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
        }
    }
}

impl IntoFrontendMod for LocalMod {
    fn into_frontend(self, _profile: Option<&Profile>) -> FrontendMod {
        let LocalMod {
            name,
            description,
            version,
            file_size,
            uuid,
            dependencies,
            icon,
            ..
        } = self;

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

/// Sorts and filters `mods` according to `args` and converts the
/// results to [`FrontendMod`].
pub fn query_frontend_mods<T, I>(
    args: &QueryModsArgs,
    mods: I,
    profile: &Profile,
) -> Vec<FrontendMod>
where
    T: Queryable + IntoFrontendMod,
    I: Iterator<Item = T>,
{
    query_mods(args, mods)
        .map(|m| m.into_frontend(Some(profile)))
        .collect()
}

/// Sorts and filters `mods` according to `args`.
pub fn query_mods<'a, T, I>(args: &QueryModsArgs, mods: I) -> impl Iterator<Item = T> + 'a
where
    T: Queryable + 'a,
    I: Iterator<Item = T> + 'a,
{
    let search_terms = args.search_term.as_ref().map(|str| {
        let full = str.to_lowercase().trim().to_owned();
        let package = full.replace(' ', "_");
        // search for packages with underscores and descriptions with spaces
        (full, package)
    });

    let mut results = mods
        .filter(|queryable| {
            if let Some((full_search, package_search)) = &search_terms {
                let name_match = queryable
                    .full_name()
                    .to_lowercase()
                    .contains(package_search);

                let description_match = queryable
                    .description()
                    .is_some_and(|description| description.to_lowercase().contains(full_search));

                if !name_match && !description_match {
                    return false;
                }
            }

            queryable.matches(args)
        })
        .collect_vec();

    results.sort_by(|a, b| a.cmp(b, args));
    results.into_iter().take(args.max_count)
}
