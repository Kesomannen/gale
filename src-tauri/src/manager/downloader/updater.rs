use uuid::Uuid;

use super::install_mod_refs;
use crate::{
    manager::{ModManager, Profile, Result},
    thunderstore::{BorrowedMod, ModRef, Thunderstore},
};
use itertools::Itertools;
use std::sync::Mutex;
use tauri::Manager;
use anyhow::Context;

pub mod commands;

pub struct AvailableUpdate<'a> {
    pub mod_ref: &'a ModRef,
    pub enabled: bool,
    pub current: &'a semver::Version,
    pub latest: BorrowedMod<'a>,
}

impl Profile {
    pub fn update_available<'a>(
        &'a self,
        uuid: &Uuid,
        thunderstore: &'a Thunderstore,
    ) -> Result<Option<AvailableUpdate<'a>>> {
        let installed = match self.get_mod(uuid)?.as_remote() {
            Some(x) => x,
            None => return Ok(None),
        };

        let installed_vers = &installed.0.borrow(thunderstore)?.version.version_number;

        let latest = thunderstore.get_package(uuid)?;

        let latest_version = latest
            .versions
            .first()
            .context("package should have at least one version")?;

        if installed_vers >= &latest_version.version_number {
            return Ok(None);
        }

        Ok(Some(AvailableUpdate {
            mod_ref: installed.0,
            enabled: installed.1,
            current: installed_vers,
            latest: BorrowedMod {
                package: latest,
                version: latest_version,
            },
        }))
    }

    pub fn available_updates<'a>(
        &'a self,
        thunderstore: &'a Thunderstore,
    ) -> impl Iterator<Item = Result<AvailableUpdate<'a>>> + 'a {
        self.remote_mods().filter_map(move |(m, _)| {
            self.update_available(&m.package_uuid, thunderstore)
                .transpose()
        })
    }
}

pub async fn update_mods(uuids: &[Uuid], app: &tauri::AppHandle) -> Result<()> {
    let to_update = {
        let manager = app.state::<Mutex<ModManager>>();
        let mut manager = manager.lock().unwrap();

        let thunderstore = app.state::<Mutex<Thunderstore>>();
        let thunderstore = thunderstore.lock().unwrap();

        let profile = manager.active_profile_mut();

        let to_update = uuids
            .iter()
            .filter_map(|uuid| profile.update_available(uuid, &thunderstore).transpose())
            .map_ok(|update| (update.latest.reference(), update.enabled))
            .collect::<Result<Vec<_>>>()?;

        for (mod_ref, _) in &to_update {
            // remove old versions
            profile.force_remove_mod(&mod_ref.package_uuid, &thunderstore)?;
        }

        to_update
    };

    install_mod_refs(&to_update, app).await
}
