use tauri::AppHandle;

use crate::{state::ManagerExt, util::cmd::Result};

use super::auth;

#[tauri::command]
pub async fn login(provider: auth::OAuthProvider, app: AppHandle) -> Result<auth::User> {
    let user = auth::login_with_oauth(provider, &app).await?;
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
