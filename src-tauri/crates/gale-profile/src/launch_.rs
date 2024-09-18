use anyhow::{anyhow, bail, ensure, Context};
use gale_core::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use std::{
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::process::Command;

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LaunchMode {
    #[default]
    Steam,
    #[serde(rename_all = "camelCase")]
    Direct {
        instances: usize,
        interval_secs: u64,
    },
}

pub async fn launch(profile_id: i64, state: &AppState) -> Result<()> {
    let profile = sqlx::query!(
        r#"SELECT
            path,
            game_id,
            launch_mode AS "launch_mode: Json<LaunchMode>"
        FROM profiles
        WHERE id = ?"#,
        profile_id
    )
    .fetch_one(&state.db)
    .await?;

    let (game_path, steam_id) = game_info(profile.game_id, state).await?;

    let launch_mode = profile.launch_mode.map(|mode| mode.0).unwrap_or_default();

    match launch_mode {
        LaunchMode::Steam => launch_steam(steam_id, &game_path, state).await?,
        LaunchMode::Direct {
            instances,
            interval_secs,
        } => launch_direct(
            &PathBuf::from(profile.path),
            &game_path,
            instances,
            Duration::from_secs(interval_secs),
        )?,
    }

    Ok(())
}

async fn game_info(game_id: i64, state: &AppState) -> Result<(PathBuf, i64)> {
    let game = sqlx::query!(
        "SELECT
            override_path,
            steam_dir_name,
            steam_id
        FROM games
        WHERE id = ?",
        game_id
    )
    .fetch_one(&state.db)
    .await?;

    if let Some(path) = game.override_path {
        Ok((PathBuf::from(path), game.steam_id))
    } else {
        let mut path: PathBuf = sqlx::query!("SELECT steam_library_path FROM settings")
            .fetch_one(&state.db)
            .await?
            .steam_library_path
            .ok_or(anyhow!("steam library path not set"))?
            .into();

        path.push("steamapps");
        path.push("common");
        path.push(game.steam_dir_name);

        ensure!(
            path.exists(),
            "game directory not found (expected at {})",
            path.display()
        );

        Ok((path, game.steam_id))
    }
}

async fn launch_steam(steam_id: i64, profile_path: &Path, state: &AppState) -> Result<()> {
    let steam_path: PathBuf = sqlx::query!("SELECT steam_executable_path FROM settings")
        .fetch_one(&state.db)
        .await?
        .steam_executable_path
        .ok_or(anyhow!("steam executable path not set"))?
        .into();

    ensure!(
        steam_path.exists(),
        "steam executable not found at {}",
        steam_path.display()
    );

    let mut command = Command::new(steam_path);
    command.arg("-applaunch").arg(steam_id.to_string());

    add_bepinex_args(&mut command, profile_path)?;

    command.spawn()?;

    Ok(())
}

fn launch_direct(
    profile_path: &Path,
    game_path: &Path,
    instances: usize,
    interval: Duration,
) -> Result<()> {
    let exe_path = find_game_exe(game_path)?;

    let mut command = Command::new(exe_path);

    add_bepinex_args(&mut command, profile_path)?;

    match instances {
        0 => bail!("instances must be greater than 0"),
        1 => {
            command.spawn()?;
        }
        _ => {
            tauri::async_runtime::spawn(async move {
                for _ in 0..instances {
                    command.spawn().ok();
                    tokio::time::sleep(interval).await;
                }
            });
        }
    };

    Ok(())
}

fn find_game_exe(game_path: &Path) -> Result<PathBuf> {
    game_path
        .read_dir()?
        .filter_map(|entry| entry.ok())
        .find(|entry| {
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();
            file_name.ends_with(".exe") && !file_name.contains("UnityCrashHandler")
        })
        .map(|entry| entry.path())
        .context("game executable not found")
}

fn add_bepinex_args(command: &mut Command, profile_path: &Path) -> Result<()> {
    let (enable_prefix, target_prefix) = get_doorstop_args(profile_path)?;

    let preloader_path = find_preloader(profile_path)?;

    command
        .args([enable_prefix, "true", target_prefix])
        .arg(preloader_path);

    Ok(())
}

fn find_preloader(path: &Path) -> Result<PathBuf> {
    let mut core_dir = path.to_path_buf();

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
        .ok_or(anyhow!(
            "BepInEx preloader not found. Is BepInEx installed?"
        ))?
        .path();

    Ok(result)
}

fn get_doorstop_args(profile_path: &Path) -> Result<(&'static str, &'static str)> {
    let path = profile_path.join(".doorstop_version");

    let version = match path.exists() {
        true => std::fs::read_to_string(&path)
            .context("failed to read .doorstop_version")?
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
