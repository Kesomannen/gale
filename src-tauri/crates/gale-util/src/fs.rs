use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{self, BufReader, BufWriter, Read},
    path::{Path, PathBuf},
};

use serde::{de::DeserializeOwned, Serialize};
use tracing::warn;
use walkdir::WalkDir;
use zip::ZipArchive;

use super::error::IoResultExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overwrite {
    Yes,
    No,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UseLinks {
    Yes,
    No,
}

pub fn copy_dir(
    src: impl AsRef<Path>,
    dest: impl AsRef<Path>,
    overwrite: Overwrite,
    use_links: UseLinks,
) -> eyre::Result<()> {
    let dest = dest.as_ref();

    fs::create_dir_all(dest).fs_context("creating root directory", dest)?;
    copy_contents(src, dest, overwrite, use_links)
}

pub fn copy_contents(
    src: impl AsRef<Path>,
    dest: impl AsRef<Path>,
    overwrite: Overwrite,
    use_links: UseLinks,
) -> eyre::Result<()> {
    let src = src.as_ref();
    let dest = dest.as_ref();

    for entry in src.read_dir().fs_context("reading directory", src)? {
        let entry = entry?;
        let entry_path = entry.path();

        let Some(file_name) = entry_path.file_name() else {
            warn!(
                "skipping directory entry at {} because it has no name",
                entry_path.display()
            );
            continue;
        };

        let entry_type = entry
            .file_type()
            .fs_context("determining file type", &entry_path)?;

        let new_path = dest.join(file_name);

        if entry_type.is_dir() {
            if !new_path.exists() {
                fs::create_dir(&new_path).fs_context("creating directory", &new_path)?;
            }

            copy_contents(&entry_path, &new_path, overwrite, use_links)?;
        } else {
            if new_path.exists() && overwrite == Overwrite::No {
                continue;
            }

            match use_links {
                UseLinks::Yes => {
                    fs::hard_link(&entry_path, &new_path).fs_context("linking file", &new_path)?;
                }
                UseLinks::No => {
                    fs::copy(&entry_path, &new_path).fs_context("copying file", &new_path)?;
                }
            };
        }
    }

    Ok(())
}

pub fn get_directory_size(path: impl AsRef<Path>) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .map(|entry| match entry.file_type().is_file() {
            true => entry.metadata().map(|meta| meta.len()).unwrap_or(0),
            false => 0,
        })
        .sum()
}

pub fn read_json<T: DeserializeOwned>(path: impl AsRef<Path>) -> eyre::Result<T> {
    let string = fs::read_to_string(path)?;
    let result = serde_json::from_str(&string)?;

    Ok(result)
}

pub fn open_zip(path: impl AsRef<Path>) -> eyre::Result<ZipArchive<BufReader<File>>> {
    let reader = File::open(path).map(BufReader::new)?;
    let archive = ZipArchive::new(reader)?;

    Ok(archive)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonStyle {
    Pretty,
    Compact,
}

pub fn write_json<T: Serialize + ?Sized>(
    path: impl AsRef<Path>,
    value: &T,
    style: JsonStyle,
) -> eyre::Result<()> {
    let writer = File::create(path).map(BufWriter::new)?;

    if style == JsonStyle::Pretty {
        serde_json::to_writer_pretty(writer, value)?;
    } else {
        serde_json::to_writer(writer, value)?;
    }

    Ok(())
}

pub fn file_name_owned(path: impl AsRef<Path>) -> String {
    path.as_ref()
        .file_name()
        .expect("file should have name")
        .to_string_lossy()
        .into_owned()
}

pub fn is_enclosed(path: impl AsRef<Path>) -> bool {
    use std::path::Component;

    if path
        .as_ref()
        .as_os_str()
        .to_str()
        .is_some_and(|str| str.contains('\0'))
    {
        return false;
    }

    let mut depth = 0usize;
    for component in path.as_ref().components() {
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

pub trait PathExt: Sized {
    fn exists_or_none(self) -> Option<Self>;
    fn add_ext(&mut self, extension: impl AsRef<OsStr>);
}

impl PathExt for PathBuf {
    fn exists_or_none(self) -> Option<PathBuf> {
        match self.exists() {
            true => Some(self),
            false => None,
        }
    }

    fn add_ext(&mut self, extension: impl AsRef<OsStr>) {
        match self.extension() {
            Some(ext) => {
                let mut ext = ext.to_os_string();
                ext.push(".");
                ext.push(extension.as_ref());
                self.set_extension(ext)
            }
            None => self.set_extension(extension.as_ref()),
        };
    }
}

pub fn checksum(path: &Path) -> io::Result<blake3::Hash> {
    let mut file = File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 4096];

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    Ok(hasher.finalize())
}
