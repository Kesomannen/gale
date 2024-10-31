use crate::{
    manager::ModManager,
    prefs::Prefs,
    util::cmd::{Result, StateMutex},
};
use anyhow::Context;
use itertools::Itertools;

#[tauri::command]
pub fn launch_game(manager: StateMutex<ModManager>, prefs: StateMutex<Prefs>) -> Result<()> {
    let manager = manager.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    manager.active_game().launch(&prefs)?;
    Ok(())
}

#[tauri::command]
pub fn get_launch_args(
    manager: StateMutex<ModManager>,
    prefs: StateMutex<Prefs>,
) -> Result<String> {
    let manager = manager.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    let (_, command) = manager.active_game().launch_command(&prefs)?;
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

    let path = manager.active_game.path(&prefs)?;
    open::that(path).context("failed to open directory")?;

    Ok(())
}
