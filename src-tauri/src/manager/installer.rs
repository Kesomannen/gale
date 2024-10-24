use anyhow::{Context, Result};

use super::{downloader::ModInstall, ModManager, ProfileMod};
use crate::{
    prefs::Prefs,
    thunderstore::{BorrowedMod, Thunderstore},
    util::{self},
};
use chrono::Utc;
use itertools::Itertools;
use log::{debug, warn};
use std::{
    borrow::Cow,
    collections::HashSet,
    ffi::OsStr,
    fs::{self, File},
    io::{self, Read, Seek},
    path::{Path, PathBuf},
    time::Instant,
};
use walkdir::WalkDir;
use zip::ZipArchive;

#[cfg(test)]
mod tests;

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

    path.push(&borrowed_mod.package.full_name);
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
    to_install: &ModInstall,
    path: &Path,
    manager: &mut ModManager,
    thunderstore: &Thunderstore,
) -> Result<bool> {
    let borrowed = to_install.mod_ref.borrow(thunderstore)?;
    let profile = manager.active_profile_mut();

    match path.exists() {
        true => {
            install(path, &profile.path, to_install.overwrite)?;

            let install_time = to_install.install_time.unwrap_or(Utc::now());
            let profile_mod = ProfileMod {
                install_time,
                enabled: true, // we switch it off later if needed
                kind: super::ProfileModKind::Remote {
                    mod_ref: to_install.mod_ref.clone(),
                    full_name: borrowed.version.full_name.clone(),
                },
            };

            match to_install.index {
                Some(index) if index < profile.mods.len() => {
                    profile.mods.insert(index, profile_mod);
                }
                _ => {
                    profile.mods.push(profile_mod);
                }
            };

            if !to_install.enabled {
                profile.force_toggle_mod(borrowed.package.uuid4)?;
            }

            Ok(true)
        }
        false => Ok(false),
    }
}

pub fn install_from_zip(src: &Path, dest: &Path, full_name: &str, prefs: &Prefs) -> Result<()> {
    // temporarily extract the zip so the same install method can be used

    // dont use tempdir since we need the files on the same drive as the destination
    // for hard linking to work

    let path = prefs.data_dir.join("temp").join("extract");
    fs::create_dir_all(&path).context("failed to create temporary directory")?;

    let file = File::open(src).context("failed to open file")?;
    let reader = io::BufReader::new(file);
    extract(reader, full_name, path.clone())?;
    install(&path, dest, false)?;

    fs::remove_dir_all(path).context("failed to remove temporary directory")?;

    Ok(())
}

pub fn extract(src: impl Read + Seek, full_name: &str, dest: PathBuf) -> Result<()> {
    let start = Instant::now();

    let is_bepinex = is_bepinex(full_name);
    let mut archive = ZipArchive::new(src)?;

    for i in 0..archive.len() {
        let mut source_file = archive.by_index(i)?;

        if source_file.is_dir() {
            continue; // we create the necessary dirs when copying files instead
        }

        let relative_path = match cfg!(unix) {
            true => PathBuf::from(source_file.name().replace('\\', "/")),
            false => PathBuf::from(source_file.name()),
        };

        if !util::fs::is_enclosed(&relative_path) {
            warn!(
                "file {} escapes the archive root, skipping",
                relative_path.display()
            );
            continue;
        }

        let target_path_rel = if is_bepinex {
            let Some(path) = map_file_bepinex(&relative_path) else {
                continue;
            };

            Cow::Borrowed(path)
        } else {
            let path = map_file_default(&relative_path, full_name)?;

            Cow::Owned(path)
        };

        let target_path = dest.join(target_path_rel);

        fs::create_dir_all(target_path.parent().unwrap())?;

        let mut target_file = File::create(&target_path)?;
        io::copy(&mut source_file, &mut target_file)?;
    }

    debug!("extracted {} in {:?}", full_name, start.elapsed());

    Ok(())
}

fn map_file_bepinex<'a>(relative_path: &'a Path) -> Option<&'a Path> {
    let mut components = relative_path.components();
    if components.clone().count() == 1 {
        // ignore top-level files, such as manifest.json and icon.png
        return None;
    }

    // remove the top-level dir (usually called BepInExPack)
    components.next();

    Some(components.as_path())
}

