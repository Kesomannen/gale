use itertools::Itertools;
use std::sync::Mutex;
use tauri::Manager;
use uuid::Uuid;

use crate::{
    profile::{install::download::install_with_deps, ModManager, Profile, Result},
    thunderstore::{
        models::{PackageListing, PackageVersion},
        ModId, Thunderstore,
    },
};
use chrono::{DateTime, Utc};
use log::info;

use super::install::download::{self, InstallOptions, ModInstall};

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
        let latest_mod_ref = ModId {
            package: value.package.uuid4,
            version: value.latest.uuid4,
        };

        ModInstall::new(latest_mod_ref)
            .with_state(value.enabled)
            .with_index(value.index)
            .with_time(value.install_time)
            .with_overwrite(value.package.is_modpack())
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
        let ts_mod = match profile_mod.as_thunderstore() {
            Some((ts_mod, _)) => ts_mod,
            None => return Ok(None), // local mods can't be updated
        };

        let current = ts_mod.id.borrow(thunderstore)?.version;
        let package = thunderstore.get_package(uuid)?;

        let latest = package
            .versions
            .first()
            .expect("package should have at least one version");

        if current.parsed_version() >= latest.parsed_version() {
            return Ok(None);
        }

        if respect_ignored && self.ignored_updates.contains(&uuid) {
            return Ok(None);
        }

        Ok(Some(AvailableUpdate {
            index,
            package,
            current,
            latest,
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
        let index = profile.index_of(mod_ref.package)?;
        let enabled = profile.mods[index].enabled;

        profile.force_remove_mod(mod_ref.package)?;

        ModInstall::new(mod_ref)
            .with_state(enabled)
            .with_index(index)
    };

    download::install_with_deps(
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

    install_with_deps(
        to_update,
        InstallOptions::default().before_install(|install, manager, _| {
            // remove the old version
            manager
                .active_profile_mut()
                .force_remove_mod(install.uuid())
                .ok();
        }),
        true,
        app,
    )
    .await
}
