use std::iter;

use chrono::{DateTime, Utc};
use eyre::{bail, Context, Result};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use strum_macros::Display;
use tauri::AppHandle;
use uuid::Uuid;

use super::{ModManager, Profile};
use crate::{
    prefs::Prefs,
    state::ManagerExt,
    thunderstore::{BorrowedMod, ModId, Thunderstore},
};

mod cache;
pub mod commands;
mod download;
mod fs;
mod installers;
pub use installers::*;

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InstallProgress<'a> {
    /// The percentage of "completed" bytes, from 0 to 1.
    pub total_progress: f32,
    pub duration_secs: f32,
    pub installed_mods: usize,
    pub total_mods: usize,
    pub current_name: &'a str,
    pub can_cancel: bool,
    pub task: InstallTask,
}

#[derive(Serialize, Debug, Clone, Display)]
#[serde(rename_all = "camelCase", tag = "kind", content = "payload")]
#[strum(serialize_all = "camelCase")]
pub enum InstallTask {
    Done,
    Error,
    Downloading { total: u64, downloaded: u64 },
    Extracting,
    Installing,
}

type ProgressHandler = Box<dyn Fn(&InstallProgress, &AppHandle) + 'static + Send>;
type EventHandler =
    Box<dyn Fn(&ModInstall, &mut ModManager, &Thunderstore) -> Result<()> + 'static + Send>;

pub struct InstallOptions {
    can_cancel: bool,
    send_progress: bool,
    on_progress: Option<ProgressHandler>,
    before_install: Option<EventHandler>,
}

impl Default for InstallOptions {
    fn default() -> Self {
        Self {
            can_cancel: true,
            send_progress: true,
            on_progress: None,
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

    pub fn on_progress(mut self, on_progress: ProgressHandler) -> Self {
        self.on_progress = Some(on_progress);
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
    enabled: bool,
    index: Option<usize>,
    install_time: Option<DateTime<Utc>>,
}

impl ModInstall {
    pub fn new(id: ModId) -> Self {
        Self {
            id,
            enabled: true,
            index: None,
            install_time: None,
        }
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
}

impl From<BorrowedMod<'_>> for ModInstall {
    fn from(borrowed_mod: BorrowedMod<'_>) -> Self {
        Self::new(borrowed_mod.into())
    }
}

/// Downloads and install mods on the active profile.
///
/// Note that this does not check for duplicates, so make sure
/// none of `mods` are already installed!
pub async fn install_mods(
    mods: Vec<ModInstall>,
    options: InstallOptions,
    app: &AppHandle,
) -> Result<()> {
    download::Installer::create(options, app)?
        .install_all(mods)
        .await
}

/// Downloads and installs mods and their missing dependencies on the active profile.
///
/// Dependencies are installed before each respective mod, sorted by descending depth.
pub async fn install_with_deps(
    mods: Vec<ModInstall>,
    options: InstallOptions,
    allow_multiple: bool,
    app: &tauri::AppHandle,
) -> Result<()> {
    let mods = {
        let manager = app.lock_manager();
        let thunderstore = app.lock_thunderstore();
        let profile = manager.active_profile();

        if !allow_multiple && mods.len() == 1 && profile.has_mod(mods[0].uuid()) {
            bail!("mod already installed");
        }

        let mods = mods
            .into_iter()
            .map(|install| {
                let borrowed = install.id.borrow(&thunderstore)?;

                Ok(iter::once(install).chain(
                    profile
                        .missing_deps(borrowed.dependencies(), &thunderstore)
                        .map(ModInstall::from),
                ))
            })
            .flatten_ok()
            .collect::<Result<Vec<_>>>()
            .context("failed to resolve dependencies")?;

        mods.into_iter()
            .unique_by(|install| install.uuid())
            .rev() // install dependencies first
            .collect()
    };

    install_mods(mods, options, app).await
}

/// Gets the number of bytes to download the given mod and its
/// missing dependencies (ignoring already cached mods).
fn total_download_size(
    borrowed: BorrowedMod<'_>,
    profile: &Profile,
    prefs: &Prefs,
    thunderstore: &Thunderstore,
) -> u64 {
    profile
        .missing_deps(borrowed.dependencies(), thunderstore)
        .chain(iter::once(borrowed))
        .filter(|borrowed| !cache::path(borrowed.ident(), prefs).exists())
        .map(|borrowed| borrowed.version.file_size)
        .sum()
}
