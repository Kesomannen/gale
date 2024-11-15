use std::{
    borrow::Cow,
    fs,
    path::{Component, PathBuf},
};

use eyre::{Context, Result};

use super::{PackageZip, PackageInstaller};
use crate::profile::{
    install::{self},
    Profile, ProfileMod,
};

pub struct ShimloaderInstaller;

impl PackageInstaller for ShimloaderInstaller {
    fn extract(&mut self, archive: PackageZip, _package_name: &str, dest: PathBuf) -> Result<()> {
        fs::create_dir_all(dest.join("shimloader").join("cfg"))
            .context("failed to create cfg directory")?;

        install::fs::extract(archive, dest, |relative_path| {
            let mut components = relative_path.components();

            if relative_path.starts_with("UE4SS") {
                components.next();
            }

            let Some(Component::Normal(next)) = components.clone().next() else {
                return Ok(None);
            };

            Ok(match next.to_str() {
                Some("dwmapi.dll" | "ue4ss.dll" | "UE4SS-settings.ini") => {
                    Some(Cow::Borrowed(components.as_path()))
                }
                Some("Mods") => {
                    components.next();

                    let mut path: PathBuf = ["shimloader", "mod"].iter().collect();
                    path.push(components);

                    Some(Cow::Owned(path))
                }
                _ => None,
            })
        })
    }

    fn toggle(
        &mut self,
        enabled: bool,
        _profile_mod: &ProfileMod,
        profile: &Profile,
    ) -> Result<()> {
        for file in ["dwmapi.dll", "ue4ss.dll", "UE4SS-settings.ini"] {
            install::fs::toggle_file(profile.path.join(file), enabled)?;
        }

        Ok(())
    }

    fn uninstall(&mut self, _profile_mod: &ProfileMod, profile: &Profile) -> Result<()> {
        for file in ["dwmapi.dll", "ue4ss.dll", "UE4SS-settings.ini"] {
            fs::remove_file(profile.path.join(file))?;
        }

        Ok(())
    }
}
