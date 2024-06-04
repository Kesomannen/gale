use std::{
    fs::{self, DirEntry, File},
    io::{self, Write},
    path::{Path, PathBuf},
};

use zip::{write::FileOptions, ZipWriter};
use serde::de::DeserializeOwned;

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

pub struct Zip {
    writer: ZipWriter<File>,
    options: FileOptions,
}

impl Zip {
    pub fn writer(&mut self, path: impl AsRef<Path>) -> io::Result<&mut ZipWriter<File>> {
        #[allow(deprecated)]
        self.writer.start_file_from_path(path.as_ref(), self.options)?;
        Ok(&mut self.writer)
    }

    pub fn write(&mut self, path: impl AsRef<Path>, data: &[u8]) -> io::Result<()> {
        self.writer(path)?.write_all(data)
    }

    pub fn write_str(&mut self, path: impl AsRef<Path>, data: &str) -> io::Result<()> {
        self.write(path, data.as_bytes())
    }
}

pub fn zip(path: &Path) -> Result<Zip, io::Error> {
    let writer = ZipWriter::new(File::create(path)?);

    let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    Ok(Zip { writer, options })
}

pub fn file_name(path: &Path) -> String {
    path.file_name().unwrap().to_string_lossy().to_string()
}
