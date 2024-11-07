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

use crate::{game::ModLoader, prefs::Prefs, profile::Profile, util};

use super::{FileInstallMethod, PackageInstaller};

pub fn install_from_zip(
    src: &Path,
    profile: &Profile,
    package_name: &str,
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

    let mut installer = mod_loader.installer(package_name);
    extract(reader, package_name, path.clone(), &mut *installer)?;
    install(&path, profile, false, &mut *installer)?;

    fs::remove_dir_all(path).context("failed to remove temporary directory")?;

    Ok(())
}

pub fn extract(
    src: impl Read + Seek,
    package_name: &str,
    dest: PathBuf,
    installer: &mut dyn PackageInstaller,
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

        let Some(relative_target) = installer.map_file(&relative_path, package_name)? else {
            continue;
        };

        let target_path = dest.join(relative_target);

        fs::create_dir_all(target_path.parent().unwrap())?;

        let mut target_file = File::create(&target_path)?;
        io::copy(&mut source_file, &mut target_file)?;
    }

    debug!("extracted {} in {:?}", package_name, start.elapsed());

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
    profile: &Profile,
    overwrite: bool,
    installer: &mut dyn PackageInstaller,
) -> Result<()> {
    for entry in WalkDir::new(src) {
        let entry = entry?;

        let relative = entry
            .path()
            .strip_prefix(src)
            .expect("WalkDir should only return full paths inside of the root");

        let target = profile.path.join(relative);
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

            let mode = installer.install_method(relative, &profile)?;

            match mode {
                FileInstallMethod::Link => {
                    fs::copy(entry.path(), target)
                        .with_context(|| format!("failed to copy file {}", relative.display()))?;
                }
                FileInstallMethod::Copy => {
                    fs::hard_link(entry.path(), target)
                        .with_context(|| format!("failed to link file {}", relative.display()))?;
                }
            }
        }
    }

    Ok(())
}
