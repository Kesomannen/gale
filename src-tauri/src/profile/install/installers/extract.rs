use std::{borrow::Cow, path::Path};

use anyhow::Result;

use super::{PackageInstaller, ScanFn};
use crate::profile::{Profile, ProfileMod};

pub struct ExtractInstaller<'a> {
    files: &'a [&'a str],
    flatten_top_level: bool,
}

impl<'a> ExtractInstaller<'a> {
    pub fn new(files: &'a [&'a str], flatten_top_level: bool) -> Self {
        Self {
            files,
            flatten_top_level,
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

        if self.flatten_top_level {
            components.next();
        }

        let path = components.as_path();

        if self.files.iter().any(|file| path.starts_with(file)) {
            Ok(Some(Cow::Borrowed(path)))
        } else {
            Ok(None)
        }
    }

    fn scan_mod(
        &mut self,
        _profile_mod: &ProfileMod,
        profile: &Profile,
        scan: ScanFn,
    ) -> Result<()> {
        for file in self.files {
            scan(&profile.path.join(file))?;
        }

        Ok(())
    }
}
