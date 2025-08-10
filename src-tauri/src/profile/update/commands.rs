use tauri::{command, AppHandle};
use uuid::Uuid;

use crate::{state::ManagerExt, thunderstore::ModId};
use gale_util::cmd::Result;

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
    profile.ignored_updates.insert(version_uuid);
    profile.save(&app, true)?;

    Ok(())
}
