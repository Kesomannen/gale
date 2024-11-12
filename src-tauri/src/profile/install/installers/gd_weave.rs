use std::{borrow::Cow, fs, path::PathBuf};

use anyhow::{ensure, Context, Result};

use super::{PackageInstaller, PackageZip};
use crate::profile::{
    install::{self},
    Profile, ProfileMod,
};

pub struct GDWeaveModInstaller;

fn relative_mod_dir(package_name: &str) -> PathBuf {
    ["GDWeave", "mods", package_name].iter().collect()
}

fn profile_mod_dir(profile_mod: &ProfileMod, profile: &Profile) -> PathBuf {
    let ident = profile_mod.ident();
    let package_name = ident.full_name();
    profile.path.join(relative_mod_dir(package_name))
}

impl PackageInstaller for GDWeaveModInstaller {
    fn extract(
        &mut self,
        mut archive: PackageZip,
        package_name: &str,
        dest: PathBuf,
    ) -> Result<()> {
        // find a directory with a manifest.json file in it
        // except the top level one since that has thunderstore's manifest

        let mut dirs = Vec::new();
        let mut manifests = Vec::new();

        for i in 0..archive.len() {
            let file = archive.by_index(i)?;

            if file.is_dir() {
                dirs.push(PathBuf::from(file.name()));
            } else if file.name().ends_with("manifest.json") {
                manifests.push(PathBuf::from(file.name()));
            }
        }

        let mut mod_roots = manifests.into_iter().filter_map(|path| {
            let parent = path.parent()?;
            dirs.iter().find(|dir| *dir == parent)
        });

        let mod_root = mod_roots
            .next()
            .context("malformed mod archive: no mod root found")?;

        ensure!(
            mod_roots.next().is_none(),
            "malformed mod archive: multiple mod roots found"
        );

        install::fs::extract(archive, dest, |relative_path| {
            if let Ok(relative_to_root) = relative_path.strip_prefix(mod_root) {
                let mut path = relative_mod_dir(package_name);
                path.push(relative_to_root);

                Ok(Some(Cow::Owned(path)))
            } else {
                Ok(None)
            }
        })
    }

    fn toggle(&mut self, enabled: bool, profile_mod: &ProfileMod, profile: &Profile) -> Result<()> {
        install::fs::toggle_dir(profile_mod_dir(profile_mod, profile), enabled)
    }

    fn uninstall(&mut self, profile_mod: &ProfileMod, profile: &Profile) -> Result<()> {
        fs::remove_dir_all(profile_mod_dir(profile_mod, profile))?;
        Ok(())
    }

    fn mod_dir<'a>(&'a self, profile_mod: &ProfileMod, profile: &Profile) -> Option<PathBuf> {
        Some(profile_mod_dir(profile_mod, profile))
    }
}
