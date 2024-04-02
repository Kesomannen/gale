use std::{cmp::Ordering, sync::Mutex};

use anyhow::Result;
use itertools::Itertools;
use serde::Deserialize;
use tauri::{AppHandle, Manager};
use typeshare::typeshare;

use super::{BorrowedMod, OwnedMod, ThunderstoreState};

pub struct QueryState {
    pub current_query: Mutex<Option<QueryModsArgs>>,
}

impl QueryState {
    pub fn new() -> Self {
        Self {
            current_query: Mutex::new(None),
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
    sort_by: Option<SortBy>,
    descending: bool,
}

pub async fn query_loop(app: AppHandle) -> Result<()> {
    loop {
        let finished = {
            let thunderstore = app.state::<ThunderstoreState>();
            let query_state = app.state::<QueryState>();

            let current_query = query_state.current_query.lock().unwrap();
            if let Some(args) = current_query.as_ref() {
                let packages = thunderstore.packages.lock().unwrap();
                let mods = query_mods(args, super::latest_versions(&packages));

                app.emit_all("mod_query_result", mods)?;
            }

            let finished_loading = thunderstore.finished_loading.lock().unwrap();
            *finished_loading
        };

        if finished {
            return Ok(());
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
    }
}

pub fn query_mods<'a, T>(args: &QueryModsArgs, mods: T) -> Vec<OwnedMod>
where
    T: Iterator<Item = BorrowedMod<'a>>,
{
    let search_term = args.search_term.as_ref().map(|s| s.to_lowercase());

    let result = mods
        .filter(|borrowed_mod| {
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
                None => Ordering::Equal,
                Some(SortBy::LastUpdated) => a.date_updated.cmp(&b.date_updated),
                Some(SortBy::Downloads) => a.total_downloads().cmp(&b.total_downloads()),
                Some(SortBy::Rating) => a.rating_score.cmp(&b.rating_score),
            };
            match args.descending {
                true => ordering.reverse(),
                false => ordering,
            }
        })
        .skip(args.page * args.page_size)
        .take(args.page_size)
        .map(OwnedMod::from)
        .collect();

    result
}
