use std::{cmp::Ordering, collections::HashSet, sync::Mutex, time::Duration};

use anyhow::Result;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use typeshare::typeshare;

use super::{
    models::{FrontendMod, FrontendModKind, FrontendVersion},
    BorrowedMod, Thunderstore,
};

use crate::manager::LocalMod;

pub fn setup(app: &AppHandle) {
    app.manage(Mutex::new(QueryState::default()));

    tauri::async_runtime::spawn(query_loop(app.clone()));
}

#[derive(Default)]
pub struct QueryState {
    pub current_query: Option<QueryModsArgs>,
}

#[typeshare]
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

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QueryModsArgs {
    pub max_count: usize,
    pub search_term: Option<String>,
    pub include_categories: HashSet<String>,
    pub exclude_categories: HashSet<String>,
    pub include_nsfw: bool,
    pub include_deprecated: bool,
    pub include_disabled: bool,
    pub sort_by: SortBy,
    pub sort_order: SortOrder,
}

const TIME_BETWEEN_QUERIES: Duration = Duration::from_millis(250);

pub async fn query_loop(app: AppHandle) -> Result<()> {
    let thunderstore = app.state::<Mutex<Thunderstore>>();
    let query_state = app.state::<Mutex<QueryState>>();

    loop {
        {
            let mut state = query_state.lock().unwrap();

            if let Some(args) = &state.current_query {
                let thunderstore = thunderstore.lock().unwrap();

                let mods = query_frontend_mods(args, thunderstore.latest());
                app.emit_all("mod_query_result", &mods)?;

                if thunderstore.finished_loading {
                    state.current_query = None;
                }
            }
        };

        tokio::time::sleep(TIME_BETWEEN_QUERIES).await;
    }
}

pub trait Queryable {
    fn full_name(&self) -> &str;
    fn matches(&self, args: &QueryModsArgs) -> bool;
    fn cmp(&self, other: &Self, args: &QueryModsArgs) -> Ordering;
}

impl Queryable for BorrowedMod<'_> {
    fn full_name(&self) -> &str {
        &self.package.full_name
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

        match args.sort_by {
            SortBy::Newest => a.date_created.cmp(&b.date_created),
            SortBy::Name => b.name.cmp(&a.name),
            SortBy::Author => b.full_name.cmp(&a.full_name),
            SortBy::LastUpdated => a.date_updated.cmp(&b.date_updated),
            SortBy::Downloads => a.total_downloads().cmp(&b.total_downloads()),
            SortBy::Rating => a.rating_score.cmp(&b.rating_score),
            SortBy::DiskSpace => self.version.file_size.cmp(&other.version.file_size),
            SortBy::InstallDate => Ordering::Equal,
            SortBy::Custom => Ordering::Equal,
        }
    }
}

impl From<BorrowedMod<'_>> for FrontendMod {
    fn from(value: BorrowedMod<'_>) -> Self {
        let pkg = value.package;
        let vers = pkg.get_version(&value.version.uuid4).unwrap();
        FrontendMod {
            name: pkg.name.clone(),
            description: Some(vers.description.clone()),
            version: Some(vers.version_number.clone()),
            categories: Some(pkg.categories.clone()),
            author: Some(pkg.owner.clone()),
            rating: Some(pkg.rating_score),
            downloads: Some(pkg.total_downloads()),
            website_url: match vers.website_url.is_empty() {
                true => None,
                false => Some(vers.website_url.clone()),
            },
            donate_url: pkg.donation_link.clone(),
            icon: Some(vers.icon.clone()),
            dependencies: Some(vers.dependencies.clone()),
            is_pinned: pkg.is_pinned,
            is_deprecated: pkg.is_deprecated,
            contains_nsfw: pkg.has_nsfw_content,
            uuid: pkg.uuid4,
            last_updated: Some(pkg.versions[0].date_created.to_string()),
            versions: pkg
                .versions
                .iter()
                .map(|v| FrontendVersion {
                    name: v.version_number.clone(),
                    uuid: v.uuid4,
                })
                .collect(),
            kind: FrontendModKind::Remote,
        }
    }
}

impl Queryable for LocalMod {
    fn full_name(&self) -> &str {
        &self.name
    }

    fn matches(&self, _args: &QueryModsArgs) -> bool {
        true
    }

    fn cmp(&self, other: &Self, args: &QueryModsArgs) -> Ordering {
        match args.sort_by {
            SortBy::Name => other.name.cmp(&self.name),
            SortBy::Author => match (&other.author, &self.author) {
                (Some(a), Some(b)) => a.cmp(b),
                (Some(_), None) => Ordering::Greater,
                (None, Some(_)) => Ordering::Less,
                (None, None) => Ordering::Equal,
            },
            _ => Ordering::Equal,
        }
    }
}

impl From<LocalMod> for FrontendMod {
    fn from(value: LocalMod) -> Self {
        FrontendMod {
            name: value.name,
            description: value.description,
            version: value.version,
            uuid: value.uuid,
            dependencies: value.dependencies,
            icon: value.icon.and_then(|path| Some(path.to_str()?.to_owned())),
            kind: FrontendModKind::Local,
            ..Default::default()
        }
    }
}

pub fn query_frontend_mods<T, I>(args: &QueryModsArgs, mods: I) -> Vec<FrontendMod>
where
    T: Queryable + Into<FrontendMod>,
    I: Iterator<Item = T>,
{
    query_mods(args, mods)
        .map_into()
        .collect()
}

pub fn query_mods<'a, T, I>(args: &QueryModsArgs, mods: I) -> impl Iterator<Item = T> + 'a
where
    T: Queryable + 'a,
    I: Iterator<Item = T> + 'a,
{
    let search_term = args
        .search_term
        .as_ref()
        .map(|s| s.to_lowercase().trim().replace(' ', "_"));

    let mut result = mods
        .filter(|queryable| {
            if let Some(search_term) = &search_term {
                if !queryable.full_name().to_lowercase().contains(search_term) {
                    return false;
                }
            }

            queryable.matches(args)
        })
        .collect_vec();

    result.sort_by(|a, b| match args.sort_order {
        SortOrder::Ascending => a.cmp(b, args),
        SortOrder::Descending => b.cmp(a, args),
    });

    result.into_iter().take(args.max_count)
}
