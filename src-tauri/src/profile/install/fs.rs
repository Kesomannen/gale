use std::{
    borrow::Cow,
    ffi::OsStr,
    fs::{self, File},
    io::{self, BufReader, Read, Seek},
    path::{Path, PathBuf},
    time::Instant,
};

use anyhow::{Context, Result};
use log::{debug, warn};
use walkdir::WalkDir;
use zip::ZipArchive;

use crate::{
    game::{Game, Subdir},
    prefs::Prefs,
    util,
};

pub fn install_from_zip(
    src: &Path,
    dest: &Path,
    full_name: &str,
    game: Game,
    prefs: &Prefs,
) -> Result<()> {
    // temporarily extract the zip so the same install method can be used

    // dont use tempdir since we need the files on the same drive as the destination
    // for hard linking to work

    let path = prefs.data_dir.join("temp").join("extract");
    fs::create_dir_all(&path).context("failed to create temporary directory")?;

    let reader = File::open(src)
        .map(BufReader::new)
        .context("failed to open file")?;

    extract(reader, full_name, path.clone(), game)?;
    install(&path, dest, false, game)?;

    fs::remove_dir_all(path).context("failed to remove temporary directory")?;

    Ok(())
}

pub fn extract(src: impl Read + Seek, full_name: &str, dest: PathBuf, game: Game) -> Result<()> {
    let start = Instant::now();

    let is_bepinex = is_bepinex(full_name);
    let mut archive = ZipArchive::new(src)?;

    for i in 0..archive.len() {
        let mut source_file = archive.by_index(i)?;

        if source_file.is_dir() {
            continue; // we create the necessary dirs when copying files instead
        }

        let name = source_file.name();
        let relative_path: Cow<'_, Path> = if cfg!(unix) && name.contains('\\') {
            PathBuf::from(name.replace('\\', "/")).into()
        } else {
            Path::new(name).into()
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
            let path = map_file_default(&relative_path, full_name, game)?;

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

pub fn map_file_bepinex(relative_path: &Path) -> Option<&Path> {
    let mut components = relative_path.components();
    if components.clone().count() == 1 {
        // ignore top-level files, such as manifest.json and icon.png
        return None;
    }

    // remove the top-level dir (usually called BepInExPack)
    components.next();

    Some(components.as_path())
}

pub trait FileMapper {
    fn match_subdir(&self, name: &str) -> Option<&Subdir>;
    fn default_subdir(&self) -> &Subdir;
}

impl FileMapper for Game {
    fn match_subdir(&self, name: &str) -> Option<&Subdir> {
        self.subdirs().find(|subdir| subdir.name() == name)
    }

    fn default_subdir(&self) -> &Subdir {
        self.mod_loader().default_subdir()
    }
}

/// Maps a file from a mod zip archive to its final extracted path.
/// Based on r2modman's structure rules, which are available here:
/// https://github.com/ebkr/r2modmanPlus/wiki/Structuring-your-Thunderstore-package
pub fn map_file_default(
    relative_path: &Path,
    full_name: &str,
    mapper: impl FileMapper,
) -> Result<PathBuf> {
    use std::path::Component;

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
                    if let Some(subdir) = mapper.match_subdir(name) {
                        break (subdir, false);
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
            None => break (mapper.default_subdir(), true),
        }
    };

    // components now contains either:
    // - the remaining path if a subdir was found, or
    // - None if the file is top level and thus defaulted to plugins

    // prev is the canonical path leading up to a subdir,
    // or the whole path if we defaulted

    // e.g. profile/BepInEx/plugins
    let mut target = PathBuf::from(subdir.target());

    if subdir.separate_mods() {
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
pub fn install(src: &Path, dest: &Path, overwrite: bool, game: Game) -> Result<()> {
    let entries = WalkDir::new(src).into_iter().filter_map(Result::ok);
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

            let mutable = game
                .subdirs()
                .find(|subdir| relative.starts_with(subdir.target()))
                .is_some_and(|subdir| subdir.is_mutable());

            if mutable {
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
