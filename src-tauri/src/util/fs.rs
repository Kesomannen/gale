use std::{
    fs::{self, DirEntry},
    io::{self},
    path::{Path, PathBuf},
};

use serde::{de::DeserializeOwned, Serialize};

pub fn flatten_if_exists(path: &Path) -> Result<bool, io::Error> {
    if !path.try_exists()? {
        return Ok(false);
    }

    let parent = path.parent().unwrap();

    for entry in fs::read_dir(path)? {
        let entry_path = entry?.path();
        let file_name = entry_path.file_name().unwrap();
        let new_path = parent.join(file_name);

        fs::rename(entry_path, new_path)?;
    }

    fs::remove_dir(path)?;

    Ok(true)
}

pub fn copy_dir(src: &Path, dest: &Path, overwrite: bool) -> Result<(), io::Error> {
    fs::create_dir_all(dest)?;
    copy_contents(src, dest, overwrite)?;
    Ok(())
}

pub fn copy_contents(src: &Path, dest: &Path, overwrite: bool) -> Result<(), io::Error> {
    for entry in read_dir(src)? {
        let entry_path = entry.path();
        let file_name = entry_path.file_name().unwrap();
        let new_path = dest.join(file_name);

        if new_path.exists() && !overwrite {
            continue;
        }

        if entry_path.is_dir() {
            fs::create_dir(&new_path)?;
            copy_contents(&entry_path, &new_path, overwrite)?;
        } else {
            fs::copy(&entry_path, &new_path)?;
        }
    }

    Ok(())
}

pub fn read_dir(path: &Path) -> io::Result<impl Iterator<Item = DirEntry>> {
    fs::read_dir(path).map(|entries| entries.filter_map(Result::ok))
}

pub fn read_json<T: DeserializeOwned>(path: &Path) -> anyhow::Result<T> {
    let file = fs::File::open(path)?;
    let reader = io::BufReader::new(file);
    let result = serde_json::from_reader(reader)?;

    Ok(result)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    Pretty,
    Compact,
}

pub fn write_json<T: Serialize + ?Sized>(path: &Path, value: &T, style: Style) -> anyhow::Result<()> {
    let file = fs::File::create(path)?;
    let writer = io::BufWriter::new(file);

    if style == Style::Pretty {
        serde_json::to_writer_pretty(writer, value)?;
    } else {
        serde_json::to_writer(writer, value)?;
    }

    Ok(())
}

pub fn add_extension(path: &mut PathBuf, extension: impl AsRef<Path>) {
    match path.extension() {
        Some(ext) => {
            let mut ext = ext.to_os_string();
            ext.push(".");
            ext.push(extension.as_ref());
            path.set_extension(ext)
        }
        None => path.set_extension(extension.as_ref()),
    };
}

pub fn file_name(path: &Path) -> String {
    path.file_name().unwrap().to_string_lossy().to_string()
}
