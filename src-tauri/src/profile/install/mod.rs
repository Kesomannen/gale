use std::{fmt::Display, iter, process};

use chrono::{DateTime, Utc};
use eyre::Result;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;
use tokio::sync::oneshot;
use tracing::{info, warn};
use uuid::Uuid;

use super::Profile;
use crate::{
    prefs::Prefs,
    profile::{ProfileMod, ProfileModKind, ThunderstoreMod},
    state::ManagerExt,
    thunderstore::{BorrowedMod, ModId, Thunderstore, VersionIdent},
};

mod cache;
pub mod commands;
mod fs;
mod installers;
pub use installers::*;
pub mod queue;

type BeforeInstallHandler =
    Box<dyn Fn(&ModInstall, &mut Profile) -> Result<()> + 'static + Send + Sync>;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CancelBehavior {
    /// When installation is cancelled, rollback the already installed mods from this batch.
    #[default]
    Batch,
    /// When installation is cancelled, don't rollback any already installed mods.
    Individual,
    /// (Attempt to) prevent this batch from being cancelled at all.
    Prevent,
}

#[derive(Default)]
pub struct InstallOptions {
    cancel_behavior: CancelBehavior,
    before_install: Option<BeforeInstallHandler>,
}

impl InstallOptions {
    pub fn cancel_individually(self) -> Self {
        self.cancel_behavior(CancelBehavior::Individual)
    }

    pub fn prevent_cancel(self) -> Self {
        self.cancel_behavior(CancelBehavior::Prevent)
    }

    pub fn cancel_behavior(mut self, behavior: CancelBehavior) -> Self {
        self.cancel_behavior = behavior;
        self
    }

    pub fn before_install(mut self, before_install: BeforeInstallHandler) -> Self {
        self.before_install = Some(before_install);
        self
    }
}

/// A mod waiting to be installed via [`queue::InstallQueue`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModInstall {
    id: ModId,
    ident: VersionIdent,
    file_size: u64,
    enabled: bool,
    /// Where in the profile this should be installed.
    index: Option<usize>,
    /// At which time this mod was originally installed.
    ///
    /// This is mainly used to retain the install date when updating mods.
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

    pub fn try_from_id(mod_id: ModId, thunderstore: &Thunderstore) -> Result<Self> {
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

    pub fn mod_id(&self) -> &ModId {
        &self.id
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

pub type InstallResult<T> = std::result::Result<T, InstallError>;

#[derive(Debug)]
pub enum InstallError {
    Cancelled,
    Err(eyre::Report),
}

impl From<eyre::Report> for InstallError {
    fn from(value: eyre::Report) -> Self {
        Self::Err(value)
    }
}

impl std::error::Error for InstallError {}

impl Display for InstallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstallError::Cancelled => f.pad("installation was cancelled"),
            InstallError::Err(report) => report.fmt(f),
        }
    }
}

pub trait InstallResultExt<T> {
    /// Maps `Err(InstallError::Cancelled)` to `Ok(())`.
    fn ignore_cancel(self) -> Result<()>;
}

impl<T> InstallResultExt<T> for InstallResult<T> {
    fn ignore_cancel(self) -> Result<()> {
        match self {
            Ok(_) => Ok(()),
            Err(InstallError::Cancelled) => Ok(()),
            Err(InstallError::Err(err)) => Err(err),
        }
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

pub async fn handle_exit(app: AppHandle) {
    let install_queue = app.install_queue();

    enum DialogDecision {
        Wait,
        Cancel,
    }

    let (dialog_tx, dialog_rx) = oneshot::channel();
    // subscribe up here in case the installation finishes while the dialog is open
    let wait_for_install = install_queue.wait_for_empty();

    app.dialog()
        .message("Gale is busy installing mods.")
        .buttons(tauri_plugin_dialog::MessageDialogButtons::OkCancelCustom(
            "Continue in background".to_string(),
            "Cancel".to_string(),
        ))
        .show(move |result| {
            dialog_tx
                .send(if result {
                    DialogDecision::Wait
                } else {
                    DialogDecision::Cancel
                })
                .ok();
        });

    let decision = dialog_rx.await.expect("dialog channel closed too early");

    match decision {
        DialogDecision::Wait => {
            info!("waiting for installations to complete before exiting");
        }
        DialogDecision::Cancel => {
            warn!("cancelling installations");
            install_queue.cancel_all();
        }
    }

    wait_for_install.await;
    process::exit(0);
}
