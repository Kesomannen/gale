use std::{fmt::Display, path::PathBuf, time::Instant};

use eyre::{Context, Result};
use serde::Deserialize;
use tauri::AppHandle;
use tracing::{debug, info, warn};

use crate::{
    game::Game,
    prefs::Prefs,
    state::ManagerExt,
    thunderstore::Thunderstore,
    util::{self, fs::JsonStyle},
};

use super::{ModId, PackageListing};

#[derive(Debug, Deserialize)]
struct MarkdownResponse {
    markdown: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MarkdownCache {
    Readme,
    Changelog,
}

impl Display for MarkdownCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarkdownCache::Readme => write!(f, "readme"),
            MarkdownCache::Changelog => write!(f, "changelog"),
        }
    }
}

pub async fn get_markdown(
    cache: MarkdownCache,
    mod_ref: ModId,
    app: &AppHandle,
) -> Result<Option<String>> {
    let table = format!("{}_cache", cache);
    if let Some(cached) = app.db().get_cached(&table, mod_ref.version_uuid)? {
        return Ok(cached);
    }

    let url = {
        let thunderstore = app.lock_thunderstore();
        let ident = mod_ref.borrow(&thunderstore)?.ident();

        format!(
            "https://thunderstore.io/api/experimental/package/{}/{}/{}/{}/",
            ident.owner(),
            ident.name(),
            ident.version(),
            cache
        )
    };

    let response: MarkdownResponse = app
        .http()
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    app.db()
        .insert_cached(&table, mod_ref.version_uuid, response.markdown.as_deref())?;

    Ok(response.markdown)
}

impl Thunderstore {
    pub fn read_and_insert_cache(&mut self, game: Game, prefs: &Prefs) {
        match get_packages(game, prefs) {
            Ok(Some(mods)) => {
                for package in mods {
                    self.packages.insert(package.uuid, package);
                }
            }
            Ok(None) => (),
            Err(err) => warn!("failed to read cache: {}", err),
        }
    }
}

fn get_packages(game: Game, prefs: &Prefs) -> Result<Option<Vec<PackageListing>>> {
    let start = Instant::now();
    let path = cache_path(game, prefs);

    if !path.exists() {
        info!("no cache file found at {}", path.display());
        return Ok(None);
    }

    let result: Vec<PackageListing> =
        util::fs::read_json(path).context("failed to deserialize cache")?;

    debug!(
        "read {} packages from cache in {:?}",
        result.len(),
        start.elapsed()
    );

    Ok(Some(result))
}

pub fn write_packages(packages: &[&PackageListing], game: Game, prefs: &Prefs) -> Result<()> {
    if packages.is_empty() {
        info!("no packages to write to cache");
        return Ok(());
    }

    let start = Instant::now();

    util::fs::write_json(cache_path(game, prefs), packages, JsonStyle::Compact)
        .context("failed to write mod cache")?;

    debug!(
        "wrote {} packages to cache in {:?}",
        packages.len(),
        start.elapsed()
    );

    Ok(())
}

fn cache_path(game: Game, prefs: &Prefs) -> PathBuf {
    prefs
        .data_dir
        .join(&*game.slug)
        .join("thunderstore_cache.json")
}
