use std::fs;

use anyhow::Context;

use crate::{
    command_util::{Result, StateMutex},
    manager::ModManager,
    prefs::Prefs,
    thunderstore::{ModRef, Thunderstore},
};

#[tauri::command]
pub async fn install_mod(mod_ref: ModRef, app: tauri::AppHandle) -> Result<()> {
    super::install_with_deps(&mod_ref, &app).await?;

    Ok(())
}

#[tauri::command]
pub fn clear_download_cache(prefs: StateMutex<Prefs>) -> Result<()> {
    let prefs = prefs.lock().unwrap();

    if prefs.cache_path.try_exists().unwrap_or(false) {
        fs::remove_dir_all(&prefs.cache_path).context("failed to delete cache dir")?;
    }

    fs::create_dir_all(&prefs.cache_path).context("failed to recreate cache dir")?;
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

    Ok(result)
}
