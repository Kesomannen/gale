use std::{fmt::Display, path::PathBuf, time::Instant};

use eyre::{Context, Result};
use serde::Deserialize;
use tauri::AppHandle;
use tracing::{debug, info};

use crate::{
    profile::ModManager,
    state::ManagerExt,
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

pub fn get_packages(app: &AppHandle) -> Result<Option<Vec<PackageListing>>> {
    let start = Instant::now();
    let path = cache_path(&app.lock_manager());

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

pub fn write_packages(packages: &[&PackageListing], manager: &ModManager) -> Result<()> {
    if packages.is_empty() {
        info!("no packages to write to cache");
        return Ok(());
    }

    let start = Instant::now();

    util::fs::write_json(cache_path(manager), packages, JsonStyle::Compact)
        .context("failed to write mod cache")?;

    debug!(
        "wrote {} packages to cache in {:?}",
        packages.len(),
        start.elapsed()
    );

    Ok(())
}

fn cache_path(manager: &ModManager) -> PathBuf {
    manager.active_game().path.join("thunderstore_cache.json")
}
