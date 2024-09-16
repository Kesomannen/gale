use std::{
    ffi::OsStr,
    io::{Read, Seek},
    path::{Path, PathBuf},
};
use zip::ZipArchive;

pub trait PathExt {
    fn exists_or_none(self) -> Option<PathBuf>;
    fn add_extension(&mut self, extension: impl AsRef<OsStr>);
}

impl PathExt for PathBuf {
    fn exists_or_none(self) -> Option<PathBuf> {
        match self.exists() {
            true => Some(self),
            false => None,
        }
    }

    fn add_extension(&mut self, extension: impl AsRef<OsStr>) {
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

pub fn extract(src: impl Read + Seek, target: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(target)?;

    let mut archive = ZipArchive::new(src)?;

    log::debug!("extracting to {}", target.display());
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;

        if file.is_dir() {
            continue; // we create the necessary dirs when copying files instead
        }

        let relative = match cfg!(unix) {
            true => PathBuf::from(file.name().replace('\\', "/")),
            false => PathBuf::from(file.name()),
        };

        if !is_enclosed(&relative) {
            log::warn!(
                "file {} escapes the archive root, skipping",
                relative.display()
            );
            continue;
        }

        let output_path = target.join(relative);

        std::fs::create_dir_all(output_path.parent().unwrap())?;
        let mut out = std::fs::File::create(&output_path)?;
        std::io::copy(&mut file, &mut out)?;

        #[cfg(unix)]
        set_unix_mode(&file, &output_path)?;
    }

    Ok(())
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
