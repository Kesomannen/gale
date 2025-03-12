use eyre::Context;
use itertools::Itertools;
use tauri::AppHandle;

use crate::{state::ManagerExt, util::cmd::Result};

#[tauri::command]
pub fn launch_game(app: AppHandle) -> Result<()> {
    let prefs = app.lock_prefs();
    let manager = app.lock_manager();

    manager.active_game().launch(&prefs, &app)?;

    Ok(())
}

#[tauri::command]
pub fn get_launch_args(app: AppHandle) -> Result<String> {
    let prefs = app.lock_prefs();
    let manager = app.lock_manager();

    let game_dir = super::game_dir(manager.active_game, &prefs)?;
    let (_, command) = manager.active_game().launch_command(&game_dir, &prefs)?;
    let text = command
        .get_args()
        .map(|arg| format!("\"{}\"", arg.to_string_lossy()))
        .join(" ");

    Ok(text)
}

#[tauri::command]
pub fn open_game_dir(app: AppHandle) -> Result<()> {
    let prefs = app.lock_prefs();
    let manager = app.lock_manager();

    let path = super::game_dir(manager.active_game, &prefs)?;
    open::that(path).context("failed to open directory")?;

    Ok(())
}
