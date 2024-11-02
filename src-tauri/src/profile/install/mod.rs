use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use download::InstallState;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{iter, sync::Mutex};
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
    pub total_progress: f32,
    pub installed_mods: usize,
    pub total_mods: usize,
    pub current_name: &'a str,
    pub can_cancel: bool,
    pub task: InstallTask,
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
    pub id: ModId,
    pub enabled: bool,
    pub overwrite: bool,
    pub index: Option<usize>,
    pub install_time: Option<DateTime<Utc>>,
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
        self.id.package
    }
}

impl From<BorrowedMod<'_>> for ModInstall {
    fn from(borrowed_mod: BorrowedMod<'_>) -> Self {
        Self::new(borrowed_mod.into())
    }
}

pub async fn install_mods(
    mods: Vec<ModInstall>,
    options: InstallOptions,
    app: &AppHandle,
) -> Result<()> {
    let client = app.state::<NetworkClient>();
    let mut installer = download::Installer::create(options, &client.0, app)?;
    installer.install_all(mods).await
}

pub async fn install_with_mods<F>(
    mods: F,
    options: InstallOptions,
    app: &tauri::AppHandle,
) -> Result<()>
where
    F: FnOnce(&ModManager, &Thunderstore) -> Result<Vec<ModInstall>>,
{
    let mods = {
        let manager = app.state::<Mutex<ModManager>>();
        let thunderstore = app.state::<Mutex<Thunderstore>>();

        let manager = manager.lock().unwrap();
        let thunderstore = thunderstore.lock().unwrap();

        mods(&manager, &thunderstore).context("failed to resolve dependencies")?
    };

    install_mods(mods, options, app).await
}

pub async fn install_with_deps(
    mods: Vec<ModInstall>,
    options: InstallOptions,
    allow_multiple: bool,
    app: &tauri::AppHandle,
) -> Result<()> {
    {
        let manager = app.state::<Mutex<ModManager>>();
        let manager = manager.lock().unwrap();

        let profile = manager.active_profile();
        if !allow_multiple && mods.len() == 1 && profile.has_mod(mods[0].uuid()) {
            bail!("mod already installed");
        }
    }

    install_with_mods(
        move |manager, thunderstore| {
            let deps = mods
                .into_iter()
                .map(|install| {
                    let borrowed = install.id.borrow(thunderstore)?;

                    Ok(iter::once(install).chain(
                        manager
                            .active_profile()
                            .missing_deps(borrowed.version.dependencies.iter(), thunderstore)
                            .map_into(),
                    ))
                })
                .flatten_ok()
                .collect::<Result<Vec<_>>>()?;

            Ok(deps
                .into_iter()
                .unique_by(|install| install.uuid())
                .collect())
        },
        options,
        app,
    )
    .await
}

fn total_download_size(
    borrowed_mod: BorrowedMod<'_>,
    profile: &Profile,
    prefs: &Prefs,
    thunderstore: &Thunderstore,
) -> u64 {
    profile
        .missing_deps(&borrowed_mod.version.dependencies, thunderstore)
        .chain(iter::once(borrowed_mod))
        .filter(|borrowed| !cache::path(&borrowed.version.ident, prefs).exists())
        .map(|borrowed_mod| borrowed_mod.version.file_size)
        .sum()
}
