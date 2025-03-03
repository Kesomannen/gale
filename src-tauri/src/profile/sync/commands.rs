use tauri::{AppHandle, State};
use uuid::Uuid;

use crate::{
    prefs::Prefs,
    profile::ModManager,
    util::cmd::{Result, StateMutex},
    NetworkClient,
};

#[tauri::command]
pub async fn create_sync_profile(
    client: State<'_, NetworkClient>,
    manager: StateMutex<'_, ModManager>,
    prefs: StateMutex<'_, Prefs>,
) -> Result<Uuid> {
    let id = super::create_profile(&manager, &prefs, &client.0).await?;

    Ok(id)
}

#[tauri::command]
pub async fn push_sync_profile(
    client: State<'_, NetworkClient>,
    manager: StateMutex<'_, ModManager>,
    prefs: StateMutex<'_, Prefs>,
) -> Result<()> {
    super::push_profile(&manager, &prefs, &client.0).await?;

    Ok(())
}

#[tauri::command]
pub async fn clone_sync_profile(id: Uuid, app: AppHandle) -> Result<()> {
    super::clone_profile(id, &app).await?;

    Ok(())
}
