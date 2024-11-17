use std::sync::Mutex;

use chrono::{DateTime, Utc};
use eyre::Context;
use itertools::Itertools;
use log::{debug, info};
use tauri::Manager;
use uuid::Uuid;

use super::install::{InstallOptions, ModInstall};
use crate::{
    profile::{install, ModManager, Profile, Result},
    thunderstore::{ModId, PackageListing, PackageVersion, Thunderstore},
};

pub mod commands;

pub struct AvailableUpdate<'a> {
    pub enabled: bool,
    pub index: usize,
    pub install_time: DateTime<Utc>,
    pub package: &'a PackageListing,
    pub current: &'a PackageVersion,
    pub latest: &'a PackageVersion,
}

impl From<AvailableUpdate<'_>> for ModInstall {
    fn from(value: AvailableUpdate<'_>) -> Self {
        let latest_id = ModId {
            package_uuid: value.package.uuid,
            version_uuid: value.latest.uuid,
        };

        ModInstall::new(latest_id)
            .with_state(value.enabled)
            .with_index(value.index)
            .with_time(value.install_time)
    }
}

impl Profile {
    pub fn check_update<'a>(
        &'a self,
        uuid: Uuid,
        respect_ignored: bool,
        thunderstore: &'a Thunderstore,
    ) -> Result<Option<AvailableUpdate<'a>>> {
        let index = self.index_of(uuid)?;
        let profile_mod = &self.mods[index];

        let Some((ts_mod, _)) = profile_mod.as_thunderstore() else {
            return Ok(None); // local mods can't be updated
        };

        let current = ts_mod.id.borrow(thunderstore)?.version;
        let package = thunderstore.get_package(uuid)?;

        if current.parsed_version() >= package.latest().parsed_version() {
            return Ok(None);
        }

        if respect_ignored && self.ignored_updates.contains(&uuid) {
            return Ok(None);
        }
        debug!(
            "update {} is available for {}",
            package.latest().version(),
            profile_mod.full_name()
        );

        Ok(Some(AvailableUpdate {
            index,
            package,
            current,
            latest: package.latest(),
            enabled: profile_mod.enabled,
            install_time: profile_mod.install_time,
        }))
    }
}

pub async fn change_version(mod_ref: ModId, app: &tauri::AppHandle) -> Result<()> {
    let install = {
        let manager = app.state::<Mutex<ModManager>>();
        let mut manager = manager.lock().unwrap();

        let profile = manager.active_profile_mut();
        let index = profile.index_of(mod_ref.package_uuid)?;
        let enabled = profile.mods[index].enabled;

        profile.force_remove_mod(mod_ref.package_uuid)?;

        ModInstall::new(mod_ref)
            .with_state(enabled)
            .with_index(index)
    };

    install::install_with_deps(
        vec![install],
        InstallOptions::default().can_cancel(false),
        false,
        app,
    )
    .await
}

pub async fn update_mods(
    uuids: Vec<Uuid>,
    respect_ignored: bool,
    app: &tauri::AppHandle,
) -> Result<()> {
    info!("updating {} mods", uuids.len());

    let to_update = {
        let manager = app.state::<Mutex<ModManager>>();
        let thunderstore = app.state::<Mutex<Thunderstore>>();

        let mut manager = manager.lock().unwrap();
        let thunderstore = thunderstore.lock().unwrap();

        let profile = manager.active_profile_mut();

        uuids
            .into_iter()
            .filter_map(|uuid| {
                profile
                    .check_update(uuid, respect_ignored, &thunderstore)
                    .transpose()
            })
            .map_ok(|update| update.into())
            .collect::<Result<Vec<ModInstall>>>()?
    };

    install::install_with_deps(
        to_update,
        InstallOptions::default().before_install(Box::new(|install, manager, _| {
            // remove the old version
            let profile = manager.active_profile_mut();

            // check since it could be a dependency, not an update itself
            if profile.has_mod(install.uuid()) {
                profile
                    .force_remove_mod(install.uuid())
                    .context("failed to remove existing version")?;
            }

            Ok(())
        })),
        true,
        app,
    )
    .await
}
