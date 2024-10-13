use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{anyhow, bail, ensure, Context, Result};
use log::warn;
use serde::{Deserialize, Serialize};
use tokio::time::Duration;
use typeshare::typeshare;

use super::ManagerGame;
use crate::{
    games::Game,
    prefs::{GamePrefs, Prefs},
    util::{error::IoResultExt, fs::PathExt},
};

pub mod commands;

#[typeshare]
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase", tag = "type", content = "content")]
pub enum LaunchMode {
    #[default]
    Steam,
    #[serde(rename_all = "camelCase")]
    Direct { instances: u32, interval_secs: f32 },
}

impl ManagerGame {
    pub fn launch(&self, prefs: &Prefs) -> Result<()> {
        let game_dir = self.game.path(prefs)?;
        if let Err(err) = self.link_files(&game_dir) {
            warn!("failed to link files: {:#}", err);
        }

        let (launch_mode, command) = self.launch_command(prefs)?;

        do_launch(command, launch_mode)?;

        Ok(())
    }

    fn launch_command(&self, prefs: &Prefs) -> Result<(LaunchMode, Command)> {
        let (launch_mode, custom_args) = prefs
            .game_prefs
            .get(&self.game.id)
            .map(|prefs| (prefs.launch_mode.clone(), prefs.custom_args.as_ref()))
            .unwrap_or_default();

        let mut command = match launch_mode {
            LaunchMode::Steam => {
                let steam_path = prefs
                    .steam_exe_path
                    .as_ref()
                    .context("steam executable path not set")?;

                ensure!(
                    steam_path.exists(),
                    "steam executable not found at {}",
                    steam_path.display()
                );

                let mut command = Command::new(steam_path);
                command
                    .arg("-applaunch")
                    .arg(self.game.steam_id.to_string());

                command
            }
            LaunchMode::Direct { .. } => {
                let path = self.game.exe_path(prefs)?;
                Command::new(path)
            }
        };

        add_bepinex_args(&mut command, &self.active_profile().path)?;
        if let Some(custom_args) = custom_args {
            command.args(custom_args);
        }

        Ok((launch_mode, command))
    }

    fn link_files(&self, target: &Path) -> Result<()> {
        const EXCLUDES: [&str; 2] = ["profile.json", "mods.yml"];

        let files = self
            .active_profile()
            .path
            .read_dir()?
            .filter_map(|entry| entry.ok())
            .filter(|entry| match entry.file_type() {
                Ok(file_type) => file_type.is_file(),
                Err(_) => false,
            })
            .filter(|name| {
                let name = name.file_name();
                EXCLUDES.iter().all(|exclude| name != *exclude)
            });

        for file in files {
            fs::copy(file.path(), target.join(file.file_name()))?;
        }

        Ok(())
    }
}

impl Game {
    pub fn path(&self, prefs: &Prefs) -> Result<PathBuf> {
        let path = match prefs.game_prefs.get(&self.id) {
            Some(GamePrefs {
                dir_override: Some(path),
                ..
            }) => path.to_path_buf(),
            _ => {
                let mut path = prefs
                    .steam_library_dir
                    .as_ref()
                    .context("steam library directory not set")?
                    .to_path_buf();

                path.push("steamapps");
                path.push("common");
                path.push(&self.steam_name);

                path
            }
        };

        ensure!(
            path.exists(),
            "game directory not found, please check your settings (expected at {})",
            path.display()
        );

        Ok(path)
    }

    fn exe_path(&self, prefs: &Prefs) -> Result<PathBuf> {
        self.path(prefs)?
            .read_dir()?
            .filter_map(Result::ok)
            .find(|entry| {
                let file_name = PathBuf::from(entry.file_name());
                let extension = file_name.extension().and_then(|ext| ext.to_str());

                matches!(extension, Some("exe" | "sh"))
                    && !file_name.to_string_lossy().contains("UnityCrashHandler")
            })
            .map(|entry| entry.path())
            .and_then(|path| path.exists_or_none())
            .context("game executable not found, try repairing the game through Steam")
    }
}

fn do_launch(mut command: Command, mode: LaunchMode) -> Result<()> {
    match mode {
        LaunchMode::Steam => {
            command.spawn()?;
        }
        LaunchMode::Direct {
            instances,
            interval_secs,
        } => match instances {
            0 => bail!("instances must be greater than 0"),
            1 => {
                command.spawn()?;
            }
            _ => {
                tauri::async_runtime::spawn(async move {
                    for _ in 0..instances {
                        command.spawn().ok();
                        tokio::time::sleep(Duration::from_secs_f32(interval_secs)).await;
                    }
                });
            }
        },
    };

    Ok(())
}

fn add_bepinex_args(command: &mut Command, profile_dir: &Path) -> Result<()> {
    let (enable_prefix, target_prefix) = doorstop_args(profile_dir)?;
    let preloader_path = preloader_path(profile_dir)?;

    command
        .args([enable_prefix, "true", target_prefix])
        .arg(preloader_path);

    Ok(())
}

fn preloader_path(profile_dir: &Path) -> Result<PathBuf> {
    let mut core_dir = profile_dir.to_path_buf();

    core_dir.push("BepInEx");
    core_dir.push("core");

    const PRELOADER_NAMES: [&str; 4] = [
        "BepInEx.Unity.Mono.Preloader.dll",
        "BepInEx.Unity.IL2CPP.dll",
        "BepInEx.Preloader.dll",
        "BepInEx.IL2CPP.dll",
    ];

    let result = core_dir
        .read_dir()
        .map_err(|_| anyhow!("failed to read BepInEx core directory. Is BepInEx installed?"))?
        .filter_map(|entry| entry.ok())
        .find(|entry| {
            let file_name = entry.file_name();
            PRELOADER_NAMES.iter().any(|name| file_name == *name)
        })
        .context("BepInEx preloader not found. Is BepInEx installed?")?
        .path();

    Ok(result)
}

fn doorstop_args(profile_dir: &Path) -> Result<(&'static str, &'static str)> {
    let path = profile_dir.join(".doorstop_version");

    let version = match path.exists() {
        true => fs::read_to_string(&path)
            .fs_context("reading version file", &path)?
            .split('.') // read only the major version number
            .next()
            .and_then(|str| str.parse().ok())
            .context("invalid version format")?,
        false => 3,
    };

    match version {
        3 => Ok(("--doorstop-enable", "--doorstop-target")),
        4 => Ok(("--doorstop-enabled", "--doorstop-target-assembly")),
        vers => bail!("unsupported doorstop version: {}", vers),
    }
}
