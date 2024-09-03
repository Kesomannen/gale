use itertools::Itertools;
use std::sync::Mutex;
use tauri::Manager;
use uuid::Uuid;

use super::{install_with_deps, InstallOptions, ModInstall};
use crate::{
    manager::{ModManager, Profile, Result},
    thunderstore::{
        models::{PackageListing, PackageVersion},
        ModRef, Thunderstore,
    },
};
use log::info;

pub mod commands;

pub struct AvailableUpdate<'a> {
    pub enabled: bool,
    pub index: usize,
    pub package: &'a PackageListing,
    pub current: &'a PackageVersion,
    pub latest: &'a PackageVersion,
}

impl From<AvailableUpdate<'_>> for ModInstall {
    fn from(value: AvailableUpdate<'_>) -> Self {
        let latest_mod_ref = ModRef {
            package_uuid: value.package.uuid4,
            version_uuid: value.latest.uuid4,
        };

        ModInstall::new(latest_mod_ref)
            .with_state(value.enabled)
            .at(value.index)
    }
}

impl Profile {
    pub fn check_update<'a>(
        &'a self,
        uuid: &Uuid,
        respect_ignored: bool,
        thunderstore: &'a Thunderstore,
    ) -> Result<Option<AvailableUpdate<'a>>> {
        let index = self.index_of(uuid)?;

        let (mod_ref, _, enabled) = match self.mods[index].as_remote() {
            Some(x) => x,
            None => return Ok(None), // local mods can't be updated
        };

        let current = mod_ref.borrow(thunderstore)?.version;
        let package = thunderstore.get_package(uuid)?;

        let latest = package
            .versions
            .first()
            .expect("package should have at least one version");

        if current.version_number >= latest.version_number {
            return Ok(None);
        }

        if respect_ignored && self.ignored_updates.contains(uuid) {
            return Ok(None);
        }

        Ok(Some(AvailableUpdate {
            index,
            enabled,
            package,
            current,
            latest,
        }))
    }
}

pub async fn change_version(mod_ref: ModRef, app: &tauri::AppHandle) -> Result<()> {
    let install = {
        let manager = app.state::<Mutex<ModManager>>();
        let mut manager = manager.lock().unwrap();

        let profile = manager.active_profile_mut();
        let index = profile.index_of(&mod_ref.package_uuid)?;
        let enabled = profile.mods[index].enabled;

        profile.force_remove_mod(&mod_ref.package_uuid)?;

        ModInstall::new(mod_ref).with_state(enabled).at(index)
    };

    super::install_with_deps(
        vec![install],
        InstallOptions::default().can_cancel(false),
        false,
        app,
    )
    .await
}

pub async fn update_mods(
    uuids: &[Uuid],
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
            .iter()
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
