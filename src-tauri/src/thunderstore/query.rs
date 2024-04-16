use std::{cmp::Ordering, sync::Mutex, time::Duration};

use anyhow::Result;
use itertools::Itertools;
use serde::Deserialize;
use tauri::{AppHandle, Manager};
use typeshare::typeshare;

use super::{models::{FrontendMod, FrontendModKind}, BorrowedMod, Thunderstore};
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
    page: usize,
    page_size: usize,
    search_term: Option<String>,
    categories: Vec<String>,
    include_nsfw: bool,
    include_deprecated: bool,
    sort_by: SortBy,
    descending: bool,
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
                    let mods = query_mods(args, thunderstore.queryable());
                    app.emit_all("mod_query_result", &mods)?;
                }
            }
        };

        tokio::time::sleep(TIME_BETWEEN_QUERIES).await;
    }
}

pub enum Queryable<'a> {
    Local(&'a LocalMod),
    Online(BorrowedMod<'a>),
}

impl From<Queryable<'_>> for FrontendMod {
    fn from(queryable: Queryable) -> Self {
        match queryable {
            Queryable::Local(local_mod) => {
                let local_mod = local_mod.clone();

                FrontendMod {
                    name: local_mod.name,
                    description: local_mod.description,
                    version: local_mod.version,
                    uuid: local_mod.uuid,
                    dependencies: local_mod.dependencies,
                    icon: local_mod.icon.and_then(|path| Some(path.to_str()?.to_owned())),
                    kind: FrontendModKind::Local,
                    ..Default::default()
                }
            },
            Queryable::Online(borrowed_mod) => {
                let pkg = borrowed_mod.package;
                let vers = &pkg.versions[0];

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
                    latest_version_uuid: Some(vers.uuid4),
                    kind: FrontendModKind::Remote,
                }
            }
        }
    }
}

pub fn query_mods<'a, T>(args: &QueryModsArgs, mods: T) -> Vec<FrontendMod>
where
    T: Iterator<Item = Queryable<'a>>,
{
    let search_term = args.search_term.as_ref().map(|s| s.to_lowercase());

    mods.filter(|queryable| {
        if let Some(search_term) = &search_term {
            let full_name = match queryable {
                Queryable::Local(local_mod) => &local_mod.name,
                Queryable::Online(borrowed_mod) => &borrowed_mod.package.full_name,
            };

            if !full_name.to_lowercase().contains(search_term) {
                return false;
            }
        }

        match queryable {
            Queryable::Local(_) => true,
            Queryable::Online(borrowed_mod) => {
                let package = borrowed_mod.package;

                if !args.include_nsfw && package.has_nsfw_content
                    || !args.include_deprecated && package.is_deprecated
                {
                    return false;
                }

                if args.categories.is_empty() {
                    return true;
                }

                for category in &args.categories {
                    if package.categories.contains(category) {
                        return true;
                    }
                }

                false
            }
        }
    })
    .sorted_by(|a, b| match (a, b) {
        (Queryable::Local(_), _) => Ordering::Less,
        (_, Queryable::Local(_)) => Ordering::Greater,
        (Queryable::Online(a), Queryable::Online(b)) => {
            let (a, b) = (a.package, b.package);

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
    })
    .skip(args.page * args.page_size)
    .take(args.page_size)
    .map(FrontendMod::from)
    .collect()
}
