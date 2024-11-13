use core::str;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{bail, ensure, Context, Result};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use tokio::time::Duration;

use super::ManagedGame;
use crate::{
    game::{Game, ModLoader, ModLoaderKind, Platform},
    prefs::{GamePrefs, Prefs},
    util::{self, error::IoResultExt},
};

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
    pub fn launch(&self, prefs: &Prefs) -> Result<()> {
        let game_dir = game_dir(self.game, prefs)?;
        if let Err(err) = self.link_files(&game_dir) {
            warn!("failed to link files: {:#}", err);
        }

        let (launch_mode, command) = self.launch_command(&game_dir, prefs)?;
        info!("launching with command {:?}", command);
        do_launch(command, launch_mode)?;

        Ok(())
    }

    fn launch_command(&self, game_dir: &Path, prefs: &Prefs) -> Result<(LaunchMode, Command)> {
        let (launch_mode, platform, custom_args) = prefs
            .game_prefs
            .get(&*self.game.slug)
            .map(|prefs| {
                (
                    prefs.launch_mode.clone(),
                    prefs.platform,
                    prefs.custom_args.as_ref(),
                )
            })
            .unwrap_or_default();

        let mut command = match (&launch_mode, platform) {
            (LaunchMode::Launcher, Some(Platform::Steam)) => steam_command(self.game, prefs)?,
            (LaunchMode::Launcher, Some(Platform::EpicGames)) => {
                let Some(epic) = &self.game.platforms.epic_games else {
                    bail!("{} is not available on Epic Games", self.game.name)
                };

                let path = format!(
                    "com.epicgames.launcher://apps/{}?action=launch&silent=true",
                    epic.identifier.unwrap_or(self.game.name)
                );

                open::commands(path)
                    .into_iter()
                    .next()
                    .context("open returned no commands to try")?
            }
            (_, _) => Command::new(exe_path(game_dir)?),
        };

        add_loader_args(
            &mut command,
            &self.active_profile().path,
            &self.game.mod_loader,
        )?;

        if let Some(custom_args) = custom_args {
            command.args(custom_args);
        }

        Ok((launch_mode, command))
    }

    fn link_files(&self, game_dir: &Path) -> Result<()> {
        const EXCLUDES: [&str; 2] = ["profile.json", "mods.yml"];

        let files = self
            .active_profile()
            .path
            .read_dir()?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_ok_and(|ty| ty.is_file()))
            .filter(|entry| {
                let name = entry.file_name();
                EXCLUDES.iter().all(|exclude| name != *exclude)
            });

        for file in files {
            fs::copy(file.path(), game_dir.join(file.file_name()))?;
        }

        Ok(())
    }
}

fn steam_command(game: Game, prefs: &Prefs) -> Result<Command> {
    let Some(steam) = &game.platforms.steam else {
        bail!("{} is not available on Steam", game.name)
    };

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
    command.arg("-applaunch").arg(steam.id.to_string());

    Ok(command)
}

fn game_dir(game: Game, prefs: &Prefs) -> Result<PathBuf> {
    let game_prefs = prefs.game_prefs.get(&*game.slug);

    let path = if let Some(GamePrefs {
        dir_override: Some(path),
        ..
    }) = game_prefs
    {
        path.to_path_buf()
    } else {
        let platform = game_prefs.and_then(|prefs| prefs.platform);

        match platform {
            Some(Platform::Steam) => {
                let Some(steam) = &game.platforms.steam else {
                    bail!("{} is not available on Steam", game.name)
                };

                let mut path = prefs
                    .steam_library_dir
                    .as_ref()
                    .context("steam library directory not set")?
                    .to_path_buf();

                if !path.ends_with("common") {
                    if !path.ends_with("steamapps") {
                        path.push("steamapps");
                    }

                    path.push("common");
                }

                path.push(steam.dir_name.unwrap_or(game.name));

                path
            }
            #[cfg(windows)]
            Some(Platform::XboxStore) => {
                let Some(xbox) = &game.platforms.xbox_store else {
                    bail!("{} is not available on Xbox Store", game.name)
                };

                let name = xbox.identifier.unwrap_or(game.name);

                let query = Command::new("powershell.exe")
                    .args([
                        "get-appxpackage",
                        "-Name",
                        name,
                        "|",
                        "select",
                        "-expand",
                        "InstallLocation",
                    ])
                    .output()?;

                ensure!(
                    query.status.success(),
                    "query returned with error code {}",
                    query.status.code().unwrap_or(-1)
                );

                let str =
                    String::from_utf8(query.stdout).context("query returned invalid UTF-8")?;
                PathBuf::from(str)
            }
            #[cfg(windows)]
            Some(Platform::EpicGames) => {
                let Some(epic) = &game.platforms.epic_games else {
                    bail!("{} is not available on Epic Games", game.name)
                };

                let name = epic.identifier.unwrap_or(game.name);

                let dat_path: PathBuf =
                    PathBuf::from("C:/ProgramData/Epic/UnrealEngineLauncher/LauncherInstalled.dat");

                #[derive(Debug, Deserialize)]
                #[serde(rename_all = "PascalCase")]
                struct ListItem {
                    install_location: PathBuf,
                    app_name: String,
                }

                let list: Vec<ListItem> = util::fs::read_json(dat_path)
                    .context("failed to read LauncherInstalled.dat file")?;

                list.into_iter()
                    .find(|item| item.app_name == name)
                    .map(|item| item.install_location)
                    .context("could not find entry in the list of installed games")?
            }
            _ => bail!("game directory not set"),
        }
    };

    ensure!(
        path.exists(),
        "game directory not found, please check your settings (expected at {})",
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

            matches!(extension, Some("exe" | "sh"))
                && !file_name.to_string_lossy().contains("UnityCrashHandler")
        })
        .map(|entry| entry.path())
        .context("game executable not found")
}

