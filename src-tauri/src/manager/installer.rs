use anyhow::{Context, Result};

use super::{downloader::ModInstall, ModManager, ProfileMod};
use crate::{
    prefs::Prefs, thunderstore::{BorrowedMod, Thunderstore}, util::{self, error::IoResultExt}
};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn cache_path(borrowed_mod: BorrowedMod, prefs: &Prefs) -> Result<PathBuf> {
    let mut path = prefs.get_path_or_err("cache_dir")?.clone();
    path.push(&borrowed_mod.package.full_name);
    path.push(&borrowed_mod.version.version_number.to_string());

    Ok(path)
}

pub fn try_cache_install(
    install: &ModInstall,
    path: &Path,
    manager: &mut ModManager,
    thunderstore: &Thunderstore,
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

            Ok(true)
        }
        false => Ok(false),
    }
}

pub fn normalize_mod_structure(path: &mut PathBuf) -> Result<()> {
    for dir in ["BepInExPack", "BepInEx", "plugins"].iter() {
        path.push(dir);
        util::fs::flatten_if_exists(&*path)?;
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
                util::fs::copy_dir(&entry_path, &target_path, false)
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
        let entry_name = util::fs::file_name(&entry_path);

        if entry_path.is_dir() && entry_name.contains("BepInEx") {
            // ... and some have even more subfolders ...
            // do this first, since otherwise entry_path will be removed already
            util::fs::flatten_if_exists(&entry_path.join("BepInEx"))?;
            util::fs::flatten_if_exists(&entry_path)?;
        }
    }

    const EXCLUDES: [&str; 3] = ["icon.png", "manifest.json", "README.md"];

    for entry in fs::read_dir(src)? {
        let entry_path = entry?.path();
        let entry_name = entry_path.file_name().unwrap();

        if entry_path.is_dir() {
            let target_path = target_path.join(entry_name);
            fs::create_dir_all(&target_path)?;

            util::fs::copy_contents(&entry_path, &target_path, false)
                .fs_context("copying directory", &entry_path)?;
        } else if !EXCLUDES.iter().any(|exclude| entry_name == *exclude) {
            fs::copy(&entry_path, dest.join(entry_name)).fs_context("copying file", &entry_path)?;
        }
    }

    Ok(())
}
