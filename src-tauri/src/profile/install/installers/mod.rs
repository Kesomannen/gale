use std::{borrow::Cow, path::Path};

use anyhow::Result;

use crate::profile::{Profile, ProfileMod};

mod bepinex;
mod melon_loader;
mod rule;
pub use rule::Subdir;

pub enum PackageInstaller<'a> {
    Rule(rule::Installer<'a>),
    BepInEx(bepinex::Installer),
    MelonLoader(melon_loader::Installer),
}

impl<'a> PackageInstaller<'a> {
    pub fn rule(
        subdirs: &'a [Subdir<'a>],
        default_subdir: Option<&'a Subdir<'a>>,
        ignored_files: &'a [&'a str],
    ) -> Self {
        Self::Rule(rule::Installer::new(subdirs, default_subdir, ignored_files))
    }

    pub fn bepinex() -> Self {
        Self::BepInEx(bepinex::Installer)
    }

    pub fn melon_loader() -> Self {
        Self::MelonLoader(melon_loader::Installer)
    }

    pub fn map_file<'p>(
        &self,
        relative_path: &'p Path,
        package_name: &str,
    ) -> Result<Option<Cow<'p, Path>>> {
        match self {
            PackageInstaller::Rule(i) => i.map_file(relative_path, package_name),
            PackageInstaller::BepInEx(i) => i.map_file(relative_path),
            PackageInstaller::MelonLoader(i) => i.map_file(relative_path),
        }
    }

    pub fn scan_mod<F>(&self, profile_mod: &ProfileMod, profile: &Profile, scan: F) -> Result<()>
    where
        F: Fn(&Path) -> Result<()>,
    {
        match self {
            PackageInstaller::Rule(i) => i.scan_mod(profile_mod, profile, scan),
            PackageInstaller::BepInEx(i) => i.scan_mod(profile, scan),
            PackageInstaller::MelonLoader(i) => i.scan_mod(profile_mod, profile, scan),
        }
    }
}
