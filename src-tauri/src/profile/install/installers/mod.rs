use std::{
    io::Cursor,
    path::{Path, PathBuf},
};

use eyre::Result;
use zip::ZipArchive;

use super::fs::{ConflictResolution, FileInstallMethod};
use crate::profile::{Profile, ProfileMod};

mod bepinex;
mod extract;
mod gd_weave;
mod shimloader;
mod subdir;

pub use self::{
    bepinex::BepinexInstaller,
    extract::{ExtractInstaller, FlattenTopLevel},
    gd_weave::GDWeaveModInstaller,
    shimloader::ShimloaderInstaller,
    subdir::{Subdir, SubdirInstaller, restore_package_state},
};

pub type PackageZip = ZipArchive<Cursor<Vec<u8>>>;

pub trait PackageInstaller {
    fn extract(&mut self, archive: PackageZip, package_name: &str, dest: PathBuf) -> Result<()>;

    fn install(&mut self, src: &Path, _package_name: &str, profile: &Profile) -> Result<()> {
        super::fs::install(src, profile, |_, _| {
            Ok((FileInstallMethod::Link, ConflictResolution::Overwrite))
        })
    }

    fn toggle(&mut self, enabled: bool, profile_mod: &ProfileMod, profile: &Profile) -> Result<()>;
    fn uninstall(&mut self, profile_mod: &ProfileMod, profile: &Profile) -> Result<()>;

    /// Absolute paths inside the profile that are owned by this mod (dirs or files).
    ///
    /// This enumerates the same paths that `uninstall` would remove. Disabled
    /// (`.old`) variants of file entries are handled by the caller.
    fn installed_paths(&self, profile_mod: &ProfileMod, profile: &Profile) -> Result<Vec<PathBuf>> {
        Ok(self
            .mod_dir(&profile_mod.full_name(), profile)
            .into_iter()
            .collect())
    }

    fn mod_dir(&self, _package_name: &str, _profile: &Profile) -> Option<PathBuf> {
        None
    }
}
