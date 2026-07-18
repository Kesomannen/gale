use tauri::{AppHandle, command};
use uuid::Uuid;

use crate::{state::ManagerExt, thunderstore::ModId, util::cmd::Result};

#[command]
pub async fn change_mod_version(id: ModId, app: AppHandle) -> Result<()> {
    super::change_version(id, &app).await?;

    Ok(())
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
