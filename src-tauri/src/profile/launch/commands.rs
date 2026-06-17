use eyre::Context;
use tauri::{command, AppHandle};

use crate::{profile::sync, state::ManagerExt, util::cmd::Result};

#[command]
pub async fn launch_game(app: AppHandle, vanilla: bool) -> Result<()> {
    if app.lock_prefs().pull_before_launch {
        sync::pull_profile(false, &app).await?;
    }

    let prefs = app.lock_prefs();
    let manager = app.lock_manager();

    manager.active_game().launch(vanilla, &prefs, &app)?;

    Ok(())
}

#[command]
pub fn get_launch_args(app: AppHandle) -> Result<String> {
    let prefs = app.lock_prefs();
    let manager = app.lock_manager();

    let game = manager.active_game();
    let game_dir = super::locate_game_dir(game.game, &prefs)?;
    let (_, command) = game.launch_command(false, &game_dir, &prefs)?;

    let text = shell_words::join(command.get_args().map(|arg| arg.to_string_lossy()));

    Ok(text)
}

#[command]
pub fn open_game_dir(app: AppHandle) -> Result<()> {
    let prefs = app.lock_prefs();
    let manager = app.lock_manager();

    let path = super::locate_game_dir(manager.active_game, &prefs)?;
    open::that(path).context("failed to open directory")?;

    Ok(())
}
