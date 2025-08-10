use std::{collections::HashSet, fs, path::PathBuf};

use eyre::{Context, Result};
use gale_core::ident::VersionIdent;
use tauri::AppHandle;
use tracing::info;

use crate::{prefs::Prefs, state::ManagerExt};

pub(super) fn path(ident: &VersionIdent, prefs: &Prefs) -> PathBuf {
    let mut path = prefs.cache_dir();

    path.push(ident.full_name());
    path.push(ident.version());

    path
}

pub(super) fn clear(path: PathBuf) -> Result<()> {
    if path.exists() {
        fs::remove_dir_all(&path).context("failed to delete cache directory")?;
        fs::create_dir_all(path).context("failed to recreate cache directory")?;
    }

    Ok(())
}

pub(super) fn prepare_soft_clear(app: AppHandle) -> Result<Vec<PathBuf>> {
    let prefs = app.lock_prefs();
    let manager = app.lock_manager();
    let thunderstore = app.lock_thunderstore();

    let installed_mods = manager
        .active_game()
        .installed_mods(&thunderstore)
        .map(|borrowed| {
            let ident = borrowed.ident();
            (ident.full_name(), ident.version())
        })
        .collect::<HashSet<_>>();

    let packages = prefs
        .cache_dir()
        .read_dir()
        .context("failed to read cache directory")?
        .filter_map(Result::ok);

    let mut to_remove = Vec::new();

    for entry in packages {
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let package_name = gale_util::fs::file_name_owned(&path);

        if thunderstore.find_package(&package_name).is_err() {
            // package from a game other than the loaded one, skip
            continue;
        }

        let versions = fs::read_dir(&path)
            .with_context(|| format!("failed to read cache for {}", &package_name))?
            .filter_map(|entry| entry.ok());

        for entry in versions {
            let path = entry.path();
            let version = gale_util::fs::file_name_owned(&path);

            if installed_mods.contains(&(&package_name, &version)) {
                // package is installed, skip
                continue;
            }

            to_remove.push(path);
        }
    }

    Ok(to_remove)
}

pub(super) fn do_soft_clear(paths: Vec<PathBuf>) -> Result<()> {
    let count = paths.len();

    for path in paths {
        fs::remove_dir_all(path)?;
    }

    info!("cleared {} mods from cache", count);

    Ok(())
}
