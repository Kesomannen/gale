use tauri::{AppHandle, command};
use uuid::Uuid;

use super::modpack::ModpackChange;
use crate::{state::ManagerExt, thunderstore::ModId, util::cmd::Result};

#[command]
pub async fn change_mod_versions(ids: Vec<ModId>, app: AppHandle) -> Result<()> {
    super::change_versions(ids, &app).await?;

    Ok(())
}

#[command]
pub async fn get_modpack_changes(id: ModId, app: AppHandle) -> Result<Vec<ModpackChange>> {
    let manager = app.lock_manager();
    let thunderstore = app.lock_thunderstore();
    let install_queue = app.install_queue().lock();

    let profile = manager.active_profile();
    if install_queue.has_mod(id.package_uuid, profile.id) {
        return Ok(Vec::new()); // the modpack is already being updated
    }

    let target = id.borrow(&thunderstore)?;

    Ok(profile.modpack_changes(target, &thunderstore))
}

#[command]
pub async fn update_mods(uuids: Vec<Uuid>, respect_ignored: bool, app: AppHandle) -> Result<()> {
    super::update_mods(uuids, respect_ignored, &app).await?;

    Ok(())
}

#[command]
pub fn ignore_update(version_uuid: Uuid, app: AppHandle) -> Result<()> {
    let mut manager = app.lock_manager();

    let profile = manager.active_profile_mut();
    profile.ignored_version_updates.insert(version_uuid);
    profile.save(&app, true)?;

    Ok(())
}

#[command]
pub fn ignore_package_updates(package_uuid: Uuid, app: AppHandle) -> Result<()> {
    let mut manager = app.lock_manager();

    let profile = manager.active_profile_mut();
    profile.ignored_package_updates.insert(package_uuid);
    profile.save(&app, true)?;

    Ok(())
}
