use core::str;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

use eyre::{bail, ensure, eyre, OptionExt, Result};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tokio::time::Duration;
use tracing::info;

use super::ManagedGame;
use crate::{
    game::Game,
    logger::log_webview_err,
    prefs::{GamePrefs, Prefs},
};

#[cfg(target_os = "linux")]
mod linux;
mod platform;

pub mod commands;

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase", tag = "type", content = "content")]
pub enum LaunchMode {
    #[default]
    #[serde(alias = "steam")]
    Launcher,
    #[serde(rename_all = "camelCase")]
    Direct { instances: u32, interval_secs: f32 },
}

impl ManagedGame {
    pub fn launch(&self, prefs: &Prefs, app: &AppHandle) -> Result<()> {
        let game_dir = locate_game_dir(self.game, prefs)?;

        let profile = self.active_profile();
        self.game
            .mod_loader
            .inner()
            .prepare_launch(&profile.path, &game_dir)?;

        let (launch_mode, command) = self.launch_command(&game_dir, prefs)?;
        info!("launching {} with command {:?}", self.game.slug, command);
        do_launch(command, app, launch_mode)?;

        Ok(())
    }

    fn launch_command(&self, game_dir: &Path, prefs: &Prefs) -> Result<(LaunchMode, Command)> {
        let (launch_mode, mut platform, game_custom_args) = prefs
            .game_prefs
            .get(&*self.game.slug)
            .map(|prefs| {
                (
                    prefs.launch_mode.clone(),
                    prefs.platform,
                    if prefs.custom_args_enabled {
                        prefs.custom_args.as_ref()
                    } else {
                        None
                    },
                )
            })
            .unwrap_or_else(|| {
                info!("game prefs not set, using default settings");
                Default::default()
            });

        // if the game has a platform but the setting is unset, fill it in
        platform = platform.or_else(|| self.game.platforms.iter().next());

        let mut command = match (&launch_mode, platform) {
            // If the setting is `Launcher` and we have a platform, use the platform-specific
            // launch command (if there is one). Otherwise, fall back to direct execution.
            (LaunchMode::Launcher, Some(platform)) => {
                platform::create_launch_command(game_dir, platform, self.game, prefs).transpose()
            }
            _ => None,
        }
        .unwrap_or_else(|| exe_path(game_dir).map(Command::new))?;

        if matches!(launch_mode, LaunchMode::Direct { .. }) {
            command.current_dir(game_dir);
        }

        let profile = self.active_profile();

        let loader_args = self
            .game
            .mod_loader
            .inner()
            .get_launch_args(&profile.path)?;

        command.args(loader_args);

        if let Some(custom_args) = game_custom_args {
            command.args(custom_args);
        }

        if profile.custom_args_enabled {
            command.args(&profile.custom_args);
        }

        Ok((launch_mode, command))
    }
}

fn do_launch(mut command: Command, app: &AppHandle, mode: LaunchMode) -> Result<()> {
    match mode {
        LaunchMode::Launcher | LaunchMode::Direct { instances: 1, .. } => {
            command.spawn()?;
        }
        LaunchMode::Direct { instances: 0, .. } => bail!("instances must be greater than 0"),
        LaunchMode::Direct {
            instances,
            interval_secs,
        } => {
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                for i in 0..instances {
                    if let Err(err) = command.spawn() {
                        log_webview_err(
                            "Failed to launch game",
                            eyre!("Launch command {} failed: {}.", i, err),
                            &app,
                        );
                    }
                    tokio::time::sleep(Duration::from_secs_f32(interval_secs)).await;
                }
            });
        }
    };

    Ok(())
}

fn locate_game_dir(game: Game, prefs: &Prefs) -> Result<PathBuf> {
    let game_prefs = prefs.game_prefs.get(&*game.slug);

    let path = if let Some(GamePrefs {
        dir_override: Some(path),
        ..
    }) = game_prefs
    {
        info!("using game directory override at {}", path.display());
        path.to_path_buf()
    } else {
        let platform = game_prefs
            .and_then(|prefs| prefs.platform)
            .or_else(|| game.platforms.iter().next());

        let path = platform::locate_game_dir(platform, game)?;
        info!(
            "found game directory via platform ({}): {}",
            match &platform {
                Some(platform) => platform.as_ref(),
                None => "none",
            },
            path.display()
        );
        path
    };

    ensure!(
        path.exists(),
        "game directory does not exist, please check your settings (expected at {})",
        path.display()
    );

    Ok(path)
}

fn exe_path(game_dir: &Path) -> Result<PathBuf> {
    game_dir
        .read_dir()?
        .filter_map(Result::ok)
        .find(|entry| {
            let file_name = PathBuf::from(entry.file_name());
            let extension = file_name.extension().and_then(|ext| ext.to_str());

            let has_correct_extension = if cfg!(windows) {
                matches!(extension, Some("exe"))
            } else {
                matches!(extension, Some("exe" | "sh"))
            };

            has_correct_extension && !file_name.to_string_lossy().contains("UnityCrashHandler")
        })
        .map(|entry| entry.path())
        .ok_or_eyre("game executable not found")
}
