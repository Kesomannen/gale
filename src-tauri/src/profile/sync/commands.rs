use gale_util::cmd::Result;
use tauri::{command, AppHandle};

use crate::state::ManagerExt;

use super::{auth, ListedSyncProfile, SyncProfileMetadata};

#[command]
pub async fn read_sync_profile(id: String, app: AppHandle) -> Result<SyncProfileMetadata> {
    let meta = super::read_profile(&id, &app).await?;

    Ok(meta)
}

#[command]
pub async fn create_sync_profile(app: AppHandle) -> Result<String> {
    let id = super::create_profile(&app).await?;

    Ok(id)
}

#[command]
pub async fn push_sync_profile(app: AppHandle) -> Result<()> {
    super::push_profile(&app).await?;

    Ok(())
}

#[command]
pub async fn clone_sync_profile(id: String, name: String, app: AppHandle) -> Result<()> {
    super::clone_profile(&id, Some(name), &app).await?;

    Ok(())
}

#[command]
pub async fn disconnect_sync_profile(delete: bool, app: AppHandle) -> Result<()> {
    super::disconnect_profile(delete, &app).await?;

    Ok(())
}

#[command]
pub async fn delete_sync_profile(id: String, app: AppHandle) -> Result<()> {
    super::delete_profile(&id, &app).await?;

    Ok(())
}

#[command]
pub async fn pull_sync_profile(app: AppHandle) -> Result<()> {
    super::pull_profile(false, &app).await?;

    Ok(())
}

#[command]
pub async fn fetch_sync_profile(app: AppHandle) -> Result<()> {
    super::pull_profile(true, &app).await?;

    Ok(())
}

#[command]
pub async fn get_owned_sync_profiles(app: AppHandle) -> Result<Vec<ListedSyncProfile>> {
    let results = super::get_owned_profiles(&app).await?;

    Ok(results)
}

#[command]
pub async fn login(app: AppHandle) -> Result<auth::User> {
    let user = auth::login_with_oauth(&app).await?;

    Ok(user)
}

#[command]
pub async fn logout(app: AppHandle) -> Result<()> {
    app.sync_auth().set_creds(None, app.db())?;

    Ok(())
}

#[command]
pub async fn get_user(app: AppHandle) -> Result<Option<auth::User>> {
    let user = auth::user_info(&app);

    Ok(user)
}
