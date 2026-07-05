use std::{fmt::Display, path::PathBuf, time::Instant};

use eyre::{Context, Result};
use itertools::Itertools;
use serde::Deserialize;
use tauri::AppHandle;
use tracing::{debug, info, warn};

use crate::{
    game::Game,
    prefs::Prefs,
    state::ManagerExt,
    thunderstore::backend::ThunderstoreBackend,
    util::{self, fs::JsonStyle},
};
use super::{Backend, ModId, PackageListing};

#[derive(Debug, Deserialize)]
struct MarkdownResponse {
    markdown: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MarkdownKind {
    Readme,
    Changelog,
}

impl Display for MarkdownKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarkdownKind::Readme => write!(f, "readme"),
            MarkdownKind::Changelog => write!(f, "changelog"),
        }
    }
}

pub async fn get_markdown(
    cache: MarkdownKind,
    mod_id: ModId,
    app: &AppHandle,
) -> Result<Option<String>> {
    let table = format!("{cache}_cache");
    if let Some(cached) = app.db().get_cached(&table, mod_id.version_uuid)? {
        return Ok(cached);
    }

    let url = {
        let thunderstore = app.lock_thunderstore();
        let ident = mod_id.borrow(&thunderstore)?.ident();
        mod_id.backend.get_markdown_url(ident, cache)
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
        .insert_cached(&table, mod_id.version_uuid, response.markdown.as_deref())?;

    Ok(response.markdown)
}

impl ThunderstoreBackend {
    pub fn read_and_insert_cache(&mut self, game: Game, prefs: &Prefs, backend: Backend) {
        match get_packages(game, prefs, backend) {
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

fn get_packages(game: Game, prefs: &Prefs, backend: Backend) -> Result<Option<Vec<PackageListing>>> {
    let start = Instant::now();
    let path = cache_path(game, prefs, backend);

    if !path.exists() {
        info!("no cache file found at {}", path.display());
        return Ok(None);
    }

    let result: Vec<PackageListing> = util::fs::read_json::<Vec<_>>(path).context("failed to deserialize cache")?.into_iter().map(|p| PackageListing { backend, ..p }).collect();

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
    for (backend, packages) in &packages.iter().chunk_by(|p| p.backend) {
        util::fs::write_json(cache_path(game, prefs, backend), &packages.collect::<Vec<_>>(), JsonStyle::Compact)
            .context("failed to write mod cache")?;
    }

    debug!(
        "wrote {} packages to cache in {:?}",
        packages.len(),
        start.elapsed()
    );

    Ok(())
}

fn cache_path(game: Game, prefs: &Prefs, backend: Backend) -> PathBuf {
    prefs
        .data_dir
        .join(&*game.slug)
        .join(format!("{}_cache.json", backend))
}
