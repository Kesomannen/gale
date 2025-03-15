use core::str;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use eyre::{bail, ensure, eyre, OptionExt, Result};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tokio::time::Duration;

use super::ManagedGame;
use crate::{
    game::Game,
    logger::log_webview_err,
    prefs::{GamePrefs, Prefs},
    util::{
        self,
        fs::{Overwrite, UseLinks},
    },
};

mod linux;
mod mod_loader;
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
        let game_dir = game_dir(self.game, prefs)?;
        if let Err(err) = self.link_files(&game_dir) {
            warn!("failed to link files: {:#}", err);
        }

        let (launch_mode, command) = self.launch_command(&game_dir, prefs)?;
        info!("launching {} with command {:?}", self.game.slug, command);
        do_launch(command, app, launch_mode)?;

        Ok(())
    }

    fn launch_command(&self, game_dir: &Path, prefs: &Prefs) -> Result<(LaunchMode, Command)> {
        let (launch_mode, mut platform, custom_args) = prefs
            .game_prefs
            .get(&*self.game.slug)
            .map(|prefs| {
                (
                    prefs.launch_mode.clone(),
                    prefs.platform,
                    prefs.custom_args.as_ref(),
                )
            })
            .unwrap_or_else(|| {
                info!("game prefs not set, using default settings");
                Default::default()
            });

        // if the game has a platform but the setting is unset, fill it in
        platform = platform.or_else(|| self.game.platforms.iter().next());

        let mut command = match (&launch_mode, platform) {
            (LaunchMode::Launcher, Some(platform)) => {
                platform::launch_command(game_dir, platform, self.game, prefs).transpose()
            }
            _ => None,
        }
        .unwrap_or_else(|| exe_path(game_dir).map(Command::new))?;

        let profile = self.active_profile();

        mod_loader::add_args(&mut command, &profile.path, &self.game.mod_loader)?;

        if let Some(custom_args) = custom_args {
            command.args(custom_args);
        }

        if self.game.server {
            command.arg("--server");
        }

        command.args(["--gale-profile", &profile.name]);

        Ok((launch_mode, command))
    }

    fn link_files(&self, game_dir: &Path) -> Result<()> {
        const EXCLUDES: [&str; 2] = ["profile.json", "mods.yml"];

        let files = self
            .active_profile()
            .path
            .read_dir()?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                return entry.file_type().is_ok_and(|ty| ty.is_file())
                    || entry.file_name() == "dotnet"; // bepinex il2cpp libraries
            })
            .filter(|entry| {
                let name = entry.file_name();
                EXCLUDES.iter().all(|exclude| name != *exclude)
            });

        for file in files {
            info!(
                "copying {} to game directory",
                file.file_name().to_string_lossy()
            );

            if file.file_type().is_ok_and(|ty| ty.is_file()) {
                fs::copy(file.path(), game_dir.join(file.file_name()))?;
            } else {
                util::fs::copy_dir(
                    &file.path(),
                    &game_dir.join(file.file_name()),
                    Overwrite::Yes,
                    UseLinks::No,
                )?;
            }
        }

        Ok(())
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

fn game_dir(game: Game, prefs: &Prefs) -> Result<PathBuf> {
    let game_prefs = prefs.game_prefs.get(&*game.slug);

    let path = if let Some(GamePrefs {
        dir_override: Some(path),
        ..
    }) = game_prefs
    {
        info!("using game path override at {}", path.display());
        path.to_path_buf()
    } else {
        let platform = game_prefs
            .and_then(|prefs| prefs.platform)
            .or_else(|| game.platforms.iter().next());

        platform::game_dir(platform, game, prefs)?
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
