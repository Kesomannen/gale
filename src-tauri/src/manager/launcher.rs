use std::{path::Path, process::Command};

use anyhow::{anyhow, bail, ensure, Result};

use crate::prefs::{PrefValue, Prefs};
use super::ModManager;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use tauri::async_runtime;

pub mod commands;

#[typeshare]
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase", tag = "type", content = "content")]
pub enum LaunchMode {
    #[default]
    Steam,
    Direct {
        instances: u32,
    }
}

impl ModManager {
    pub fn launch_game(&self, prefs: &Prefs) -> Result<()> {
        let preference = match prefs.get_or_err("launch_mode")? {
            PrefValue::LaunchMode(mode) => mode,
            _ => bail!("launch mode not set"),
        };

        match preference {
            LaunchMode::Steam => self.launch_game_steam(prefs),
            LaunchMode::Direct { instances } => self.launch_game_direct(prefs, *instances),
        }
    }

    fn launch_game_steam(&self, prefs: &Prefs) -> Result<()> {
        let steam_path = prefs.get_path_or_err("steam_exe_path")?;
        let steam_path = resolve_path(steam_path, "steam executable")?;

        let mut command = Command::new(steam_path);
        command
            .arg("-applaunch")
            .arg(self.active_game.steam_id.to_string());

        add_bepinex_args(&mut command, &self.active_profile().path)?;
 
        command.spawn()?;

        Ok(())
    }

    fn launch_game_direct(&self, prefs: &Prefs, instances: u32) -> Result<()> {
        let mut game_path = prefs.get_path_or_err("steam_exe_path")?.parent().unwrap().to_path_buf();

        game_path.push("steamapps");
        game_path.push("common");
        game_path.push(&self.active_game.display_name);

        ensure!(game_path.exists(), "game path not found (at {})", game_path.display());
        
        let exe_path = game_path.read_dir()?
            .filter_map(|entry| entry.ok())
            .find(|entry| {
                let file_name = entry.file_name();
                let file_name = file_name.to_string_lossy();
                file_name.ends_with(".exe") && !file_name.contains("UnityCrashHandler")
            })
            .map(|entry| entry.path())
            .ok_or_else(|| anyhow!("game .exe not found"))?;

        let exe_path = resolve_path(&exe_path, "game executable")?;

        let mut command = Command::new(exe_path);

        add_bepinex_args(&mut command, &self.active_profile().path)?;

        match instances {
            0 => bail!("instances must be greater than 0"),
            1 => {
                command.spawn()?;
            },
            _ => {
                async_runtime::spawn(async move {
                    // wait a bit between launches
                    for _ in 0..instances {
                        command.spawn().ok();
                        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                    }
                });
            },
        };

        Ok(())
    }
}

fn add_bepinex_args(command: &mut Command, root_path: &Path) -> Result<()>{
    let mut preloader_path = root_path.to_path_buf();
    preloader_path.push("BepInEx");
    preloader_path.push("core");
    preloader_path.push("BepInEx.Preloader.dll");

    let preloader_path = resolve_path(&preloader_path, "preloader")?;

    command
        .arg("--doorstop-enable")
        .arg("true")
        .arg("--doorstop-target")
        .arg(preloader_path);

    Ok(())
}

fn resolve_path<'a>(path: &'a Path, name: &'static str) -> Result<&'a str> {
    let str = path.to_str();
    if !path.try_exists()? || str.is_none() {
        bail!("{} path could not be resolved", name);
    }
    Ok(str.unwrap())
}
