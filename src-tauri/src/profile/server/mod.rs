use eyre::{Result, ensure};
use tauri::AppHandle;

use crate::state::ManagerExt;

pub mod commands;

pub mod config;
pub mod deploy;
pub mod local;
pub mod manifest;
pub mod remote;
pub mod runtime;
pub mod secrets;

pub fn ensure_profile_unlocked(app: &AppHandle, profile_id: i64) -> Result<()> {
    let runtime = app.lock_server_runtime();

    ensure!(
        !runtime.is_profile_locked(profile_id),
        "this profile is currently in use by a dedicated server"
    );

    Ok(())
}

pub fn ensure_active_profile_unlocked(app: &AppHandle) -> Result<()> {
    let profile_id = {
        let manager = app.lock_manager();
        manager.active_profile().id
    };

    ensure_profile_unlocked(app, profile_id)
}

pub fn ensure_no_pending_installs(app: &AppHandle) -> Result<()> {
    ensure!(
        !app.install_queue().lock().is_processing(),
        "please wait for mod installations to finish before launching the server"
    );

    Ok(())
}
