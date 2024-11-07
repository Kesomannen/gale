use std::{
    borrow::Cow,
    path::{Component, Path},
};

use anyhow::{bail, Result};

use super::{FileInstallMethod, PackageInstaller, ScanFn};
use crate::profile::{Profile, ProfileMod};

pub struct ExtractInstaller<'a> {
    dir_name: &'a str,
    dirs: &'a [&'a str],
    top_level_files: &'a [&'a str],
}

impl<'a> ExtractInstaller<'a> {
    pub fn new(dir_name: &'a str, dirs: &'a [&'a str], top_level_files: &'a [&'a str]) -> Self {
        Self {
            dir_name,
            dirs,
            top_level_files,
        }
    }
}

impl<'a> PackageInstaller for ExtractInstaller<'a> {
    fn map_file<'p>(
        &mut self,
        relative_path: &'p Path,
        _package_name: &str,
    ) -> Result<Option<Cow<'p, Path>>> {
        let mut components = relative_path.components();

        let first = match components.next() {
            Some(Component::Normal(name)) => name,
            _ => bail!("malformed package"),
        };

        if components.next().is_none() {
            // top level file
            if !self.top_level_files.iter().any(|file| *file == first) {
                return Ok(None);
            }
        }

        Ok(Some(Cow::Borrowed(relative_path)))
    }

    fn scan_mod(
        &mut self,
        _profile_mod: &ProfileMod,
        profile: &Profile,
        scan: ScanFn,
    ) -> Result<()> {
        let mut path = profile.path.to_path_buf();

        for file in self.top_level_files {
            path.push(file);
            scan(&path)?;
            path.pop();
        }
        path.push(self.dir_name);
        for dir in self.dirs {
            path.push(dir);
            scan(&path)?;
            path.pop();
        }
        path.pop();

        Ok(())
    }

    fn install_file(
        &mut self,
        _relative_path: &Path,
        _profile: &Profile,
    ) -> Result<FileInstallMethod> {
        Ok(FileInstallMethod::Link)
    }
}
