use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{self, BufReader, BufWriter},
    path::{Path, PathBuf},
};

use serde::{de::DeserializeOwned, Serialize};
use walkdir::WalkDir;
use zip::ZipArchive;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overwrite {
    Yes,
    No,
}

pub fn copy_dir(src: &Path, dest: &Path, overwrite: Overwrite) -> io::Result<()> {
    fs::create_dir_all(dest)?;
    let entries = WalkDir::new(src);

    for entry in entries {
        let entry = entry?;

        let relative = entry.path().strip_prefix(src).unwrap();
        let new_path = src.join(relative);

        if entry.file_type().is_dir() {
            if !new_path.exists() {
                fs::create_dir(new_path)?;
            }
        } else {
            if new_path.exists() && overwrite == Overwrite::No {
                continue;
            }

            fs::copy(entry.path(), new_path)?;
        }
    }

    Ok(())
}

pub fn read_json<T: DeserializeOwned>(path: impl AsRef<Path>) -> anyhow::Result<T> {
    let reader = File::open(path).map(BufReader::new)?;
    let result = serde_json::from_reader(reader)?;

    Ok(result)
}

pub fn open_zip(path: impl AsRef<Path>) -> anyhow::Result<ZipArchive<BufReader<File>>> {
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
) -> anyhow::Result<()> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);

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
