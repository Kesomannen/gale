use log::{debug, warn};
use std::{
    fs::{self, File},
    io::{self, Read, Seek, Write},
    path::{Path, PathBuf},
};
use zip::ZipArchive;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use crate::util;

pub trait ZipWriterExt {
    fn write_str<S: Into<String>>(&mut self, name: S, data: &str) -> io::Result<()>;
}

impl<W: Write + Seek> ZipWriterExt for zip::ZipWriter<W> {
    fn write_str<S: Into<String>>(&mut self, name: S, data: &str) -> io::Result<()> {
        self.start_file(name, Default::default())?;
        self.write_all(data.as_bytes())?;
        Ok(())
    }
}

pub fn extract(src: impl Read + Seek, target: &Path) -> io::Result<()> {
    fs::create_dir_all(target)?;

    let mut archive = ZipArchive::new(src)?;

    debug!("extracting to {}", target.display());
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;

        if file.is_dir() {
            continue; // we create the necessary dirs when copying files instead
        }

        let relative = match cfg!(unix) {
            true => PathBuf::from(file.name().replace('\\', "/")),
            false => PathBuf::from(file.name()),
        };

        if !util::fs::is_enclosed(&relative) {
            warn!("file {} escapes the archive root, skipping", relative.display());
            continue;
        }

        let output_path = target.join(relative);

        fs::create_dir_all(output_path.parent().unwrap())?;
        let mut out = File::create(&output_path)?;
        io::copy(&mut file, &mut out)?;

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