/// Maps a file from a mod zip archive to its final extracted path.
/// Based on r2modman's structure rules, which are available here:
/// https://github.com/ebkr/r2modmanPlus/wiki/Structuring-your-Thunderstore-package
fn map_file_default(relative_path: &Path, full_name: &str) -> Result<PathBuf> {
    use std::path::Component;

    const SUBDIRS: &[&str] = &["plugins", "config", "patchers", "monomod", "core"];

    // first, flatten the path until a subdir appears
    // if the path contains no subdirs, default to /plugins

    let mut prev: Vec<&OsStr> = Vec::new();
    let mut components = relative_path.components();

    let (subdir, is_top_level) = loop {
        let current = components.next();

        match current {
            Some(Component::Normal(name)) => {
                // check for a subdir
                if let Some(name) = name.to_str() {
                    if let Some(subdir) = SUBDIRS.iter().find(|subdir| **subdir == name) {
                        break (*subdir, false);
                    }
                }

                // otherwise store the parent and continue
                prev.push(name);
            }
            // remove the previous parent
            Some(Component::ParentDir) => {
                prev.pop();
            }
            // we don't care/don't expect any of these
            Some(Component::RootDir | Component::Prefix(_) | Component::CurDir) => continue,
            // default to plugins when the whole path is exhausted
            None => break ("plugins", true),
        }
    };

    // components now contains either:
    // - the remaining path if a subdir was found, or
    // - None if the file is top level and thus defaulted to plugins

    // prev is the canonical path leading up to a subdir,
    // or the whole path if we defaulted

    // dest is the BepInEx dir inside of the target path

    // e.g. profile/BepInEx/plugins
    let mut target: PathBuf = ["BepInEx", subdir].iter().collect();

    // don't add separators for config
    if subdir != "config" {
        // e.g. profile/BepInEx/plugins/Kesomannen-CoolMod
        target.push(full_name);
    }

    if is_top_level {
        // since we advanced components to the end, prev.pop() will give the
        // last component, i.e. the file name
        let file_name = prev.pop().context("malformed zip file")?;

        // e.g. profile/BepInEx/plugins/Kesomannen-CoolMod/CoolMod.dll
        target.push(file_name);
    } else {
        // add the remainder of the path after the subdir
        // e.g. profile/BepInEx/plugins/Kesomannen-CoolMod/assets/cool_icon.png
        target.push(components.as_path());
    }

    Ok(target)
}

// install from a well structured mod directory
// for example:
// - Kesomannen-KeepItDown (src)
//   - BepInEx
//     - plugins
//       - Kesomannen-KeepItDown
//         - KeepItDown.dll
//         - manifest.json
//         - ...
//     - config
//       - KeepItDown.cfg
fn install(src: &Path, dest: &Path, overwrite: bool) -> Result<()> {
    let config_dir = ["BepInEx", "config"].into_iter().collect::<PathBuf>();
    let entries = WalkDir::new(src).into_iter().filter_map(|entry| entry.ok());

    for entry in entries {
        let relative = entry
            .path()
            .strip_prefix(src)
            .expect("walkdir should only return full paths inside of the root");

        let target = dest.join(relative);
        if entry.file_type().is_dir() {
            if target.exists() {
                continue;
            }

            fs::create_dir(target)
                .with_context(|| format!("failed to create directory {}", relative.display()))?;
        } else {
            if target.exists() {
                match overwrite {
                    true => {
                        fs::remove_file(&target).with_context(|| {
                            format!("failed to remove existing file {}", relative.display())
                        })?;
                    }
                    false => continue,
                }
            }

            if relative.starts_with(&config_dir) {
                // copy config files so they can be edited without affecting the original
                fs::copy(entry.path(), target)
                    .with_context(|| format!("failed to copy file {}", relative.display()))?;
            } else {
                fs::hard_link(entry.path(), target)
                    .with_context(|| format!("failed to link file {}", relative.display()))?;
            }
        }
    }

    Ok(())
}
