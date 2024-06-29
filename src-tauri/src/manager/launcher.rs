use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{anyhow, bail, ensure, Context, Result};

use super::ManagerGame;
use crate::{
    games::Game,
    prefs::Prefs,
    util::error::IoResultExt,
};
use log::info;
use serde::{Deserialize, Serialize};
use tauri::async_runtime;
use typeshare::typeshare;

pub mod commands;

#[typeshare]
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase", tag = "type", content = "content")]
pub enum LaunchMode {
    #[default]
    Steam,
    Direct {
        instances: u32,
    },
}

impl ManagerGame {
    pub fn launch(&self, prefs: &Prefs) -> Result<()> {
        let game_dir = self.game.path(prefs)?;
        self.link_files(&game_dir).context("failed to link files")?;

        match &prefs.launch_mode {
            LaunchMode::Steam => self.launch_steam(prefs),
            LaunchMode::Direct { instances } => self.launch_direct(prefs, *instances),
        }
    }

    fn launch_steam(&self, prefs: &Prefs) -> Result<()> {
        let steam_path = prefs.steam_exe_path.as_ref().context("steam exe path not set")?;
        let steam_path = resolve_path(steam_path, "steam executable")?;

        let mut command = Command::new(steam_path);
        command
            .arg("-applaunch")
            .arg(self.game.steam_id.to_string());

        add_bepinex_args(&mut command, &self.active_profile().path)?;

        info!("launching from steam with command: {:?}", command);

        command.spawn()?;

        Ok(())
    }

    fn launch_direct(&self, prefs: &Prefs, instances: u32) -> Result<()> {
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
            .ok_or_else(|| anyhow!("game executable not found"))?;

        let exe_path = resolve_path(&exe_path, "game executable")?;

        let mut command = Command::new(exe_path);

        add_bepinex_args(&mut command, &self.active_profile().path)?;

        info!("launching locally with command: {:?}", command);

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
                        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
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
        let path = match prefs.game_dir_overrides.get(&self.id) {
            Some(path) => path.to_path_buf(),
            None => {
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
            "game path not found (at {}), please check your settings",
            path.display()
        );

        Ok(path)
    }
}

fn add_bepinex_args(command: &mut Command, path: &Path) -> Result<()> {
    let mut preloader_path = path.to_path_buf();
    preloader_path.push("BepInEx");
    preloader_path.push("core");
    preloader_path.push("BepInEx.Preloader.dll");

    let preloader_path = resolve_path(&preloader_path, "preloader")
        .map_err(|_| anyhow!("failed to resolve BepInEx preloader path, is BepInEx installed?"))?;

    let (enable_name, target_name) =
        match doorstop_version(path).context("failed to determine doorstop version")? {
            3 => ("--dorstop-enable", "--doorstop-target"),
            4 => ("--doorstop-enabled", "--doorstop-target-assembly"),
            vers => bail!("unsupported doorstop version: {}", vers),
        };

    command.args([enable_name, "true", target_name, preloader_path]);

    Ok(())
}

fn resolve_path<'a>(path: &'a Path, name: &'static str) -> Result<&'a str> {
    let str = path.to_str();
    if !path.try_exists()? || str.is_none() {
        bail!("{} path could not be resolved", name);
    }
    Ok(str.unwrap())
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
