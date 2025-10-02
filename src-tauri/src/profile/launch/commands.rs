use eyre::Context;
use itertools::Itertools;
use tauri::{command, AppHandle};

use crate::{profile::sync, state::ManagerExt, util::cmd::Result};

#[command]
pub async fn launch_game(app: AppHandle, args: Option<String>) -> Result<()> {
    if app.lock_prefs().pull_before_launch {
        sync::pull_profile(false, &app).await?;
    }

    let prefs = app.lock_prefs();
    let manager = app.lock_manager();

    manager.active_game().launch_with_args(&prefs, &app, args)?;

    Ok(())
}

#[command]
pub fn get_launch_args(app: AppHandle) -> Result<String> {
    let prefs = app.lock_prefs();
    let manager = app.lock_manager();

    let game_dir = super::locate_game_dir(manager.active_game, &prefs)?;
    let (_, command) = manager.active_game().launch_command(&game_dir, &prefs)?;
    let text = command
        .get_args()
        .map(|arg| format!("\"{}\"", arg.to_string_lossy()))
        .join(" ");

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

#[command]
pub fn get_steam_launch_options(app: AppHandle) -> Result<Vec<super::LaunchOption>> {
    let manager = app.lock_manager();
    let managed_game = manager.active_game();
    let game_name = &managed_game.game.name;
    let Some(steam) = &managed_game.game.platforms.steam else {
        return Err(eyre::eyre!("{} is not available on Steam", game_name).into());
    };

    Ok(super::parse_steam_launch_options(steam.id)?)
}
