use tauri::{AppHandle, command};

use crate::{state::ManagerExt, util::cmd::Result};

use super::{
    ConfigUpdatePolicy, ListedSyncProfile, SyncProfileMetadata,
    apply::{ConfigApplyReport, ConfigReviewState},
    auth,
    publish::{PublishMode, SyncConfigFileInfo},
};
use crate::profile::export::ConfigPath;

#[command]
pub async fn read_sync_profile(id: String, app: AppHandle) -> Result<SyncProfileMetadata> {
    let meta = super::read_profile(&id, &app).await?;

    Ok(meta)
}

#[command]
pub async fn create_sync_profile(profile_id: i64, app: AppHandle) -> Result<String> {
    let id = super::publish::create_profile(&app, profile_id).await?;

    Ok(id)
}

#[command]
pub async fn push_sync_profile(mode: PublishMode, profile_id: i64, app: AppHandle) -> Result<()> {
    super::publish::publish_profile(&app, profile_id, mode).await?;

    Ok(())
}

#[command]
pub async fn get_sync_config_files(
    profile_id: i64,
    app: AppHandle,
) -> Result<Vec<SyncConfigFileInfo>> {
    let files = super::publish::list_config_files(&app, profile_id).await?;

    Ok(files)
}

#[command]
pub async fn clone_sync_profile(id: String, name: String, app: AppHandle) -> Result<()> {
    super::clone_profile(&id, Some(name), &app).await?;

    Ok(())
}

#[command]
pub async fn disconnect_sync_profile(delete: bool, profile_id: i64, app: AppHandle) -> Result<()> {
    super::disconnect_profile(delete, profile_id, &app).await?;

    Ok(())
}

#[command]
pub async fn delete_sync_profile(id: String, app: AppHandle) -> Result<()> {
    super::delete_profile(&id, &app).await?;

    Ok(())
}

#[command]
pub async fn pull_sync_profile(profile_id: i64, app: AppHandle) -> Result<ConfigApplyReport> {
    crate::profile::server::ensure_profile_unlocked(&app, profile_id)?;

    let report = super::pull_profile(false, profile_id, &app).await?;

    Ok(report)
}

#[command]
pub async fn fetch_sync_profile(profile_id: i64, app: AppHandle) -> Result<()> {
    super::pull_profile(true, profile_id, &app).await?;

    Ok(())
}

#[command]
pub async fn get_pending_sync_config(profile_id: i64, app: AppHandle) -> Result<ConfigReviewState> {
    let items = super::pending_config_items(profile_id, &app)?;

    Ok(items)
}

#[command]
pub async fn decline_sync_config(
    files: Vec<ConfigPath>,
    remember: bool,
    profile_id: i64,
    app: AppHandle,
) -> Result<()> {
    super::decline_selected_config(&files, remember, profile_id, &app)?;

    Ok(())
}

#[command]
pub async fn apply_sync_config(
    files: Vec<ConfigPath>,
    remember: bool,
    restore_deleted: Vec<ConfigPath>,
    profile_id: i64,
    app: AppHandle,
) -> Result<Vec<ConfigPath>> {
    crate::profile::server::ensure_profile_unlocked(&app, profile_id)?;

    let written =
        super::apply_selected_config(files, remember, restore_deleted, profile_id, &app).await?;

    Ok(written)
}

#[command]
pub async fn set_sync_config_policy(
    file: ConfigPath,
    policy: ConfigUpdatePolicy,
    profile_id: i64,
    app: AppHandle,
) -> Result<()> {
    super::set_config_policy(file, policy, profile_id, &app)?;

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
