use std::{sync::Mutex, time::Duration};

use anyhow::Result;
use itertools::Itertools;
use serde::Deserialize;
use tauri::{AppHandle, Manager};
use typeshare::typeshare;

use super::{BorrowedMod, OwnedMod, Thunderstore};

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
        let finished = {
            let query_state = query_state.lock().unwrap();
            let thunderstore = thunderstore.lock().unwrap();

            if let Some(args) = query_state.current_query.as_ref() {
                let mods = query_mods(args, thunderstore.latest_versions());
                app.emit_all("mod_query_result", mods)?;
            }

            thunderstore.finished_loading
        };

        if finished {
            return Ok(());
        }

        tokio::time::sleep(TIME_BETWEEN_QUERIES).await;
    }
}

pub fn query_mods<'a, T>(args: &QueryModsArgs, mods: T) -> Vec<OwnedMod>
where
    T: Iterator<Item = BorrowedMod<'a>>,
{
    let search_term = args.search_term.as_ref().map(|s| s.to_lowercase());

    mods.filter(|borrowed_mod| {
        let package = borrowed_mod.package;

        if !args.include_nsfw && package.has_nsfw_content
            || !args.include_deprecated && package.is_deprecated
        {
            return false;
        }

        if let Some(search_term) = &search_term {
            if !package.full_name.to_lowercase().contains(search_term) {
                return false;
            }
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
    })
    .sorted_by(|a, b| {
        let (a, b) = (a.package, b.package);
        let ordering = match args.sort_by {
            SortBy::LastUpdated => a.date_updated.cmp(&b.date_updated),
            SortBy::Downloads => a.total_downloads().cmp(&b.total_downloads()),
            SortBy::Rating => a.rating_score.cmp(&b.rating_score),
        };
        match args.descending {
            true => ordering.reverse(),
            false => ordering,
        }
    })
    .skip(args.page * args.page_size)
    .take(args.page_size)
    .map(OwnedMod::from)
    .collect()
}
