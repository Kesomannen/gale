use std::{borrow::Cow, path::Path};

use anyhow::Result;

use crate::profile::{Profile, ProfileMod};

mod bepinex;
mod extract;
mod gd_weave;
mod subdir;

pub use self::{
    bepinex::BepinexInstaller,
    extract::ExtractInstaller,
    gd_weave::GDWeaveModInstaller,
    subdir::{Subdir, SubdirInstaller},
};

pub type ScanFn = Box<dyn Fn(&Path) -> Result<()>>;

pub trait PackageInstaller {
    fn map_file<'p>(
        &mut self,
        relative_path: &'p Path,
        package_name: &str,
    ) -> Result<Option<Cow<'p, Path>>>;

    fn scan_mod(&mut self, profile_mod: &ProfileMod, profile: &Profile, scan: ScanFn)
        -> Result<()>;

    fn install_file(
        &mut self,
        _relative_path: &Path,
        _package_name: &str,
        _profile: &Profile,
    ) -> Result<FileInstallMethod> {
        Ok(FileInstallMethod::Link)
    }
}

#[derive(Debug)]
pub enum FileInstallMethod {
    Link,
    Copy,
}
