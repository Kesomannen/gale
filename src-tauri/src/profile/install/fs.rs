use std::{
    borrow::Cow,
    fs::{self, File},
    io::{self, Cursor, Read, Seek},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use log::warn;
use walkdir::WalkDir;
use zip::ZipArchive;

use crate::{
    game::ModLoader,
    prefs::Prefs,
    profile::Profile,
    util::{self, fs::PathExt},
};

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

    let temp_path = prefs.data_dir.join("temp").join("extract");
    fs::create_dir_all(&temp_path).context("failed to create temporary directory")?;

    let reader = fs::read(src)
        .map(Cursor::new)
        .context("failed to read file")?;
    let archive = ZipArchive::new(reader)?;

    let mut installer = mod_loader.installer_for(package_name);
    installer.extract(archive, package_name, temp_path.clone())?;
    installer.install(&temp_path, package_name, false, profile);

    fs::remove_dir_all(temp_path).context("failed to remove temporary directory")?;

    Ok(())
}

pub(super) fn extract<S, M>(
    mut archive: ZipArchive<S>,
    dest: PathBuf,
    mut map_file: M,
) -> Result<()>
where
    S: Read + Seek,
    M: FnMut(&Path) -> Result<Option<Cow<Path>>>,
{
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

        let Some(relative_target) = map_file(&relative_path)? else {
            continue;
        };

        let target_path = dest.join(relative_target);

        fs::create_dir_all(target_path.parent().unwrap())?;

        let mut target_file = File::create(&target_path)?;
        io::copy(&mut source_file, &mut target_file)?;
    }

    Ok(())
}

#[derive(Debug)]
pub enum FileInstallMethod {
    Link,
    Copy,
}

#[derive(Debug)]
pub enum ConflictResolution {
    Skip,
    Overwrite,
}

impl ConflictResolution {
    pub fn overwrite(yes: bool) -> Self {
        if yes {
            ConflictResolution::Overwrite
        } else {
            ConflictResolution::Skip
        }
    }
}

/// Install from a well structured mod directory.
pub(super) fn install<F, G>(
    src: &Path,
    profile: &Profile,
    mut method: F,
    mut on_conflict: G,
) -> Result<()>
where
    F: FnMut(&Path) -> Result<FileInstallMethod>,
    G: FnMut(&Path) -> ConflictResolution,
{
    for entry in WalkDir::new(src) {
        let entry = entry?;

        let relative_path = entry
            .path()
            .strip_prefix(src)
            .expect("WalkDir should only return full paths inside of the root");

        let target = profile.path.join(relative_path);
        if entry.file_type().is_dir() {
            if target.exists() {
                continue;
            }

            fs::create_dir(target).with_context(|| {
                format!("failed to create directory {}", relative_path.display())
            })?;
        } else {
            if target.exists() {
                match on_conflict(relative_path) {
                    ConflictResolution::Skip => continue,
                    ConflictResolution::Overwrite => {
                        fs::remove_file(&target).with_context(|| {
                            format!(
                                "failed to remove existing file at {}",
                                relative_path.display()
                            )
                        })?;
                    }
                }
            }

            match method(relative_path)? {
                FileInstallMethod::Link => {
                    fs::copy(entry.path(), target).with_context(|| {
                        format!("failed to copy file at {}", relative_path.display())
                    })?;
                }
                FileInstallMethod::Copy => {
                    fs::hard_link(entry.path(), target).with_context(|| {
                        format!("failed to link file at {}", relative_path.display())
                    })?;
                }
            }
        }
    }

    Ok(())
}

pub(super) fn uninstall_any(path: impl AsRef<Path>) -> Result<()> {
    for_any(
        path.as_ref(),
        |path| fs::remove_dir_all(path).map_err(|err| err.into()),
        |path| fs::remove_file(path).map_err(|err| err.into()),
    )
}

pub(super) fn toggle_any(path: impl AsRef<Path>, enabled: bool) -> Result<()> {
    for_any(
        path.as_ref(),
        |path| toggle_dir(path, enabled),
        |path| toggle_file(path, enabled),
    )
}

fn for_any<F, G>(path: &Path, for_dir: F, for_file: G) -> Result<()>
where
    F: FnOnce(&Path) -> Result<()>,
    G: FnOnce(&Path) -> Result<()>,
{
    if let Ok(metadata) = path.metadata() {
        if metadata.is_dir() {
            for_dir(path)
        } else {
            for_file(path)
        }
    } else {
        let mut path = path.to_path_buf();
        path.add_ext("old");

        if path.exists() {
            for_file(&path)
        } else {
            Ok(())
        }
    }
}

pub(super) fn toggle_file(path: impl AsRef<Path>, enabled: bool) -> Result<()> {
    let mut new_path = path.as_ref().to_path_buf();

    if enabled {
        new_path.add_ext("old");
    } else {
        // remove all old extensions if multiple got added somehow
        while let Some("old") = new_path.extension().and_then(|ext| ext.to_str()) {
            new_path.set_extension("");
        }
    }

    fs::rename(path, &new_path)?;

    Ok(())
}

pub(super) fn toggle_dir(path: impl AsRef<Path>, enabled: bool) -> Result<()> {
    let files = WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| {
            let file_type = entry.file_type();
            file_type.is_file() || file_type.is_symlink()
        });

    for file in files {
        toggle_file(file.path(), enabled)?;
    }

    Ok(())
}