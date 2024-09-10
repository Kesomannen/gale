use anyhow::{anyhow, Context};
use gale_core::prelude::*;
use std::{
    ffi::OsStr,
    io::{Read, Seek},
    path::{Path, PathBuf},
    time::Instant,
};
use walkdir::WalkDir;
use zip::ZipArchive;

fn is_bepinex(full_name: &str) -> bool {
    match full_name {
        // special cases
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

pub fn extract(src: impl Read + Seek, package_guid: &str, mut dest: PathBuf) -> Result<()> {
    let start = Instant::now();

    dest.push("BepInEx");

    std::fs::create_dir_all(&dest)?;

    let is_bepinex = is_bepinex(package_guid);
    let mut archive = ZipArchive::new(src)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;

        if file.is_dir() {
            continue; // we create the necessary dirs when copying files instead
        }

        let file_path = match cfg!(unix) {
            true => PathBuf::from(file.name().replace('\\', "/")),
            false => PathBuf::from(file.name()),
        };

        if !is_enclosed(&file_path) {
            log::warn!(
                "file {} escapes the archive root, skipping",
                file_path.display()
            );
            continue;
        }

        let target_path = map_file(file_path, &dest, package_guid, is_bepinex)?;

        std::fs::create_dir_all(target_path.parent().unwrap())?;
        let mut target_file = std::fs::File::create(&target_path)?;
        std::io::copy(&mut file, &mut target_file)?;
    }

    log::trace!("extracted in {:?}", start.elapsed());
    Ok(())
}

fn map_file(
    file_path: PathBuf,
    dest: &Path,
    package_guid: &str,
    is_bepinex: bool,
) -> Result<PathBuf> {
    use std::path::Component;

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
            // extract outside of the BepInEx directory
            let file_name = prev.ok_or(anyhow!("malformed zip file"))?;
            target = dest.parent().unwrap().join(file_name);
        } else {
            target = dest.join(subdir);
            target = components.fold(target, |mut acc, comp| {
                acc.push(comp);
                acc
            });
        }
    } else {
        target = dest.join(subdir);
        if subdir != "config" {
            target.push(package_guid);
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

    Ok(target)
}

fn is_enclosed(path: &Path) -> bool {
    use std::path::Component;

    if path
        .as_os_str()
        .to_str()
        .is_some_and(|str| str.contains('\0'))
    {
        return false;
    }

    let mut depth = 0usize;
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => return false,
            Component::ParentDir => match depth.checked_sub(1) {
                Some(new_depth) => depth = new_depth,
                None => return false,
            },
            Component::Normal(_) => depth += 1,
            Component::CurDir => (),
        }
    }

    true
}

/// Install from a well structured mod directory
pub fn install(src: &Path, dest: &Path) -> Result<()> {
    let config_dir = ["BepInEx", "config"].into_iter().collect::<PathBuf>();
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
            std::fs::create_dir(target)
                .with_context(|| format!("failed to create directory {}", relative.display()))?;
        } else if relative.starts_with(&config_dir) {
            // copy config files since they're mutable
            std::fs::copy(entry.path(), target)
                .with_context(|| format!("failed to copy file {}", relative.display()))?;
        } else {
            std::fs::hard_link(entry.path(), target)
                .with_context(|| format!("failed to link file {}", relative.display()))?;
        }
    }

    Ok(())
}
