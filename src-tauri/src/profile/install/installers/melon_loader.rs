use std::{
    borrow::Cow,
    path::{Component, Path},
};

use anyhow::{bail, Result};

use super::{FileInstallMethod, PackageInstaller, ScanFn};
use crate::profile::{Profile, ProfileMod};

pub struct MelonLoaderInstaller;

impl PackageInstaller for MelonLoaderInstaller {
    fn map_file<'p>(
        &mut self,
        relative_path: &'p Path,
        _package_name: &str,
    ) -> Result<Option<Cow<'p, Path>>> {
        let mut components = relative_path.components();

        let first = match components.next() {
            Some(Component::Normal(name)) => name,
            _ => bail!("malformed package"),
        };

        if components.next().is_none() {
            // top level file
            if first != "dobby.dll" && first != "version.dll" {
                return Ok(None);
            }
        }

        Ok(Some(Cow::Borrowed(relative_path)))
    }

    fn scan_mod(
        &mut self,
        _profile_mod: &ProfileMod,
        profile: &Profile,
        scan: ScanFn,
    ) -> Result<()> {
        let mut path = profile.path.to_path_buf();

        path.push("dobby.dll");
        scan(&path)?;
        path.pop();

        path.push("version.dll");
        scan(&path)?;
        path.pop();

        path.push("MelonLoader");
        for dir in ["Dependencies", "Documentation", "net6", "net35"] {
            path.push(dir);
            scan(&path)?;
            path.pop();
        }
        path.pop();

        Ok(())
    }

    fn install_file(
        &mut self,
        _relative_path: &Path,
        _profile: &Profile,
    ) -> Result<FileInstallMethod> {
        Ok(FileInstallMethod::Link)
    }
}
