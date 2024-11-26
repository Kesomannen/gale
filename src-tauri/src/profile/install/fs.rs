use std::{
    borrow::Cow,
    fs::{self, File},
    io::{self, Read, Seek},
    path::{Path, PathBuf},
};

use eyre::{Context, Result};
use log::warn;
use walkdir::WalkDir;
use zip::ZipArchive;

use crate::{
    profile::Profile,
    util::{self, error::IoResultExt, fs::PathExt},
};

/// Extract a package archive to `dest`, mapping files using `map_file`.
///
/// `map_file` is called with each file's relative path. It should return
/// the (relative) output path where the file should be copied. `Ok(None)`
/// skips the file entirely.
///
/// Directories are created as needed.
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

#[derive(Debug, Clone, Copy)]
pub enum FileInstallMethod {
    /// Use a hard link.
    Link,
    /// Copy the file.
    Copy,
}

#[derive(Debug, Clone, Copy)]
pub enum ConflictResolution {
    /// Do nothing, keeping the current file.
    Skip,
    /// Overwrite the current file.
    Overwrite,
}

/// Install from a well structured mod directory.
///
/// This essentially copies `src` to the profile directory.
///
/// `before_install` is called each time a file is encountered,
/// with the file's relative path and whether the target file already exists.
pub(super) fn install<F>(src: &Path, profile: &Profile, mut before_install: F) -> Result<()>
where
    F: FnMut(&Path, bool) -> Result<(FileInstallMethod, ConflictResolution)>,
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
            let target_exists = target.exists();
            let (method, conflict) = before_install(relative_path, target_exists)?;

            if target_exists {
                match (conflict, method) {
                    (ConflictResolution::Skip, _) => {
                        warn!(
                            "skipping file {} since it already exists",
                            relative_path.display()
                        );
                        continue;
                    }
                    // fs::copy already overwrites the target, no need to remove it
                    (ConflictResolution::Overwrite, FileInstallMethod::Copy) => (),
                    (ConflictResolution::Overwrite, FileInstallMethod::Link) => {
                        fs::remove_file(&target).with_context(|| {
                            format!(
                                "failed to remove existing file at {}",
                                relative_path.display()
                            )
                        })?;
                    }
                }
            }

            match method {
                FileInstallMethod::Link => {
                    fs::hard_link(entry.path(), target).with_context(|| {
                        format!("failed to link file at {}", relative_path.display())
                    })?;
                }
                FileInstallMethod::Copy => {
                    fs::copy(entry.path(), target).with_context(|| {
                        format!("failed to copy file at {}", relative_path.display())
                    })?;
                }
            }
        }
    }

    Ok(())
}

/// Removes either a directory or file at `path`. Also accounts for any
/// `.old` extensions that may exist.
pub(super) fn uninstall_any(path: impl AsRef<Path>) -> Result<()> {
    for_any(
        path.as_ref(),
        |path| fs::remove_dir_all(path).map_err(|err| err.into()),
        |path| fs::remove_file(path).map_err(|err| err.into()),
    )
}

/// Toggles either a directory or file at `path`.
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

/// Toggles a file by adding/removing a `.old` extension to it.
pub(super) fn toggle_file(path: impl AsRef<Path>, enabled: bool) -> Result<()> {
    let path = path.as_ref();
    let mut new_path = path.to_path_buf();

    if enabled {
        new_path.add_ext("old");
    } else {
        // remove all old extensions if multiple got added somehow
        while let Some("old") = new_path.extension().and_then(|ext| ext.to_str()) {
            new_path.set_extension("");
        }
    }

    fs::rename(path, &new_path).fs_context("renaming file", path)?;

    Ok(())
}

/// Toggles a directory by recursively adding/removing a `.old` extension to all files within it.
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
