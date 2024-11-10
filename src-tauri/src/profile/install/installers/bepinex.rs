use std::{
    borrow::Cow,
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;

use super::{FileInstallMethod, ModArchive, PackageInstaller};
use crate::profile::{
    install::{self, fs::ConflictResolution},
    Profile, ProfileMod,
};

pub struct BepinexInstaller;

fn scan(profile: &Profile) -> Result<impl Iterator<Item = PathBuf>> {
    let core_path: PathBuf = ["BepInEx", "core"].iter().collect();

    Ok(profile
        .path
        .join(core_path)
        .read_dir()?
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|ty| ty.is_file()))
        .map(|entry| entry.path()))
}

impl PackageInstaller for BepinexInstaller {
    fn extract(&mut self, archive: ModArchive, _package_name: &str, dest: PathBuf) -> Result<()> {
        install::fs::extract(archive, dest, |relative_path| {
            let mut components = relative_path.components();
            if components.clone().count() == 1 {
                // ignore top-level files, such as manifest.json and icon.png
                return Ok(None);
            }

            // remove the top-level dir (usually called BepInExPack)
            components.next();

            Ok(Some(Cow::Borrowed(components.as_path())))
        })
    }

    fn install(&mut self, src: &Path, _package_name: &str, profile: &Profile) -> Result<()> {
        install::fs::install(src, profile, |relative_path, _| {
            let method = if relative_path.extension().is_some_and(|ext| ext == "cfg") {
                FileInstallMethod::Copy
            } else {
                FileInstallMethod::Link
            };

            Ok((method, ConflictResolution::Skip))
        })
    }

    fn toggle(
        &mut self,
        enabled: bool,
        _profile_mod: &ProfileMod,
        profile: &Profile,
    ) -> Result<()> {
        for file in scan(profile)? {
            install::fs::toggle_file(file, enabled)?;
        }

        Ok(())
    }

    fn uninstall(&mut self, _profile_mod: &ProfileMod, profile: &Profile) -> Result<()> {
        for file in scan(profile)? {
            fs::remove_file(file)?;
        }

        Ok(())
    }
}