fn do_launch(mut command: Command, mode: LaunchMode) -> Result<()> {
    match mode {
        LaunchMode::Launcher | LaunchMode::Direct { instances: 1, .. } => {
            command.spawn()?;
        }
        LaunchMode::Direct { instances: 0, .. } => bail!("instances must be greater than 0"),
        LaunchMode::Direct {
            instances,
            interval_secs,
        } => {
            tauri::async_runtime::spawn(async move {
                for _ in 0..instances {
                    command.spawn().ok();
                    tokio::time::sleep(Duration::from_secs_f32(interval_secs)).await;
                }
            });
        }
    };

    Ok(())
}

fn add_loader_args(
    command: &mut Command,
    profile_dir: &Path,
    mod_loader: &ModLoader,
) -> Result<()> {
    match &mod_loader.kind {
        ModLoaderKind::BepInEx { .. } => add_bepinex_args(command, profile_dir),
        ModLoaderKind::MelonLoader { .. } => add_melon_loader_args(command, profile_dir),
        ModLoaderKind::Northstar {} => add_northstar_args(command, profile_dir),
        ModLoaderKind::GDWeave {} => add_gd_weave_args(command, profile_dir),
        ModLoaderKind::Shimloader {} => add_shimloader_args(command, profile_dir),
    }
}

fn add_bepinex_args(command: &mut Command, profile_dir: &Path) -> Result<()> {
    let (enable_prefix, target_prefix) = doorstop_args(profile_dir)?;
    let preloader_path = bepinex_preloader_path(profile_dir)?;

    command
        .args([enable_prefix, "true", target_prefix])
        .arg(preloader_path);

    Ok(())
}

fn bepinex_preloader_path(profile_dir: &Path) -> Result<PathBuf> {
    let mut core_dir = profile_dir.to_path_buf();

    core_dir.push("BepInEx");
    core_dir.push("core");

    const PRELOADER_NAMES: &[&str] = &[
        "BepInEx.Unity.Mono.Preloader.dll",
        "BepInEx.Unity.IL2CPP.dll",
        "BepInEx.Preloader.dll",
        "BepInEx.IL2CPP.dll",
    ];

    let result = core_dir
        .read_dir()
        .context("failed to read BepInEx core directory. Is BepInEx installed?")?
        .filter_map(|entry| entry.ok())
        .find(|entry| {
            let file_name = entry.file_name();
            PRELOADER_NAMES.iter().any(|name| file_name == **name)
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

fn add_melon_loader_args(command: &mut Command, profile_dir: &Path) -> Result<()> {
    command.arg("--melonloader.basedir").arg(profile_dir);

    let agf_path: PathBuf = ["MelonLoader", "Managed", "Assembly-CSharp.dll"]
        .iter()
        .collect();

    if !profile_dir.join(agf_path).exists() {
        command.arg("--melonloader.agfregenerate");
    }

    Ok(())
}

fn add_northstar_args(command: &mut Command, profile_dir: &Path) -> Result<()> {
    let path = profile_dir.join("R2Northstar");
    let path = path.to_str().context("profile path is not valid UTF-8")?;

    command.arg("-northstar").arg(format!("-profile={}", path));

    Ok(())
}

fn add_gd_weave_args(command: &mut Command, profile_dir: &Path) -> Result<()> {
    let path = profile_dir.join("GDWeave");
    let path = path.to_str().context("profile path is not valid UTF-8")?;

    command.arg(format!("--gdweave-folder-override={}", path));

    Ok(())
}

fn add_shimloader_args(command: &mut Command, profile_dir: &Path) -> Result<()> {
    let path = profile_dir.join("shimloader");

    command
        .arg("--mod-dir")
        .arg(path.join("mod"))
        .arg("--pak-dir")
        .arg(path.join("pak"))
        .arg("--cfg-dir")
        .arg(path.join("cfg"));

    Ok(())
}
