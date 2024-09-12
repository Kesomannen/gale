use crate::{
    emit_update,
    install::{InstallMetadata, InstallSource},
    query::ProfileInfo,
};
use gale_core::prelude::*;
use std::path::PathBuf;
use tauri::{AppHandle, State};
use uuid::Uuid;

#[tauri::command]
pub async fn create(
    name: String,
    path: PathBuf,
    community_id: i64,
    state: State<'_, AppState>,
    app: AppHandle,
) -> CmdResult<i64> {
    let _ = LoadingBar::indeterminate("Creating profile", &app);

    let id = crate::actions::create_profile(&name, &path, community_id, &state).await?;

    Ok(id)
}

#[tauri::command]
pub async fn delete(id: i64, state: State<'_, AppState>, app: AppHandle) -> CmdResult<()> {
    let _ = LoadingBar::indeterminate("Deleting profile", &app);

    crate::actions::delete_profile(id, &state).await?;

    Ok(())
}

#[tauri::command]
pub async fn rename(
    id: i64,
    name: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> CmdResult<()> {
    let _ = LoadingBar::indeterminate("Renaming profile", &app);

    crate::actions::rename_profile(id, &name, &state).await?;

    emit_update(id, &state, &app).await?;

    Ok(())
}

#[tauri::command]
pub async fn force_uninstall(profile_mod_id: i64, state: State<'_, AppState>) -> CmdResult<()> {
    crate::actions::uninstall_mod(profile_mod_id, &state).await?;

    Ok(())
}

#[tauri::command]
pub async fn force_toggle(profile_mod_id: i64, state: State<'_, AppState>) -> CmdResult<()> {
    crate::actions::toggle_mod(profile_mod_id, &state).await?;

    Ok(())
}

#[tauri::command]
pub async fn query(id: i64, state: State<'_, AppState>) -> CmdResult<ProfileInfo> {
    crate::query::single(id, &state).await.map_into()
}

#[tauri::command]
pub async fn install_from_thunderstore(
    version_uuid: Uuid,
    profile_id: i64,
    app: AppHandle,
) -> CmdResult<()> {
    crate::install::enqueue(
        InstallSource::Thunderstore(version_uuid),
        InstallMetadata::new(profile_id),
        &app,
    )?;

    Ok(())
}
