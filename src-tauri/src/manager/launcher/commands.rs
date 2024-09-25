use crate::{
    manager::ModManager,
    prefs::Prefs,
    util::cmd::{Result, StateMutex},
};
use anyhow::Context;
use itertools::Itertools;
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;

#[tauri::command]
pub fn launch_game(manager: StateMutex<ModManager>, prefs: StateMutex<Prefs>) -> Result<()> {
    let manager = manager.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    manager.active_game().launch(&prefs)?;
    Ok(())
}

#[tauri::command]
pub fn copy_launch_args(
    app: AppHandle,
    manager: StateMutex<ModManager>,
    prefs: StateMutex<Prefs>,
) -> Result<()> {
    let manager = manager.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    let (_, command) = manager.active_game().get_launch_command(&prefs)?;
    let text = command
        .get_args()
        .map(|arg| arg.to_string_lossy())
        .join(" ");

    app.clipboard()
        .write_text(text)
        .context("failed to copy launch args")?;

    Ok(())
}
