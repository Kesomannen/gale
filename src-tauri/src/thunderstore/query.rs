use std::{cmp::Ordering, sync::Mutex, time::Duration};

use anyhow::Result;
use itertools::Itertools;
use serde::Deserialize;
use tauri::{AppHandle, Manager};
use typeshare::typeshare;

use super::{models::{FrontendMod, FrontendModKind, FrontendVersion}, BorrowedMod, Thunderstore};
use crate::manager::LocalMod;

pub fn setup(app: &AppHandle) -> Result<()> {
    app.manage(Mutex::new(QueryState::new()));

    tauri::async_runtime::spawn(query_loop(app.clone()));

    Ok(())
}

pub struct QueryState {
    pub current_query: Option<QueryModsArgs>,
}

impl QueryState {
    pub fn new() -> Self {
        Self {
            current_query: None,
        }
    }
}

#[typeshare]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum SortBy {
    LastUpdated,
    Downloads,
    Rating,
}

#[typeshare]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QueryModsArgs {
    pub page: usize,
    pub page_size: usize,
    pub search_term: Option<String>,
    pub categories: Vec<String>,
    pub include_nsfw: bool,
    pub include_deprecated: bool,
    pub include_disabled: bool,
    pub sort_by: SortBy,
    pub descending: bool,
}

const TIME_BETWEEN_QUERIES: Duration = Duration::from_millis(250);

pub async fn query_loop(app: AppHandle) -> Result<()> {
    let thunderstore = app.state::<Mutex<Thunderstore>>();
    let query_state = app.state::<Mutex<QueryState>>();

    loop {
        {
            let thunderstore = thunderstore.lock().unwrap();

            if !thunderstore.finished_loading {
                let query_state = query_state.lock().unwrap();
            
                if let Some(args) = query_state.current_query.as_ref() {
                    let mods = query_frontend_mods(args, thunderstore.latest());
                    app.emit_all("mod_query_result", &mods)?;
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

        if args.categories.is_empty() {
            return true;
        }

        for category in &args.categories {
            if pkg.categories.contains(category) {
                return true;
            }
        }

        false
    }

    fn cmp(&self, other: &Self, args: &QueryModsArgs) -> Ordering {
        let (a, b) = (self.package, other.package);

        match (a.is_pinned, b.is_pinned) {
            (true, false) => return Ordering::Less,
            (false, true) => return Ordering::Greater,
            _ => (),
        }

        let ordering = match args.sort_by {
            SortBy::LastUpdated => a.date_updated.cmp(&b.date_updated),
            SortBy::Downloads => a.total_downloads().cmp(&b.total_downloads()),
            SortBy::Rating => a.rating_score.cmp(&b.rating_score),
        };

        match args.descending {
            true => ordering.reverse(),
            false => ordering,
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
            uuid: pkg.uuid4,
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

    fn cmp(&self, _other: &Self, _args: &QueryModsArgs) -> Ordering {
        Ordering::Greater
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
        .map(|queryable| queryable.into())
        .collect()
}


pub fn query_mods<'a, T, I>(args: &QueryModsArgs, mods: I) -> impl Iterator<Item = T> + 'a 
where
    T: Queryable + 'a,
    I: Iterator<Item = T> + 'a,
{
    let search_term = args.search_term.as_ref().map(|s| s.to_lowercase());

    mods.filter(|queryable| {
        if let Some(search_term) = &search_term {
            if !queryable.full_name().to_lowercase().contains(search_term) {
                return false;
            }
        }

        queryable.matches(args)
    })
    .sorted_by(|a, b| a.cmp(b, args))
    .skip(args.page * args.page_size)
    .take(args.page_size)
}
