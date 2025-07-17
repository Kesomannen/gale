use std::iter;

use chrono::{DateTime, Utc};
use eyre::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{ModManager, Profile};
use crate::{
    prefs::Prefs,
    profile::{ProfileMod, ProfileModKind, ThunderstoreMod},
    thunderstore::{BorrowedMod, ModId, Thunderstore, VersionIdent},
};

mod cache;
pub mod commands;
mod fs;
mod installers;
pub use installers::*;
pub mod queue;

type EventHandler = Box<dyn Fn(&ModInstall, &mut ModManager) -> Result<()> + 'static + Send + Sync>;

pub struct InstallOptions {
    can_cancel: bool,
    send_progress: bool,
    before_install: Option<EventHandler>,
}

impl Default for InstallOptions {
    fn default() -> Self {
        Self {
            can_cancel: true,
            send_progress: true,
            before_install: None,
        }
    }
}

impl InstallOptions {
    pub fn can_cancel(mut self, can_cancel: bool) -> Self {
        self.can_cancel = can_cancel;
        self
    }

    pub fn send_progress(mut self, send_progress: bool) -> Self {
        self.send_progress = send_progress;
        self
    }

    pub fn before_install(mut self, before_install: EventHandler) -> Self {
        self.before_install = Some(before_install);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModInstall {
    id: ModId,
    ident: VersionIdent,
    file_size: u64,
    enabled: bool,
    index: Option<usize>,
    install_time: Option<DateTime<Utc>>,
}

impl ModInstall {
    pub fn new<'a>(borrowed: impl Into<BorrowedMod<'a>>) -> Self {
        let borrowed = borrowed.into();
        Self {
            id: borrowed.into(),
            ident: borrowed.ident().to_owned(),
            file_size: borrowed.version.file_size,
            enabled: true,
            index: None,
            install_time: None,
        }
    }

    pub fn from_id(mod_id: ModId, thunderstore: &Thunderstore) -> Result<Self> {
        mod_id.borrow(thunderstore).map(Self::new)
    }

    pub fn with_state(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn with_index(mut self, index: usize) -> Self {
        self.index = Some(index);
        self
    }

    pub fn with_time(mut self, date: DateTime<Utc>) -> Self {
        self.install_time = Some(date);
        self
    }

    /// The uuid the resulting `ProfileMod` will get after the mod is installed.
    pub fn uuid(&self) -> Uuid {
        self.id.package_uuid
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    fn insert_into(self, profile: &mut Profile) -> Result<()> {
        let ModInstall {
            id,
            ident,
            enabled,
            index,
            install_time,
            ..
        } = self;

        let uuid = id.package_uuid;
        let install_time = install_time.unwrap_or_else(Utc::now);

        let profile_mod = ProfileMod::new_at(
            install_time,
            ProfileModKind::Thunderstore(ThunderstoreMod { ident, id }),
        );

        match index {
            Some(index) if index < profile.mods.len() => {
                profile.mods.insert(index, profile_mod);
            }
            _ => {
                profile.mods.push(profile_mod);
            }
        };

        if !enabled {
            profile.force_toggle_mod(uuid)?;
        }

        Ok(())
    }
}

impl From<BorrowedMod<'_>> for ModInstall {
    fn from(borrowed_mod: BorrowedMod<'_>) -> Self {
        Self::new(borrowed_mod)
    }
}

/// Gets the number of bytes to download the given mod and its
/// missing dependencies (ignoring already cached mods).
fn total_download_size(
    borrowed: BorrowedMod<'_>,
    profile: &Profile,
    prefs: &Prefs,
    thunderstore: &Thunderstore,
    queue: &queue::InstallQueueHandle,
) -> u64 {
    profile
        .missing_deps(borrowed.dependencies(), thunderstore)
        .chain(iter::once(borrowed))
        .filter(|borrowed| {
            !cache::path(borrowed.ident(), prefs).exists()
                && !queue.has_mod(borrowed.package.uuid, profile.id)
        })
        .map(|borrowed| borrowed.version.file_size)
        .sum()
}
