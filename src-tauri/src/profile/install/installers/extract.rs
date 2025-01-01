use std::{borrow::Cow, path::PathBuf};

use eyre::Result;

use super::{PackageInstaller, PackageZip};
use crate::profile::{install, Profile, ProfileMod};

pub enum FlattenTopLevel {
    Yes,
    No,
}

/// Installs packages with normal zip extraction rules.
pub struct ExtractInstaller<'a> {
    /// The files/directories to copy over.
    /// Any paths that start with any of these will match.
    files: &'a [&'a str],
    /// Whether to remove any top-level directories in the zip.
    flatten_top_level: FlattenTopLevel,
}

impl<'a> ExtractInstaller<'a> {
    pub fn new(files: &'a [&'a str], flatten_top_level: FlattenTopLevel) -> Self {
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
    fn extract(&mut self, archive: PackageZip, _package_name: &str, dest: PathBuf) -> Result<()> {
        install::fs::extract(archive, dest, |relative_path| {
            let mut components = relative_path.components();

            if matches!(self.flatten_top_level, FlattenTopLevel::Yes) {
                components.next();
            }

            let path = components.as_path();

            Ok(self
                .files
                .iter()
                .any(|file| path.starts_with(file))
                .then(|| Cow::Borrowed(path)))
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
