use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{anyhow, bail, ensure, Context, Result};
use log::warn;
use serde::{Deserialize, Serialize};
use tauri::async_runtime;
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
    pub fn launch(&self, vanilla: bool, prefs: &Prefs) -> Result<()> {
        let game_dir = self.game.path(prefs)?;
        if let Err(err) = self.link_files(&game_dir) {
            warn!("failed to link files: {:#}", err);
        }

        let launch_mode = prefs
            .game_prefs
            .get(&self.game.id)
            .map(|prefs| prefs.launch_mode.clone())
            .unwrap_or_default();

        match launch_mode {
            LaunchMode::Steam => self.launch_steam(vanilla, prefs),
            LaunchMode::Direct {
                instances,
                interval_secs,
            } => self.launch_direct(
                vanilla,
                instances,
                Duration::from_secs_f32(interval_secs),
                prefs,
            ),
        }
    }

    fn launch_steam(&self, vanilla: bool, prefs: &Prefs) -> Result<()> {
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

        add_bepinex_args(&mut command, vanilla, &self.active_profile().path)?;

        command.spawn()?;

        Ok(())
    }

    fn launch_direct(
        &self,
        vanilla: bool,
        instances: u32,
        interval: Duration,
        prefs: &Prefs,
    ) -> Result<()> {
        let exe_path = self
            .game
            .path(prefs)?
            .read_dir()?
            .filter_map(|entry| entry.ok())
            .find(|entry| {
                let file_name = entry.file_name();
                let file_name = file_name.to_string_lossy();
                file_name.ends_with(".exe") && !file_name.contains("UnityCrashHandler")
            })
            .map(|entry| entry.path())
            .and_then(|path| path.exists_or_none())
            .context("game executable not found")?;

        let mut command = Command::new(exe_path);

        add_bepinex_args(&mut command, vanilla, &self.active_profile().path)?;

        match instances {
            0 => bail!("instances must be greater than 0"),
            1 => {
                command.spawn()?;
            }
            _ => {
                async_runtime::spawn(async move {
                    // wait a bit between launches
                    for _ in 0..instances {
                        command.spawn().ok();
                        tokio::time::sleep(interval).await;
                    }
                });
            }
        };

        Ok(())
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
}

fn add_bepinex_args(command: &mut Command, vanilla: bool, path: &Path) -> Result<()> {
    let (enable_label, target_label) =
        match doorstop_version(path).context("failed to determine doorstop version")? {
            3 => ("--doorstop-enable", "--doorstop-target"),
            4 => ("--doorstop-enabled", "--doorstop-target-assembly"),
            vers => bail!("unsupported doorstop version: {}", vers),
        };

    if vanilla {
        command.args([enable_label, "false"]);
        return Ok(());
    }

    let mut core_dir = path.to_path_buf();
    core_dir.push("BepInEx");
    core_dir.push("core");

    const PRELOADER_NAMES: [&str; 4] = [
        "BepInEx.Unity.Mono.Preloader.dll",
        "BepInEx.Unity.IL2CPP.dll",
        "BepInEx.Preloader.dll",
        "BepInEx.IL2CPP.dll",
    ];

    let preloader_path = core_dir
        .read_dir()
        .map_err(|_| anyhow!("failed to read BepInEx core directory. Is BepInEx installed?"))?
        .filter_map(|entry| entry.ok())
        .find(|entry| {
            let file_name = entry.file_name();
            PRELOADER_NAMES.iter().any(|name| file_name == *name)
        })
        .map(|entry| entry.path());

    if let Some(preloader_path) = preloader_path {
        command
            .args([enable_label, "true", target_label])
            .arg(preloader_path);

        Ok(())
    } else {
        bail!("BepInEx preloader not found. Is BepInEx installed?",)
    }
}

fn doorstop_version(root_path: &Path) -> Result<u32> {
    let path = root_path.join(".doorstop_version");

    match path.exists() {
        true => fs::read_to_string(&path)
            .fs_context("reading version file", &path)?
            .split('.') // read only the major version number
            .next()
            .and_then(|str| str.parse().ok())
            .context("invalid version format"),
        false => Ok(3),
    }
}
