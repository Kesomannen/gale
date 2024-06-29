use anyhow::{Context, Result};

use super::{downloader::ModInstall, ModManager, ProfileMod};
use crate::{
    prefs::Prefs,
    thunderstore::{BorrowedMod, Thunderstore},
    util::{self, error::IoResultExt},
};
use std::{
    collections::HashSet, fs, path::{Path, PathBuf}
};
use itertools::Itertools;

pub fn cache_path(borrowed_mod: BorrowedMod, prefs: &Prefs) -> Result<PathBuf> {
    let mut path = prefs.cache_dir.get().to_path_buf();
    path.push(&borrowed_mod.package.full_name);
    path.push(&borrowed_mod.version.version_number.to_string());

    Ok(path)
}

pub fn clear_cache(prefs: &Prefs) -> Result<()> {
    let cache_dir = prefs.cache_dir.get();

    if !cache_dir.exists() {
        return Ok(());
    }

    fs::remove_dir_all(cache_dir).context("failed to delete cache")?;
    fs::create_dir_all(cache_dir).context("failed to recreate cache directory")?;

    Ok(())
}

pub fn soft_clear_cache(
    manager: &ModManager,
    thunderstore: &Thunderstore,
    prefs: &Prefs,
) -> Result<()> {
    let installed_mods = manager
        .active_game()
        .installed_mods(thunderstore)
        .map_ok(|borrowed| {
            (
                &borrowed.package.full_name,
                borrowed.version.version_number.to_string(),
            )
        })
        .collect::<Result<HashSet<_>>>()
        .context("failed to resolve installed mods")?;

    let packages = fs::read_dir(&*prefs.cache_dir)
        .context("failed to read cache directory")?
        .filter_map(|e| e.ok());

    for entry in packages {
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let package_name = util::fs::file_name_lossy(&path);

        if thunderstore.find_package(&package_name).is_err() {
            // package from a game other than the loaded one, skip
            continue;
        }

        let versions = fs::read_dir(&path)
            .with_context(|| format!("failed to read cache for {}", &package_name))?
            .filter_map(|e| e.ok());

        for entry in versions {
            let path = entry.path();
            let version = util::fs::file_name_lossy(&path);

            if installed_mods.contains(&(&package_name, version)) {
                // package is installed, skip
                continue;
            }

            fs::remove_dir_all(path)
                .with_context(|| format!("failed to delete cache for {}", &package_name))?;
        }
    }

    Ok(())
}

pub fn try_cache_install(
    install: &ModInstall,
    path: &Path,
    manager: &mut ModManager,
    thunderstore: &Thunderstore,
    prefs: &Prefs,
) -> Result<bool> {
    let borrowed = install.mod_ref.borrow(thunderstore)?;
    let profile = manager.active_profile_mut();

    match path.exists() {
        true => {
            let name = &borrowed.package.full_name;
            from_disk(path, &profile.path, name)?;

            let profile_mod = ProfileMod::remote_now(install.mod_ref.clone());
            match install.index {
                Some(index) if index < profile.mods.len() => {
                    profile.mods.insert(index, profile_mod);
                }
                _ => {
                    profile.mods.push(profile_mod);
                }
            };

            if !install.enabled {
                profile.force_toggle_mod(&borrowed.package.uuid4, thunderstore)?;
            }

            if !prefs.mod_cache_enabled() {
                fs::remove_dir_all(path).ok();
                fs::remove_dir(path.parent().unwrap()).ok();
            }

            Ok(true)
        }
        false => Ok(false),
    }
}

pub fn normalize_mod_structure(path: &mut PathBuf) -> Result<()> {
    for dir in ["BepInExPack", "BepInEx", "plugins"].iter() {
        path.push(dir);
        util::fs::flatten(&*path, true)?;
        path.pop();
    }

    Ok(())
}

pub fn from_disk(src: &Path, dest: &Path, full_name: &str) -> Result<()> {
    let name = match full_name.split_once('-') {
        Some((_, name)) => name,
        None => full_name,
    };

    match name.starts_with("BepInExPack") {
        true => bepinex(src, dest),
        false => default(src, dest, full_name),
    }
}

fn default(src: &Path, dest: &Path, name: &str) -> Result<()> {
    let target_path = dest.join("BepInEx");
    let target_plugins_path = target_path.join("plugins").join(name);
    fs::create_dir_all(&target_plugins_path).context("failed to create plugins directory")?;

    for entry in fs::read_dir(src)? {
        let entry_path = entry?.path();
        let entry_name = entry_path.file_name().unwrap();

        if entry_path.is_dir() {
            if entry_name == "config" {
                let target_path = target_path.join("config");
                fs::create_dir_all(&target_path)?;
                util::fs::copy_contents(&entry_path, &target_path, true)
                    .fs_context("copying config", &entry_path)?;
            } else {
                let target_path = match entry_name.to_string_lossy().as_ref() {
                    "patchers" | "core" | "monomod" => target_path.join(entry_name).join(name),
                    "plugins" => target_plugins_path.clone(),
                    _ => target_plugins_path.join(entry_name),
                };

                fs::create_dir_all(target_path.parent().unwrap())?;
                util::fs::copy_dir(&entry_path, &target_path, true)
                    .fs_context("copying directory", &entry_path)?;
            }
        } else {
            fs::copy(&entry_path, &target_plugins_path.join(entry_name))
                .fs_context("copying file", &entry_path)?;
        }
    }

    Ok(())
}

fn bepinex(src: &Path, dest: &Path) -> Result<()> {
    let target_path = dest.join("BepInEx");

    // Some BepInEx packs come with a subfolder where the actual BepInEx files are
    for entry in fs::read_dir(src)? {
        let entry_path = entry?.path();
        let entry_name = util::fs::file_name_lossy(&entry_path);

        if entry_path.is_dir() && entry_name.contains("BepInEx") {
            // ... and some have even more subfolders ...
            // do this first, since otherwise entry_path will be removed already
            util::fs::flatten(&entry_path.join("BepInEx"), true)?;
            util::fs::flatten(&entry_path, true)?;
        }
    }

    const EXCLUDES: [&str; 3] = ["icon.png", "manifest.json", "README.md"];

    for entry in fs::read_dir(src)? {
        let entry_path = entry?.path();
        let entry_name = entry_path.file_name().unwrap();

        if entry_path.is_dir() {
            let target_path = target_path.join(entry_name);
            fs::create_dir_all(&target_path)?;

            util::fs::copy_contents(&entry_path, &target_path, true)
                .fs_context("copying directory", &entry_path)?;
        } else if !EXCLUDES.iter().any(|exclude| entry_name == *exclude) {
            fs::copy(&entry_path, dest.join(entry_name)).fs_context("copying file", &entry_path)?;
        }
    }

    Ok(())
}
