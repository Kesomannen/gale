use tauri::AppHandle;
use uuid::Uuid;

use crate::{state::ManagerExt, util::cmd::Result};

use super::auth;

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
pub async fn fetch_sync_profile(app: AppHandle) -> Result<()> {
    super::fetch_profile(&app).await?;

    Ok(())
}

#[tauri::command]
pub async fn login(app: AppHandle) -> Result<auth::User> {
    let user = auth::login_with_oauth(&app).await?;
    app.db().save_auth(app.lock_auth().as_ref())?;

    Ok(user)
}

#[tauri::command]
pub async fn logout(app: AppHandle) -> Result<()> {
    let mut auth = app.lock_auth();
    *auth = None;
    app.db().save_auth(auth.as_ref())?;

    Ok(())
}

#[tauri::command]
pub async fn get_user(app: AppHandle) -> Result<Option<auth::User>> {
    let user = auth::user_info(&app);

    Ok(user)
}
