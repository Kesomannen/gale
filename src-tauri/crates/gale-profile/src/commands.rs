use crate::{
    emit_update,
    get::ProfileInfo,
    get_profile_id,
    install::{InstallMetadata, InstallQueue},
};
use futures_util::try_join;
use gale_core::prelude::*;
use gale_install::InstallSource;
use std::path::PathBuf;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn create(
    name: String,
    path: PathBuf,
    game_id: i64,
    state: State<'_, AppState>,
    app: AppHandle,
) -> CmdResult<i64> {
    let _ = LoadingBar::new("Creating profile", &app);

    let id = crate::actions::create(&name, &path, game_id, &state).await?;

    Ok(id)
}

#[tauri::command]
pub async fn delete(id: i64, state: State<'_, AppState>, app: AppHandle) -> CmdResult<()> {
    let _ = LoadingBar::new("Deleting profile", &app);

    crate::actions::delete(id, &state).await?;

    Ok(())
}

#[tauri::command]
pub async fn rename(
    id: i64,
    name: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> CmdResult<()> {
    let _ = LoadingBar::new("Renaming profile", &app);

    crate::actions::rename(id, &name, &state).await?;

    emit_update(id, &state, &app).await?;

    Ok(())
}

#[tauri::command]
pub async fn force_uninstall_mod(
    id: i64,
    state: State<'_, AppState>,
    app: AppHandle,
) -> CmdResult<()> {
    let profile_id = get_profile_id(id, &state).await?;

    crate::actions::uninstall_mod(id, &state).await?;

    emit_update(profile_id, &state, &app).await?;

    Ok(())
}

#[tauri::command]
pub async fn force_toggle_mod(
    id: i64,
    state: State<'_, AppState>,
    app: AppHandle,
) -> CmdResult<()> {
    let (profile_id, ()) = try_join!(
        get_profile_id(id, &state),
        crate::actions::toggle_mod(id, &state)
    )?;

    emit_update(profile_id, &state, &app).await?;

    Ok(())
}

#[tauri::command]
pub async fn get(id: i64, state: State<'_, AppState>) -> CmdResult<ProfileInfo> {
    crate::get::single(id, &state).await.map_into()
}

#[tauri::command]
pub fn queue_install(
    source: InstallSource,
    profile_id: i64,
    queue: State<'_, InstallQueue>,
) -> CmdResult<()> {
    queue.enqueue(source, InstallMetadata::new(profile_id));

    Ok(())
}

#[tauri::command]
pub async fn launch(id: i64, state: State<'_, AppState>) -> CmdResult<()> {
    crate::launch_::launch(id, &state).await?;

    Ok(())
}
