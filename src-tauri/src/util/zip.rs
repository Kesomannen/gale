use log::debug;
use std::{
    fs::{self, File},
    io::{self, Read, Seek, Write},
    path::Path,
};
use zip::{write::FileOptions, ZipArchive, ZipWriter};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
pub struct ZipBuilder {
    writer: ZipWriter<File>,
    options: FileOptions,
}

impl ZipBuilder {
    pub fn writer(&mut self, path: impl AsRef<Path>) -> io::Result<&mut ZipWriter<File>> {
        #[allow(deprecated)]
        self.writer
            .start_file_from_path(path.as_ref(), self.options)?;
        Ok(&mut self.writer)
    }

    pub fn write(&mut self, path: impl AsRef<Path>, data: &[u8]) -> io::Result<()> {
        self.writer(path)?.write_all(data)
    }

    pub fn write_str(&mut self, path: impl AsRef<Path>, data: &str) -> io::Result<()> {
        self.write(path, data.as_bytes())
    }
}

pub fn builder(path: &Path) -> Result<ZipBuilder, io::Error> {
    let writer = ZipWriter::new(File::create(path)?);

    let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    Ok(ZipBuilder { writer, options })
}

pub fn extract(src: impl Read + Seek, target: &Path) -> io::Result<()> {
    if !target.exists() {
        fs::create_dir_all(target)?;
    }

    let mut archive = ZipArchive::new(src)?;

    debug!("extracting to {}", target.display());
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let relative = file.mangled_name();

        if relative.as_os_str().is_empty() {
            continue;
        }

        let output_path = target.join(relative);

        if file.name().ends_with(['/', '\\']) {
            fs::create_dir_all(&output_path)?;
        } else {
            if let Some(parent) = output_path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }
            let mut out = File::create(&output_path)?;
            io::copy(&mut file, &mut out)?;
        }

        #[cfg(unix)]
        set_unix_mode(&file, &output_path)?;
    }

    Ok(())
}

#[cfg(unix)]
fn set_unix_mode(file: &zip::read::ZipFile, path: &Path) -> io::Result<()> {
    if let Some(mode) = file.unix_mode() {
        fs::set_permissions(&path, PermissionsExt::from_mode(mode))?
    }
    Ok(())
}
