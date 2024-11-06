use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use anyhow::Result;

use crate::profile::Profile;

pub struct Installer;

impl Installer {
    pub fn map_file<'p>(&self, relative_path: &'p Path) -> Result<Option<Cow<'p, Path>>> {
        let mut components = relative_path.components();
        if components.clone().count() == 1 {
            // ignore top-level files, such as manifest.json and icon.png
            return Ok(None);
        }

        // remove the top-level dir (usually called BepInExPack)
        components.next();

        Ok(Some(Cow::Borrowed(components.as_path())))
    }

    pub fn scan_mod<F>(&self, profile: &Profile, scan: F) -> Result<()>
    where
        F: Fn(&Path) -> Result<()>,
    {
        let core_path: PathBuf = ["BepInEx", "core"].iter().collect();

        let files = profile
            .path
            .join(core_path)
            .read_dir()?
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_ok_and(|ty| ty.is_file()));

        for file in files {
            scan(&file.path())?;
        }

        Ok(())
    }
}
