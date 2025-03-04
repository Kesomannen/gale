use tauri::AppHandle;

use crate::util::cmd::Result;

use super::auth;

#[tauri::command]
pub async fn login(provider: super::OAuthProvider, app: AppHandle) -> Result<auth::User> {
    let user = super::login_with_oauth(provider, app).await?;

    Ok(user)
}

#[tauri::command]
pub async fn logout(app: AppHandle) -> Result<()> {
    super::logout(app).await?;

    Ok(())
}

#[tauri::command]
pub async fn get_user(app: AppHandle) -> Result<Option<auth::User>> {
    let user = super::user_info(&app);

    Ok(user)
}
