use crate::{emit_update, query::ProfileInfo};
use gale_core::prelude::*;
use std::path::PathBuf;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn create(
    name: String,
    path: PathBuf,
    community_id: i64,
    state: State<'_, AppState>,
) -> CmdResult<i64> {
    crate::actions::create(&name, &path, community_id, &state)
        .await
        .map_into()
}

#[tauri::command]
pub async fn delete(id: i64, state: State<'_, AppState>) -> CmdResult<()> {
    crate::actions::delete(id, &state).await.map_into()
}

#[tauri::command]
pub async fn rename(
    id: i64,
    name: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> CmdResult<()> {
    crate::actions::rename(id, &name, &state).await?;

    emit_update(id, &state, &app).await?;

    Ok(())
}

#[tauri::command]
pub async fn query(id: i64, state: State<'_, AppState>) -> CmdResult<ProfileInfo> {
    crate::query::single(id, &state).await.map_into()
}
