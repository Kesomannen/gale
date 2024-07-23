use anyhow::{Context, Result};

use super::{downloader::ModInstall, ModManager, ProfileMod};
use crate::{
    prefs::Prefs,
    thunderstore::{BorrowedMod, Thunderstore},
    util::{self, error::IoResultExt, fs::Overwrite},
};
use itertools::Itertools;
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

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
            install_from_disk(path, &profile.path, name)?;

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

                // remove the parent if it's empty
                fs::remove_dir(path.parent().unwrap()).ok();
            }

            Ok(true)
        }
        false => Ok(false),
    }
}

pub fn install_from_disk(src: &Path, dest: &Path, full_name: &str) -> Result<()> {
    let name = match full_name.split_once('-') {
        Some((_, name)) => name,
        None => full_name,
    };

    match name.starts_with("BepInExPack") {
        true => install_bepinex(src, dest),
        false => install_default(src, dest, full_name),
    }
}

fn install_default(src: &Path, dest: &Path, mod_name: &str) -> Result<()> {
    let bepinex = dest.join("BepInEx");
    let plugin_dir = bepinex.join("plugins").join(mod_name);
    fs::create_dir_all(&plugin_dir)?;

    for entry in src.read_dir()? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().unwrap();

        if path.is_dir() {
            if file_name == "BepInEx" {
                install_default(&path, dest, mod_name)?;
                continue;
            }

            let target = match file_name.to_string_lossy().as_ref() {
                // Copy to BepInEx/{plugins | patchers | core | monomod}/{mod_name}
                "patchers" | "core" | "monomod" => bepinex.join(file_name).join(mod_name),
                "plugins" => plugin_dir.clone(),
                // Copy directly without a subfolder
                "config" => bepinex.join("config"),
                // Copy others to the mod's plugin directory
                _ => plugin_dir.join(file_name),
            };

            fs::create_dir_all(target.parent().unwrap())?;

            util::fs::copy_dir(&path, &target, Overwrite::Yes)
                .fs_context("copying directory", &path)?;
        } else {
            fs::copy(&path, &plugin_dir.join(file_name)).fs_context("copying file", &path)?;
        }
    }

    Ok(())
}

fn install_bepinex(src: &Path, dest: &Path) -> Result<()> {
    let target_path = dest.join("BepInEx");

    // Some BepInEx packs come with a subfolder where the actual BepInEx files are
    for entry in fs::read_dir(src)? {
        let entry_path = entry?.path();
        let entry_name = util::fs::file_name_lossy(&entry_path);

        if entry_path.is_dir() && entry_name.contains("BepInEx") {
            // ... and some have even more subfolders ...
            // do this first, since otherwise entry_path will be removed already
            util::fs::flatten(&entry_path.join("BepInEx"), Overwrite::Yes)?;
            util::fs::flatten(&entry_path, Overwrite::Yes)?;
        }
    }

    const EXCLUDES: [&str; 3] = ["icon.png", "manifest.json", "README.md"];

    for entry in fs::read_dir(src)? {
        let entry_path = entry?.path();
        let entry_name = entry_path.file_name().unwrap();

        if entry_path.is_dir() {
            let target_path = target_path.join(entry_name);
            fs::create_dir_all(&target_path)?;

            util::fs::copy_contents(&entry_path, &target_path, Overwrite::Yes)
                .fs_context("copying directory", &entry_path)?;
        } else if !EXCLUDES.iter().any(|exclude| entry_name == *exclude) {
            fs::copy(&entry_path, dest.join(entry_name)).fs_context("copying file", &entry_path)?;
        }
    }

    Ok(())
}
