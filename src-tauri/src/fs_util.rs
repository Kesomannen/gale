use std::{fs::{self, File}, io::{self, Write}, path::{Path, PathBuf}};

use zip::{write::FileOptions, ZipWriter};

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

pub fn copy_dir(src: &Path, dest: &Path) -> Result<(), io::Error> {
    fs::create_dir_all(dest)?;
    copy_contents(src, dest, true)?;
    Ok(())
}

pub fn copy_contents(src: &Path, dest: &Path, overwrite: bool) -> Result<(), io::Error> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
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

pub fn add_extension<P: AsRef<Path>>(path: &mut PathBuf, extension: P) {
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
    pub fn write<P: AsRef<Path>>(&mut self, path: P, data: &[u8]) -> Result<(), io::Error> {
        #[allow(deprecated)]
        self.writer.start_file_from_path(path.as_ref(), self.options)?;
        self.writer.write_all(data)?;
        Ok(())
    }

    pub fn write_str<P: AsRef<Path>>(&mut self, path: P, data: &str) -> Result<(), io::Error> {
        self.write(path.as_ref(), data.as_bytes())
    }

    pub fn add_dir<P: AsRef<Path>>(&mut self, path: P) -> Result<(), io::Error> {
        #[allow(deprecated)]
        self.writer.add_directory_from_path(path.as_ref(), self.options)?;
        Ok(())
    }
}

pub fn zip(path: &Path) -> Result<Zip, io::Error> {
    let writer = ZipWriter::new(File::create(path)?);

    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Stored);

    Ok(Zip { writer, options })
}

pub fn file_name(path: &Path) -> String {
    path.file_name().unwrap().to_string_lossy().to_string()
}
