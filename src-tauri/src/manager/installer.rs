use anyhow::{Context, Result};

use super::{downloader::ModInstall, ModManager, ProfileMod};
use crate::{
    prefs::Prefs,
    thunderstore::{BorrowedMod, Thunderstore},
    util::{self},
};
use itertools::Itertools;
use log::{trace, warn};
use std::{
    collections::HashSet,
    ffi::OsStr,
    fs,
    io::{self, Read, Seek},
    path::{Path, PathBuf},
    time::Instant,
};
use tempfile::tempdir;
use walkdir::WalkDir;
use zip::ZipArchive;

fn is_bepinex(full_name: &str) -> bool {
    match full_name {
        "bbepis-BepInExPack"
        | "xiaoxiao921-BepInExPack"
        | "xiaoye97-BepInEx"
        | "denikson-BepInExPack_Valheim"
        | "1F31A-BepInEx_Valheim_Full"
        | "bbepisTaleSpire-BepInExPack"
        | "Zinal001-BepInExPack_MECHANICA"
        | "bbepis-BepInEx_Rogue_Tower"
        | "Subnautica_Modding-BepInExPack_Subnautica"
        | "Subnautica_Modding-BepInExPack_Subnautica_Experimental"
        | "Subnautica_Modding-BepInExPack_BelowZero"
        | "PCVR_Modders-BepInExPack_GHVR"
        | "BepInExPackMTD-BepInExPack_20MTD"
        | "Modding_Council-BepInExPack_of_Legend"
        | "SunkenlandModding-BepInExPack_Sunkenland"
        | "BepInEx_Wormtown-BepInExPack" => true,
        full_name if full_name.starts_with("BepInEx-BepInExPack") => true,
        _ => false,
    }
}

pub fn cache_path(borrowed_mod: BorrowedMod, prefs: &Prefs) -> Result<PathBuf> {
    let mut path = prefs.cache_dir();

    path.push(&borrowed_mod.package.owner);
    path.push(&borrowed_mod.package.name);
    path.push(borrowed_mod.version.version_number.to_string());

    Ok(path)
}

pub fn clear_cache(prefs: &Prefs) -> Result<()> {
    let cache_dir = prefs.cache_dir();

    if !cache_dir.exists() {
        return Ok(());
    }

    fs::remove_dir_all(&cache_dir).context("failed to delete cache")?;
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

    let packages = fs::read_dir(prefs.cache_dir())
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
            super::installer::install_default(path, &profile.path)?;

            let profile_mod = ProfileMod::remote_now(install.mod_ref.clone(), name.clone());
            match install.index {
                Some(index) if index < profile.mods.len() => {
                    profile.mods.insert(index, profile_mod);
                }
                _ => {
                    profile.mods.push(profile_mod);
                }
            };

            if !install.enabled {
                profile.force_toggle_mod(&borrowed.package.uuid4)?;
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

pub fn install_from_zip(src: &Path, dest: &Path, full_name: &str) -> Result<()> {
    // temporarily extract the zip so the same install method can be used
    let temp_dir = tempdir().context("failed to create temporary directory")?;

    let file = fs::File::open(src).context("failed to open file")?;
    extract(file, full_name, temp_dir.path().to_path_buf())?;
    install_default(&temp_dir.path(), dest)?;

    Ok(())
}

pub fn extract(src: impl Read + Seek, full_name: &str, mut path: PathBuf) -> Result<()> {
    use std::path::Component;

    let start = Instant::now();

    path.push("BepInEx");

    fs::create_dir_all(&path)?;

    let is_bepinex = is_bepinex(full_name);
    let mut archive = ZipArchive::new(src)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;

        if file.is_dir() {
            continue; // we create the necessary dirs when copying files instead
        }

        let file_path = match file.enclosed_name() {
            Some(path) => path,
            None => {
                warn!("zip file {} escapes the archive, skipping", file.name());
                continue;
            }
        };

        let mut prev: Option<&OsStr> = None;
        let mut components = file_path.components();

        let (subdir, is_top_level) = loop {
            let current = components.next();

            match current {
                Some(Component::Normal(name)) => match name.to_str() {
                    Some("plugins") => break ("plugins", false),
                    Some("config") => break ("config", false),
                    Some("patchers") => break ("patchers", false),
                    Some("monomod") => break ("monomod", false),
                    Some("core") => break ("core", false),
                    _ => prev = Some(name),
                },
                Some(_) => prev = None,
                None => break ("plugins", true),
            }
        };

        let mut target: PathBuf;

        if is_bepinex {
            if is_top_level {
                let file_name = match prev {
                    Some(name) => name,
                    None => {
                        warn!("zip file {} has no name, skipping", file_path.display());
                        continue;
                    }
                };

                // extract outside of the BepInEx directory
                target = path.parent().unwrap().join(file_name);
            } else {
                target = path.join(subdir);
                target = components.fold(target, |mut acc, comp| {
                    acc.push(comp);
                    acc
                });
            }
        } else {
            target = path.join(subdir);
            if subdir != "config" {
                target.push(full_name);
            }

            if is_top_level {
                let file_name = match prev {
                    Some(name) => name,
                    None => {
                        warn!("zip file {} has no name, skipping", file_path.display());
                        continue;
                    }
                };

                target.push(file_name)
            } else {
                target = components.fold(target, |mut acc, comp| {
                    acc.push(comp);
                    acc
                });
            }
        }

        trace!("extracting {} to {}", file_path.display(), target.display());

        fs::create_dir_all(target.parent().unwrap())?;
        let mut target_file = fs::File::create(&target)?;
        io::copy(&mut file, &mut target_file)?;
    }

    trace!("extracted in {:?}", start.elapsed());
    Ok(())
}

// install from a well structured mod directory
// for example:
// - BepInEx (src)
//   - KeepItDown
//     - plugins
//       - Kesomannen-KeepItDown
//         - KeepItDown.dll
//         - manifest.json
//         - ...
//     - config
//       - KeepItDown.cfg
fn install_default(src: &Path, dest: &Path) -> Result<()> {
    let entries = WalkDir::new(src).into_iter().filter_map(|entry| entry.ok());

    for entry in entries {
        let relative = entry
            .path()
            .strip_prefix(src)
            .expect("walkdir should only return full paths inside of the root");

        let target = dest.join(relative);

        if target.exists() {
            continue;
        }

        if entry.file_type().is_dir() {
            fs::create_dir(target)
                .with_context(|| format!("failed to create directory {}", relative.display()))?;
        } else {
            fs::hard_link(entry.path(), target)
                .with_context(|| format!("failed to link file {}", relative.display()))?;
        }
    }

    Ok(())
}

/*
fn install_bepinex(src: &Path, dest: &Path) -> Result<()> {
    let target_path = dest.join("BepInEx");

    // Some BepInEx packs come with a subfolder where the actual BepInEx files are
    for entry in src.read_dir()? {
        let entry = entry?;
        let entry_path = entry.path();

        let entry_name = entry.file_name();
        let entry_name = entry_name.to_string_lossy();

        if entry_path.is_dir() && entry_name.contains("BepInEx") {
            // ... and some have even more subfolders ...
            // do this first, since otherwise entry_path will be removed already
            util::fs::flatten(&entry_path.join("BepInEx"), Overwrite::Yes)?;
            util::fs::flatten(&entry_path, Overwrite::Yes)?;
        }
    }

    const EXCLUDES: [&str; 4] = ["icon.png", "manifest.json", "README.md", "changelog.txt"];

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
*/
