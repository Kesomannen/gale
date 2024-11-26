use std::{
    borrow::Cow,
    fs,
    path::{self, PathBuf},
};

use eyre::{bail, Result};

use super::{PackageInstaller, PackageZip};
use crate::profile::{
    install::{self},
    Profile, ProfileMod,
};

pub struct GDWeaveModInstaller;

fn relative_mod_dir(package_name: &str) -> PathBuf {
    ["GDWeave", "mods", package_name].iter().collect()
}

fn profile_mod_dir(package_name: &str, profile: &Profile) -> PathBuf {
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

        let mut roots: Vec<PathBuf> = Vec::new();

        for i in 0..archive.len() {
            let file = archive.by_index(i)?;
            let Some(path) = file.enclosed_name() else {
                continue;
            };

            let mut components = path.components();

            match components.next_back() {
                Some(path::Component::Normal(name))
                    if name == "manifest.json" && components.clone().count() > 0 =>
                {
                    roots.push(components.collect());
                }
                _ => (),
            }
        }

        let root = match roots.len() {
            0 => bail!("no mod root found"),
            1 => roots.into_iter().next().unwrap(),
            _ => bail!("multiple mod roots found"),
        };

        install::fs::extract(archive, dest, |relative_path| {
            if let Ok(relative_to_root) = relative_path.strip_prefix(&root) {
                let mut path = relative_mod_dir(package_name);
                path.push(relative_to_root);

                Ok(Some(Cow::Owned(path)))
            } else {
                Ok(None)
            }
        })
    }

    fn toggle(&mut self, enabled: bool, profile_mod: &ProfileMod, profile: &Profile) -> Result<()> {
        install::fs::toggle_dir(profile_mod_dir(&*profile_mod.full_name(), profile), enabled)
    }

    fn uninstall(&mut self, profile_mod: &ProfileMod, profile: &Profile) -> Result<()> {
        fs::remove_dir_all(profile_mod_dir(&*profile_mod.full_name(), profile))?;
        Ok(())
    }

    fn mod_dir<'a>(&'a self, package_name: &str, profile: &Profile) -> Option<PathBuf> {
        Some(profile_mod_dir(package_name, profile))
    }
}
