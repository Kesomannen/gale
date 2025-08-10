use std::{fmt::Display, iter, process};

use chrono::{DateTime, Utc};
use eyre::Result;
use gale_core::{
    game::mod_loader::{ModLoader, ModLoaderKind, Subdir},
    ident::VersionIdent,
};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;
use tokio::sync::oneshot;
use tracing::{info, warn};
use uuid::Uuid;

use super::{ModManager, Profile};
use crate::{
    prefs::Prefs,
    profile::{ProfileMod, ProfileModKind, ThunderstoreMod},
    state::ManagerExt,
    thunderstore::{BorrowedMod, ModId, Thunderstore},
};

mod cache;
pub mod commands;
mod fs;
mod installers;
pub use installers::*;
pub mod queue;

type BeforeInstallHandler =
    Box<dyn Fn(&ModInstall, &mut ModManager) -> Result<()> + 'static + Send + Sync>;

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

/// Checks for the mod loader's own package on Thunderstore.
fn is_loader_package(mod_loader: &ModLoader, full_name: &str) -> bool {
    if let Some(package_name) = mod_loader.package_name {
        full_name == package_name
    } else {
        match &mod_loader.kind {
            ModLoaderKind::BepInEx { .. } => full_name.starts_with("BepInEx-BepInExPack"),
            ModLoaderKind::MelonLoader { .. } => full_name == "LavaGang-MelonLoader",
            ModLoaderKind::GDWeave {} => full_name == "NotNet-GDWeave",
            ModLoaderKind::Northstar {} => full_name == "northstar-Northstar",
            ModLoaderKind::Shimloader {} => full_name == "Thunderstore-unreal_shimloader",
            ModLoaderKind::Lovely {} => full_name == "Thunderstore-lovely",
            ModLoaderKind::ReturnOfModding { .. } => full_name == "ReturnOfModding-ReturnOfModding",
        }
    }
}

pub fn installer_for(
    mod_loader: &'static ModLoader,
    package_name: &str,
) -> Box<dyn PackageInstaller> {
    match (
        is_loader_package(mod_loader, package_name),
        &mod_loader.kind,
    ) {
        (true, ModLoaderKind::BepInEx { .. }) => Box::new(BepinexInstaller),
        (false, ModLoaderKind::BepInEx { extra_subdirs, .. }) => {
            const SUBDIRS: &[Subdir] = &[
                Subdir::flat_separated("plugins", "BepInEx/plugins"),
                Subdir::flat_separated("patchers", "BepInEx/patchers"),
                Subdir::flat_separated("monomod", "BepInEx/monomod").extension(".mm.dll"),
                Subdir::flat_separated("core", "BepInEx/core"),
                Subdir::untracked("config", "BepInEx/config").mutable(),
            ];

            Box::new(
                SubdirInstaller::new(SUBDIRS)
                    .with_default(0)
                    .with_extras(extra_subdirs),
            )
        }

        (true, ModLoaderKind::MelonLoader { .. }) => {
            const FILES: &[&str] = &[
                "dobby.dll",
                "version.dll",
                "MelonLoader/Dependencies",
                "MelonLoader/Documentation",
                "MelonLoader/net6",
                "MelonLoader/net35",
            ];

            Box::new(ExtractInstaller::new(FILES, FlattenTopLevel::No))
        }
        (false, ModLoaderKind::MelonLoader { extra_subdirs }) => {
            const SUBDIRS: &[Subdir] = &[
                Subdir::tracked("UserLibs", "UserLibs").extension(".lib.dll"),
                Subdir::tracked("Managed", "MelonLoader/Managed").extension(".managed.dll"),
                Subdir::tracked("Mods", "Mods").extension(".dll"),
                Subdir::separated("ModManager", "UserData/ModManager"),
                Subdir::tracked("MelonLoader", "MelonLoader"),
                Subdir::tracked("Libs", "MelonLoader/Libs"),
            ];
            const IGNORED: &[&str] = &["manifest.json", "icon.png", "README.md"];

            Box::new(
                SubdirInstaller::new(SUBDIRS)
                    .with_default(2)
                    .with_extras(extra_subdirs)
                    .with_ignored_files(IGNORED),
            )
        }

        (true, ModLoaderKind::GDWeave {}) => {
            const FILES: &[&str] = &["winmm.dll", "GDWeave/core"];

            Box::new(ExtractInstaller::new(FILES, FlattenTopLevel::No))
        }
        (false, ModLoaderKind::GDWeave {}) => Box::new(GDWeaveModInstaller),

        (true, ModLoaderKind::Northstar {}) => {
            const FILES: &[&str] = &[
                "Northstar.dll",
                "NorthstarLauncher.exe",
                "r2ds.bat",
                "bin",
                "R2Northstar/plugins",
                "R2Northstar/mods/Northstar.Client",
                "R2Northstar/mods/Northstar.Custom",
                "R2Northstar/mods/Northstar.CustomServers",
                "R2Northstar/mods/md5sum.text",
            ];

            Box::new(ExtractInstaller::new(FILES, FlattenTopLevel::Yes))
        }
        (false, ModLoaderKind::Northstar {}) => {
            const SUBDIRS: &[Subdir] = &[Subdir::tracked("mods", "R2Northstar/mods")];
            const IGNORED: &[&str] = &["manifest.json", "icon.png", "README.md", "LICENSE"];

            Box::new(SubdirInstaller::new(SUBDIRS).with_ignored_files(IGNORED))
        }

        (true, ModLoaderKind::Shimloader {}) => Box::new(ShimloaderInstaller),
        (false, ModLoaderKind::Shimloader {}) => {
            const SUBDIRS: &[Subdir] = &[
                Subdir::flat_separated("mod", "shimloader/mod"),
                Subdir::flat_separated("pak", "shimloader/pak"),
                Subdir::untracked("cfg", "shimloader/cfg").mutable(),
            ];

            Box::new(SubdirInstaller::new(SUBDIRS).with_default(0))
        }

        (true, ModLoaderKind::ReturnOfModding { files }) => {
            Box::new(ExtractInstaller::new(files, FlattenTopLevel::Yes))
        }
        (false, ModLoaderKind::ReturnOfModding { .. }) => {
            const SUBDIRS: &[Subdir] = &[
                Subdir::separated("plugins", "ReturnOfModding/plugins"),
                Subdir::separated("plugins_data", "ReturnOfModding/plugins_data"),
                Subdir::separated("config", "ReturnOfModding/config").mutable(),
            ];

            Box::new(SubdirInstaller::new(SUBDIRS).with_default(0))
        }

        (true, ModLoaderKind::Lovely {}) => {
            const FILES: &[&str] = &["version.dll"];

            Box::new(ExtractInstaller::new(FILES, FlattenTopLevel::No))
        }
        (false, ModLoaderKind::Lovely {}) => {
            const SUBDIRS: &[Subdir] = &[Subdir::separated("", "mods")];

            Box::new(SubdirInstaller::new(SUBDIRS).with_default(0))
        }
    }
}
