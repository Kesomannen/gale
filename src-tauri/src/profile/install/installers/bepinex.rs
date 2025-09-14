use std::{
    borrow::Cow,
    fs,
    path::{Path, PathBuf},
};

use eyre::Result;

use super::{FileInstallMethod, PackageInstaller, PackageZip};
use crate::profile::{
    install::{self, fs::ConflictResolution},
    Profile, ProfileMod,
};

pub struct BepinexInstaller;

fn get_core_path(package_name: &str) -> PathBuf {
    const CORE_PATH: &str = "BepInEx/core";
    match package_name {
        "ResoniteModding-BepInExRenderer" => PathBuf::from("Renderer").join(CORE_PATH),
        _ => PathBuf::from(CORE_PATH),
    }
}

fn scan(profile: &Profile, package_name: &str) -> Result<impl Iterator<Item = PathBuf>> {
    Ok(profile
        .path
        .join(get_core_path(package_name))
        .read_dir()?
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|ty| ty.is_file()))
        .map(|entry| entry.path()))
}

impl PackageInstaller for BepinexInstaller {
    fn extract(&mut self, archive: PackageZip, _package_name: &str, dest: PathBuf) -> Result<()> {
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
            if relative_path.extension().is_some_and(|ext| ext == "cfg") {
                Ok((FileInstallMethod::Copy, ConflictResolution::Skip))
            } else {
                Ok((FileInstallMethod::Link, ConflictResolution::Overwrite))
            }
        })
    }

    fn toggle(&mut self, enabled: bool, profile_mod: &ProfileMod, profile: &Profile) -> Result<()> {
        for file in scan(profile, &profile_mod.full_name())? {
            install::fs::toggle_file(file, enabled)?;
        }

        Ok(())
    }

    fn uninstall(&mut self, profile_mod: &ProfileMod, profile: &Profile) -> Result<()> {
        for file in scan(profile, &profile_mod.full_name())? {
            fs::remove_file(file)?;
        }

        Ok(())
    }

    fn mod_dir(&self, package_name: &str, profile: &Profile) -> Option<PathBuf> {
        Some(profile.path.join(get_core_path(package_name)))
    }
}
