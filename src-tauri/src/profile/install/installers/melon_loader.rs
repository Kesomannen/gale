use std::{borrow::Cow, path::Path};

use anyhow::Result;

use crate::profile::{Profile, ProfileMod};

pub struct Installer;

impl Installer {
    pub fn map_file<'p>(&self, relative_path: &'p Path) -> Result<Option<Cow<'p, Path>>> {
        Ok(Some(Cow::Borrowed(relative_path)))
    }

    pub fn scan_mod<F>(&self, profile_mod: &ProfileMod, profile: &Profile, scan: F) -> Result<()>
    where
        F: Fn(&Path) -> Result<()>,
    {
        todo!()
    }
}
