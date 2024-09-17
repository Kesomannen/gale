use crate::{import::{ImportData, ImportTarget}, modpack::ModpackArgs};
use gale_core::prelude::*;
use std::path::PathBuf;
use tauri::{AppHandle, State};
use uuid::Uuid;

#[tauri::command]
pub async fn read_code(key: Uuid, state: State<'_, AppState>) -> CmdResult<(String, ImportData)> {
    let result = crate::import::read_code(key, &state).await?;

    Ok(result)
}

#[tauri::command]
pub async fn read_file(
    path: PathBuf,
    state: State<'_, AppState>,
) -> CmdResult<(String, ImportData)> {
    let result = crate::import::read_file(&path, &state).await?;

    Ok(result)
}

#[tauri::command]
pub async fn import(
    data: ImportData,
    target: ImportTarget,
    state: State<'_, AppState>,
    app: AppHandle,
) -> CmdResult<()> {
    crate::import::import(data, target, &state, &app).await?;

    Ok(())
}

#[tauri::command]
pub async fn export_file(
    profile_id: i64,
    path: PathBuf,
    state: State<'_, AppState>,
) -> CmdResult<()> {
    crate::export::to_file(profile_id, &path, &state).await?;

    Ok(())
}

#[tauri::command]
pub async fn export_code(
    profile_id: i64,
    state: State<'_, AppState>,
) -> CmdResult<Uuid> {
    let key = crate::export::as_code(profile_id, &state).await?;

    Ok(key)
}

#[tauri::command]
pub async fn export_modpack(profile_id: i64, path: PathBuf, args: ModpackArgs, state: State<'_, AppState>) -> CmdResult<()> {
    crate::modpack::export_to_file(profile_id, &path, &args, &state).await?;

    Ok(())
}

#[tauri::command]
pub async fn publish_modpack(profile_id: i64, args: ModpackArgs, community_id: i64, state: State<'_, AppState>) -> CmdResult<()> {
    crate::modpack::export_and_publish(profile_id, args, community_id, &state).await?;

    Ok(())
}
