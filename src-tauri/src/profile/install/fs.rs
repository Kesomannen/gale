use std::{
    borrow::Cow,
    fs::{self, File},
    io::{self, BufReader, Read, Seek},
    path::{Path, PathBuf},
    time::Instant,
};

use anyhow::{Context, Result};
use log::{debug, warn};
use walkdir::WalkDir;
use zip::ZipArchive;

use crate::{game::ModLoader, prefs::Prefs, util};

use super::PackageInstaller;

pub fn install_from_zip(
    src: &Path,
    dest: &Path,
    full_name: &str,
    mod_loader: &ModLoader,
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

    let installer = mod_loader.installer(full_name);
    extract(reader, full_name, path.clone(), &installer)?;
    install(&path, dest, false, &installer)?;

    fs::remove_dir_all(path).context("failed to remove temporary directory")?;

    Ok(())
}

pub fn extract(
    src: impl Read + Seek,
    full_name: &str,
    dest: PathBuf,
    installer: &PackageInstaller,
) -> Result<()> {
    let start = Instant::now();

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

        let Some(relative_target) = installer.map_file(&relative_path, full_name)? else {
            continue;
        };

        let target_path = dest.join(relative_target);

        fs::create_dir_all(target_path.parent().unwrap())?;

        let mut target_file = File::create(&target_path)?;
        io::copy(&mut source_file, &mut target_file)?;
    }

    debug!("extracted {} in {:?}", full_name, start.elapsed());

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
pub fn install(
    src: &Path,
    dest: &Path,
    overwrite: bool,
    installer: &PackageInstaller,
) -> Result<()> {
    let entries = WalkDir::new(src).into_iter().filter_map(Result::ok);
    for entry in entries {
        let relative = entry
            .path()
            .strip_prefix(src)
            .expect("WalkDir should only return full paths inside of the root");

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

            /*
            let mutable = mod_loader
                .subdirs()
                .find(|subdir| relative.starts_with(subdir.target))
                .is_some_and(|subdir| subdir.mutable);
            */

            let mutable = false;

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
