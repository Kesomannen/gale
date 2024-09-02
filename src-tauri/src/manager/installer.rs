use anyhow::{anyhow, Context, Result};

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
            let name = &borrowed.package.full_name;
            install(path, &profile.path)?;

            let profile_mod = ProfileMod::remote_now(to_install.mod_ref.clone(), name.clone());
            match to_install.index {
                Some(index) if index < profile.mods.len() => {
                    profile.mods.insert(index, profile_mod);
                }
                _ => {
                    profile.mods.push(profile_mod);
                }
            };

            if !to_install.enabled {
                profile.force_toggle_mod(&borrowed.package.uuid4)?;
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

    let file = fs::File::open(src).context("failed to open file")?;
    let reader = io::BufReader::new(file);
    extract(reader, full_name, path.clone())?;
    install(&path, dest)?;

    fs::remove_dir_all(path).context("failed to remove temporary directory")?;

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

        let path = match cfg!(unix) {
            true => PathBuf::from(file.name().replace('\\', "/")),
            false => PathBuf::from(file.name()),
        };

        if !util::fs::is_enclosed(&path) {
            warn!("file {} escapes the archive root, skipping", path.display());
            continue;
        }

        let mut prev: Option<&OsStr> = None;
        let mut components = path.components();

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
                // extract outside of the BepInEx directory
                let file_name = prev.ok_or(anyhow!("malformed zip file"))?;
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
                let file_name = prev.ok_or(anyhow!("malformed zip file"))?;
                target.push(file_name);
            } else {
                target = components.fold(target, |mut acc, comp| {
                    acc.push(comp);
                    acc
                });
            }
        }

        fs::create_dir_all(target.parent().unwrap())?;
        let mut target_file = fs::File::create(&target)?;
        io::copy(&mut file, &mut target_file)?;
    }

    trace!("extracted in {:?}", start.elapsed());
    Ok(())
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
fn install(src: &Path, dest: &Path) -> Result<()> {
    let config_dir = ["BepInEx", "config"].iter().collect::<PathBuf>();
    let entries = WalkDir::new(src).into_iter().filter_map(|entry| entry.ok());

    for entry in entries {
        let relative = entry
            .path()
            .strip_prefix(src)
            .expect("walkdir should only return full paths inside of the root");

        let target = dest.join(relative);

        if target.exists() {
            // maybe we should overwrite instead?
            continue;
        }

        if entry.file_type().is_dir() {
            fs::create_dir(target)
                .with_context(|| format!("failed to create directory {}", relative.display()))?;
        } else if relative.starts_with(&config_dir) {
            // copy config files so they can be edited without affecting the original
            fs::copy(entry.path(), target)
                .with_context(|| format!("failed to copy file {}", relative.display()))?;
        } else {
            fs::hard_link(entry.path(), target)
                .with_context(|| format!("failed to link file {}", relative.display()))?;
        }
    }

    Ok(())
}
