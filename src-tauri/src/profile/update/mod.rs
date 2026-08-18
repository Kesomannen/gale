use chrono::{DateTime, Utc};
use eyre::Context;
use itertools::Itertools;
use tauri::AppHandle;
use uuid::Uuid;

use super::install::{InstallOptions, ModInstall};
use crate::{
    profile::{
        Profile, Result,
        install::{InstallResultExt, queue::InstallQueueLock},
    },
    state::ManagerExt,
    thunderstore::{BorrowedMod, ModId, Thunderstore},
};

pub mod commands;

pub struct AvailableUpdate<'a> {
    pub enabled: bool,
    pub index: usize,
    pub install_time: DateTime<Utc>,
    pub current: BorrowedMod<'a>,
    pub latest: BorrowedMod<'a>,
}

impl From<AvailableUpdate<'_>> for ModInstall {
    fn from(value: AvailableUpdate<'_>) -> Self {
        ModInstall::new(value.latest)
            .with_state(value.enabled)
            .with_index(value.index)
            .with_time(value.install_time)
    }
}

impl Profile {
    pub fn is_update_ignored(&self, update: &AvailableUpdate<'_>) -> bool {
        let latest = &update.latest;

        self.ignored_version_updates.contains(&latest.version.uuid)
            || self.ignored_package_updates.contains(&latest.package.uuid)
    }

    pub fn check_update<'a>(
        &'a self,
        uuid: Uuid,
        thunderstore: &'a Thunderstore,
        install_queue: &InstallQueueLock,
    ) -> Result<Option<AvailableUpdate<'a>>> {
        if install_queue.has_mod(uuid, self.id) {
            return Ok(None); // a new version of this mod is installing or pending
        }

        let index = self.index_of(uuid)?;
        let profile_mod = &self.mods[index];

        let Some((ts_mod, _)) = profile_mod.as_thunderstore() else {
            return Ok(None); // local mods can't be updated
        };

        let Ok(current) = ts_mod.id.borrow(thunderstore) else {
            return Ok(None); // ignore missing mods
        };

        let package = thunderstore.get_package(current.package.uuid)?;
        let current_version = current.version.parsed_version();
        let latest = BorrowedMod {
            package,
            version: if current_version.pre.is_empty() {
                package.latest_released()
            } else {
                package.latest()
            },
        };

        if current_version >= latest.version.parsed_version() {
            return Ok(None);
        }

        Ok(Some(AvailableUpdate {
            index,
            enabled: profile_mod.enabled,
            install_time: profile_mod.install_time,
            current,
            latest,
        }))
    }
}

pub async fn change_version(mod_id: ModId, app: &AppHandle) -> Result<()> {
    let (profile_id, install) = {
        let manager = app.lock_manager();
        let thunderstore = app.lock_thunderstore();

        let profile = manager.active_profile();

        let index = profile.index_of(mod_id.package_uuid)?;
        let enabled = profile.mods[index].enabled;
        let install_time = profile.mods[index].install_time;

        (
            profile.id,
            ModInstall::try_from_id(mod_id, &thunderstore)?
                .with_state(enabled)
                .with_index(index)
                .with_time(install_time),
        )
    };

    install_updates(vec![install], profile_id, app).await
}

pub async fn update_mods(uuids: Vec<Uuid>, respect_ignored: bool, app: &AppHandle) -> Result<()> {
    let (profile_id, installs) = {
        let mut manager = app.lock_manager();
        let thunderstore = app.lock_thunderstore();
        let install_queue = app.install_queue().lock();

        let profile = manager.active_profile_mut();

        let installs = uuids
            .into_iter()
            .filter_map(|uuid| {
                profile
                    .check_update(uuid, &thunderstore, &install_queue)
                    .transpose()
            })
            .filter_ok(|update| !respect_ignored || !profile.is_update_ignored(update))
            .map_ok(ModInstall::from)
            .collect::<Result<Vec<_>>>()?;

        (profile.id, installs)
    };

    install_updates(installs, profile_id, app).await
}

async fn install_updates(
    installs: Vec<ModInstall>,
    profile_id: i64,
    app: &AppHandle,
) -> Result<()> {
    app.install_queue()
        .install_with_deps(
            installs,
            profile_id,
            InstallOptions::default()
                .cancel_individually()
                .before_install(Box::new(|install, profile| {
                    // remove the old version
                    // check since it could be a new dependency being installed, not an update itself
                    if profile.has_mod(install.uuid()) {
                        profile
                            .force_remove_mod(install.uuid())
                            .context("failed to remove existing version")?;
                    }

                    Ok(())
                })),
            true,
            app,
        )?
        .await
        .ignore_cancel()
}
