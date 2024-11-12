use std::{iter, sync::Mutex};

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use download::InstallState;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use uuid::Uuid;

use super::{ModManager, Profile};
use crate::{
    prefs::Prefs,
    thunderstore::{BorrowedMod, ModId, Thunderstore},
    NetworkClient,
};

mod cache;
pub mod commands;
pub mod deep_link;
mod download;
mod fs;
mod installers;
pub use installers::*;
#[cfg(test)]
mod tests;

pub use fs::install_from_zip;

pub fn setup(handle: &AppHandle) -> Result<()> {
    handle.manage(Mutex::new(InstallState::default()));

    Ok(())
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InstallProgress<'a> {
    /// The percentage of "completed" bytes, from 0 to 1.
    total_progress: f32,
    installed_mods: usize,
    total_mods: usize,
    current_name: &'a str,
    can_cancel: bool,
    task: InstallTask,
}

impl<'a> InstallProgress<'a> {
    pub fn total_progress(&self) -> f32 {
        self.total_progress
    }
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase", tag = "kind", content = "payload")]
pub enum InstallTask {
    Done,
    Error,
    Downloading { total: u64, downloaded: u64 },
    Extracting,
    Installing,
}

type ProgressHandler = Box<dyn Fn(&InstallProgress, &AppHandle) + 'static + Send>;
type EventHandler = Box<dyn Fn(&ModInstall, &mut ModManager, &Thunderstore) + 'static + Send>;

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

    pub fn on_progress<F>(mut self, on_progress: F) -> Self
    where
        F: Fn(&InstallProgress, &AppHandle) + 'static + Send,
    {
        self.on_progress = Some(Box::new(on_progress));
        self
    }

    pub fn before_install<F>(mut self, before_install: F) -> Self
    where
        F: Fn(&ModInstall, &mut ModManager, &Thunderstore) + 'static + Send,
    {
        self.before_install = Some(Box::new(before_install));
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModInstall {
    id: ModId,
    enabled: bool,
    overwrite: bool,
    index: Option<usize>,
    install_time: Option<DateTime<Utc>>,
}

impl ModInstall {
    pub fn new(id: ModId) -> Self {
        Self {
            id,
            enabled: true,
            overwrite: false,
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

    pub fn with_overwrite(mut self, overwrite: bool) -> Self {
        self.overwrite = overwrite;
        self
    }

    pub fn uuid(&self) -> Uuid {
        self.id.package_uuid
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
    let client = app.state::<NetworkClient>();
    let mut installer = download::Installer::create(options, &client.0, app)?;
    installer.install_all(mods).await
}

/// Downloads and installs mods returned by a closure on the active profile.
///
/// Note that this does not check for duplicates, so make sure
/// none of `mods` are already installed!
pub async fn install_with_mods<F>(
    options: InstallOptions,
    app: &tauri::AppHandle,
    mods: F,
) -> Result<()>
where
    F: FnOnce(&ModManager, &Thunderstore) -> Result<Vec<ModInstall>>,
{
    let mods = {
        let manager = app.state::<Mutex<ModManager>>();
        let thunderstore = app.state::<Mutex<Thunderstore>>();

        let manager = manager.lock().unwrap();
        let thunderstore = thunderstore.lock().unwrap();

        mods(&manager, &thunderstore)?
    };

    install_mods(mods, options, app).await
}

/// Downloads and installs mods and their missing dependencies on the active profile.
///
/// Dependencies are installed right after each respective mod. The ordering of the
/// dependencies is decided by [`Thunderstore::dependencies`].
pub async fn install_with_deps(
    mods: Vec<ModInstall>,
    options: InstallOptions,
    allow_multiple: bool,
    app: &tauri::AppHandle,
) -> Result<()> {
    install_with_mods(options, app, move |manager, thunderstore| {
        let profile = manager.active_profile();

        if !allow_multiple && mods.len() == 1 && profile.has_mod(mods[0].uuid()) {
            bail!("mod already installed");
        }

        let deps = mods
            .into_iter()
            .map(|install| {
                let borrowed = install.id.borrow(thunderstore)?;

                Ok(iter::once(install).chain(
                    profile
                        .missing_deps(borrowed.dependencies(), thunderstore)
                        .map(ModInstall::from),
                ))
            })
            .flatten_ok()
            .collect::<Result<Vec<_>>>()
            .context("failed to resolve dependencies")?;

        Ok(deps
            .into_iter()
            .unique_by(|install| install.uuid())
            .collect())
    })
    .await
}

/// Gets the number of bytes to download the given mod and its
/// missing dependencies.
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
