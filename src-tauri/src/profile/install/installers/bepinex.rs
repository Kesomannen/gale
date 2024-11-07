use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use anyhow::Result;

use super::{FileInstallMethod, PackageInstaller, ScanFn};
use crate::profile::{Profile, ProfileMod};

pub struct BepinexInstaller;

impl PackageInstaller for BepinexInstaller {
    fn map_file<'p>(
        &mut self,
        relative_path: &'p Path,
        _package_name: &str,
    ) -> Result<Option<Cow<'p, Path>>> {
        let mut components = relative_path.components();
        if components.clone().count() == 1 {
            // ignore top-level files, such as manifest.json and icon.png
            return Ok(None);
        }

        // remove the top-level dir (usually called BepInExPack)
        components.next();

        Ok(Some(Cow::Borrowed(components.as_path())))
    }

    fn scan_mod(
        &mut self,
        _profile_mod: &ProfileMod,
        profile: &Profile,
        scan: ScanFn,
    ) -> Result<()> {
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

    fn install_method(
        &mut self,
        relative_path: &Path,
        _profile: &Profile,
    ) -> Result<FileInstallMethod> {
        if relative_path
            .file_name()
            .is_some_and(|name| name == "BepInex.cfg")
        {
            Ok(FileInstallMethod::Copy)
        } else {
            Ok(FileInstallMethod::Link)
        }
    }
}
