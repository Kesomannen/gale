use eyre::Context;
use itertools::Itertools;
use tauri::AppHandle;

use super::game_dir;
use crate::{
    prefs::Prefs,
    profile::ModManager,
    util::cmd::{Result, StateMutex},
};

#[tauri::command]
pub fn launch_game(
    manager: StateMutex<ModManager>,
    prefs: StateMutex<Prefs>,
    app: AppHandle,
) -> Result<()> {
    let manager = manager.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    manager.active_game().launch(&prefs, app)?;
    Ok(())
}

#[tauri::command]
pub fn get_launch_args(
    manager: StateMutex<ModManager>,
    prefs: StateMutex<Prefs>,
) -> Result<String> {
    let manager = manager.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    let game_dir = game_dir(manager.active_game, &prefs)?;
    let (_, command) = manager.active_game().launch_command(&game_dir, &prefs)?;
    let text = command
        .get_args()
        .map(|arg| format!("\"{}\"", arg.to_string_lossy()))
        .join(" ");

    Ok(text)
}

#[tauri::command]
pub fn open_game_dir(manager: StateMutex<ModManager>, prefs: StateMutex<Prefs>) -> Result<()> {
    let manager = manager.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    let path = game_dir(manager.active_game, &prefs)?;
    open::that(path).context("failed to open directory")?;

    Ok(())
}
