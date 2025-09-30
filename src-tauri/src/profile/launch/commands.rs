use eyre::Context;
use itertools::Itertools;
use serde::Serialize;
use tauri::{command, AppHandle};

use crate::{profile::sync, state::ManagerExt, util::cmd::Result};

#[derive(Debug, Clone, Serialize)]
pub struct LaunchOption {
    pub arguments: String,
    #[serde(rename = "type")]
    pub launch_type: String,
    pub description: Option<String>,
}

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
pub fn get_steam_launch_options(app: AppHandle) -> Result<Vec<LaunchOption>> {
    let manager = app.lock_manager();
    let managed_game = manager.active_game();
    let game_name = &managed_game.game.name;
    let Some(steam) = &managed_game.game.platforms.steam else {
        return Err(eyre::eyre!("{} is not available on Steam", game_name).into());
    };
    let raw_options = super::platform::get_steam_launch_options(steam.id)
        .context("failed to get Steam launch options")?;

    let mut launch_options = Vec::new();

    if let Some(options_obj) = raw_options.as_object() {
        for (_, option_value) in options_obj.iter() {
            if let Some(option) = option_value.as_object() {
                let option_type = option
                    .get("type")
                    .and_then(|t| t.as_str())
                    .unwrap_or("undefined");

                let arguments = option
                    .get("arguments")
                    .and_then(|a| a.as_str())
                    .unwrap_or("")
                    .to_string();

                let description = option
                    .get("description")
                    .and_then(|d| d.as_str())
                    .map(|s| s.to_string());

                launch_options.push(LaunchOption {
                    arguments,
                    launch_type: option_type.to_string(),
                    description,
                });
            }
        }
    }

    Ok(launch_options)
}
