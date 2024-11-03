use std::{collections::HashSet, fs, path::PathBuf};

use anyhow::{Context, Result};
use itertools::Itertools;

use crate::{
    prefs::Prefs,
    profile::ModManager,
    thunderstore::{BorrowedMod, Thunderstore, VersionIdent},
    util,
};

pub(super) fn path(ident: &VersionIdent, prefs: &Prefs) -> PathBuf {
    let mut path = prefs.cache_dir();

    path.push(ident.full_name());
    path.push(ident.version());

    path
}

pub(super) fn clear(prefs: &Prefs) -> Result<()> {
    let cache_dir = prefs.cache_dir();

    if !cache_dir.exists() {
        return Ok(());
    }

    fs::remove_dir_all(&cache_dir).context("failed to delete cache")?;
    fs::create_dir_all(cache_dir).context("failed to recreate cache directory")?;

    Ok(())
}

pub(super) fn soft_clear(
    manager: &ModManager,
    thunderstore: &Thunderstore,
    prefs: &Prefs,
) -> Result<()> {
    let installed_mods = manager
        .active_game()
        .installed_mods(thunderstore)
        .map_ok(|BorrowedMod { package, version }| (package.ident.as_str(), version.version()))
        .collect::<Result<HashSet<_>>>()
        .context("failed to resolve installed mods")?;

    let packages = prefs
        .cache_dir()
        .read_dir()
        .context("failed to read cache directory")?
        .filter_map(|err| err.ok());

    for entry in packages {
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let package_name = util::fs::file_name_owned(&path);

        if thunderstore.find_package(&package_name).is_err() {
            // package from a game other than the loaded one, skip
            continue;
        }

        let versions = fs::read_dir(&path)
            .with_context(|| format!("failed to read cache for {}", &package_name))?
            .filter_map(|entry| entry.ok());

        for entry in versions {
            let path = entry.path();
            let version = util::fs::file_name_owned(&path);

            if installed_mods.contains(&(&package_name, &version)) {
                // package is installed, skip
                continue;
            }

            fs::remove_dir_all(path)
                .with_context(|| format!("failed to delete cache for {}", &package_name))?;
        }
    }

    Ok(())
}
