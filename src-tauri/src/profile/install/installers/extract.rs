use std::{borrow::Cow, path::PathBuf};

use anyhow::Result;

use super::{ModArchive, PackageInstaller};
use crate::profile::{install, Profile, ProfileMod};

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

    fn scan_mod<'b>(&'b self, profile: &'b Profile) -> impl Iterator<Item = PathBuf> + 'b {
        self.files.iter().map(|file| profile.path.join(file))
    }
}

impl<'a> PackageInstaller for ExtractInstaller<'a> {
    fn extract(&mut self, archive: ModArchive, _package_name: &str, dest: PathBuf) -> Result<()> {
        install::fs::extract(archive, dest, |relative_path| {
            let mut components = relative_path.components();

            if self.flatten_top_level {
                components.next();
            }

            let path = components.as_path();

            Ok(if self.files.iter().any(|file| path.starts_with(file)) {
                Some(Cow::Borrowed(path))
            } else {
                None
            })
        })
    }

    fn toggle(
        &mut self,
        enabled: bool,
        _profile_mod: &ProfileMod,
        profile: &Profile,
    ) -> Result<()> {
        for path in self.scan_mod(profile) {
            install::fs::toggle_any(path, enabled)?;
        }

        Ok(())
    }

    fn uninstall(&mut self, _profile_mod: &ProfileMod, profile: &Profile) -> Result<()> {
        for path in self.scan_mod(profile) {
            install::fs::uninstall_any(path)?;
        }

        Ok(())
    }
}
