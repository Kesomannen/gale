use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use anyhow::Result;

use super::{PackageInstaller, ScanFn};
use crate::profile::{Profile, ProfileMod};

pub struct GDWeaveModInstaller;

fn relative_mod_dir(package_name: &str) -> PathBuf {
    ["GDWeave", "mods", package_name].iter().collect()
}

impl PackageInstaller for GDWeaveModInstaller {
    fn map_file<'p>(
        &mut self,
        relative_path: &'p Path,
        package_name: &str,
    ) -> Result<Option<Cow<'p, Path>>> {
        if let Ok(path) = relative_path.strip_prefix("GDWeave/mods") {
            let mut relative_dir = relative_mod_dir(package_name);
            relative_dir.push(path);

            Ok(Some(Cow::Owned(relative_dir)))
        } else {
            Ok(None)
        }
    }

    fn scan_mod(
        &mut self,
        profile_mod: &ProfileMod,
        profile: &Profile,
        scan: ScanFn,
    ) -> Result<()> {
        let relative_dir = relative_mod_dir(profile_mod.ident().full_name());

        scan(&profile.path.join(relative_dir))
    }
}
