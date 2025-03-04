use tauri::AppHandle;
use uuid::Uuid;

use crate::{supabase, util::cmd::Result};

#[tauri::command]
pub async fn create_sync_profile(app: AppHandle) -> Result<Uuid> {
    let id = super::create_profile(&app).await?;

    Ok(id)
}

#[tauri::command]
pub async fn push_sync_profile(app: AppHandle) -> Result<()> {
    super::push_profile(&app).await?;

    Ok(())
}

#[tauri::command]
pub async fn clone_sync_profile(id: Uuid, app: AppHandle) -> Result<()> {
    super::clone_profile(id, &app).await?;

    Ok(())
}

#[tauri::command]
pub async fn pull_sync_profile(app: AppHandle) -> Result<()> {
    super::pull_profile(&app).await?;

    Ok(())
}

#[tauri::command]
pub async fn login(provider: supabase::OAuthProvider, app: AppHandle) -> Result<()> {
    supabase::login_with_oauth(provider, app).await?;

    Ok(())
}
