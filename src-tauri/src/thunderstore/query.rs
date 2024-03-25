use std::cmp::Ordering;

use itertools::Itertools;
use serde::Deserialize;

use super::{BorrowedMod, OwnedMod};

#[derive(Deserialize, Debug)]
pub enum SortBy {
    LastUpdated,
    Downloads,
    Rating,
}

#[derive(Deserialize, Debug)]
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

pub fn query_mods<'a, T>(mut args: QueryModsArgs, mods: T) -> Vec<OwnedMod>
where
    T: Iterator<Item = BorrowedMod<'a>>,
{
    args.search_term = args.search_term.map(|s| s.to_lowercase());

    let result = mods
        .filter(|borrowed_mod| {
            let package = borrowed_mod.package;

            if !args.include_nsfw && package.has_nsfw_content
                || !args.include_deprecated && package.is_deprecated
            {
                return false;
            }

            if let Some(search_term) = &args.search_term {
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
