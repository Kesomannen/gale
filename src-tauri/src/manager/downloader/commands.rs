use std::fs;

use anyhow::Context;

use crate::{
    command_util::{Result, StateMutex},
    manager::ModManager,
    prefs::Prefs,
    thunderstore::{ModRef, Thunderstore},
};

use super::InstallState;

#[tauri::command]
pub async fn install_mod(mod_ref: ModRef, app: tauri::AppHandle) -> Result<()> {
    super::install_with_deps(&mod_ref, &app).await?;

    Ok(())
}

#[tauri::command]
pub fn cancel_install(install_state: StateMutex<InstallState>) -> Result<()> {
    install_state.lock().unwrap().cancelled = true;

    Ok(())
}

#[tauri::command]
pub fn clear_download_cache(prefs: StateMutex<Prefs>) -> Result<()> {
    let prefs = prefs.lock().unwrap();
    let cache_dir = prefs.get_path_or_err("cache_dir")?;

    if cache_dir.exists() {
        fs::remove_dir_all(cache_dir).context("failed to delete cache dir")?;
    }

    fs::create_dir_all(cache_dir).context("failed to recreate cache dir")?;
    Ok(())
}

#[tauri::command]
pub fn get_download_size(
    mod_ref: ModRef,
    manager: StateMutex<ModManager>,
    thunderstore: StateMutex<Thunderstore>,
    prefs: StateMutex<Prefs>,
) -> Result<u64> {
    let manager = manager.lock().unwrap();
    let thunderstore = thunderstore.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    let result = super::total_download_size(
        mod_ref.borrow(&thunderstore)?,
        manager.active_profile(),
        &prefs,
        &thunderstore,
    );

    Ok(result.unwrap_or(0))
}