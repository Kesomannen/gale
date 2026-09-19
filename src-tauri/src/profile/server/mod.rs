use eyre::{Result, ensure};
use tauri::AppHandle;

use crate::state::ManagerExt;

pub mod commands;

pub(crate) mod args;
pub(crate) mod deploy;
pub(crate) mod local;
pub(crate) mod manifest;
pub(crate) mod paths;
pub(crate) mod remote;
pub(crate) mod runtime;
pub(crate) mod secrets;
pub(crate) mod settings;
pub(crate) mod spec;

pub(crate) fn ensure_profile_unlocked(app: &AppHandle, profile_id: i64) -> Result<()> {
    let runtime = app.lock_server_runtime();

    ensure!(
        !runtime.is_profile_locked(profile_id),
        "this profile is currently in use by a dedicated server"
    );

    Ok(())
}

pub(crate) fn ensure_active_profile_unlocked(app: &AppHandle) -> Result<()> {
    let profile_id = {
        let manager = app.lock_manager();
        manager.active_profile().id
    };

    ensure_profile_unlocked(app, profile_id)
}
