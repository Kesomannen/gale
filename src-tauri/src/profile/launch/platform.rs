use std::{
    path::{Path, PathBuf},
    process::Command,
};

use eyre::{bail, ensure, Context, OptionExt, Result};
use tracing::{info, warn};

use crate::{
    game::{platform::Platform, Game},
    prefs::Prefs,
    util::fs::PathExt,
};

pub fn create_launch_command(
    game_dir: &Path,
    platform: Platform,
    game: Game,
    prefs: &Prefs,
) -> Result<Option<Command>> {
    match platform {
        Platform::Steam => create_steam_command(game_dir, game, prefs).map(Some),
        Platform::EpicGames => create_epic_command(game).map(Some),
        _ => Ok(None),
    }
}

#[allow(unused_variables)] // allow unused game_dir on windows
fn create_steam_command(game_dir: &Path, game: Game, prefs: &Prefs) -> Result<Command> {
    let Some(steam) = &game.platforms.steam else {
        bail!("{} is not available on Steam", game.name)
    };

    #[cfg(target_os = "linux")]
    if let Some(proxy_dll) = game.mod_loader.proxy_dll() {
        use super::linux;
        use tracing::warn;

        if linux::is_proton(game_dir).unwrap_or_else(|err| {
            warn!("failed to determine if game uses proton: {:#}", err);
            false
        }) {
            linux::ensure_wine_override(steam.id as u64, proxy_dll, game_dir).unwrap_or_else(
                |err| {
                    warn!("failed to ensure wine dll override: {:#}", err);
                },
            );
        }
    }

    let mut command = create_base_steam_command()?;
    command.arg("-applaunch").arg(steam.id.to_string());

    Ok(command)
}

#[cfg(target_os = "windows")]
fn create_base_steam_command() -> Result<Command> {
    let path = match read_steam_registry() {
        Ok(install_dir) => {
            let exe_path = install_dir.join("steam.exe");

            info!(
                "read steam installation path from registry: {}",
                exe_path.display()
            );

            exe_path
        }
        Err(err) => {
            warn!("failed to read steam installation path from registry: {err:#}, using fallback path");

            r"C:\Program Files (x86)\Steam\steam.exe".into()
        }
    };

    let path = path
        .exists_or_none()
        .ok_or_eyre("failed to find Steam installation, is it not installed?")?;

    Ok(Command::new(path))
}

#[cfg(target_os = "linux")]
fn create_base_steam_command() -> Result<Command> {
    use tracing::debug;

    if let Ok(path) = which::which("steam") {
        info!("found steam installation via which: {}", path.display());
        return Ok(Command::new(path));
    }

    let mut flatpak_check = Command::new("flatpak");
    flatpak_check.args(["info", "com.valvesoftware.Steam"]);

    debug!("checking for steam flatpak installation with command {flatpak_check:?}");

    if flatpak_check.status().is_ok_and(|status| status.success()) {
        info!("using flatpak steam installation");

        let mut command = Command::new("flatpak");
        command.args(["run", "com.valvesoftware.Steam"]);

        return Ok(command);
    }

    let path = PathBuf::from("/usr/bin/steam")
        .exists_or_none()
        .ok_or_eyre("failed to find Steam installation, is it not installed?")?;

    info!(
        "using steam installation at fallback path: {}",
        path.display()
    );

    Ok(Command::new(path))
}

#[cfg(target_os = "windows")]
fn read_steam_registry() -> Result<PathBuf> {
    use tracing::debug;
    use winreg::enums::*;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hklm.open_subkey(r"SOFTWARE\WOW6432Node\Valve\Steam")?;

    debug!("reading InstallPath from {key:?}");

    let path: String = key.get_value("InstallPath")?;

    Ok(PathBuf::from(path))
}

pub fn get_steam_launch_options(app_id: u32) -> Result<serde_json::Value> {
    let app_info = get_steam_app_info(app_id)?;

    app_info
        .get("config")
        .and_then(|config| config.get("launch"))
        .cloned()
        .ok_or_eyre(format!("No launch options found for app ID {}", app_id))
}

pub fn get_steam_app_info(app_id: u32) -> Result<serde_json::Value> {
    use crate::util::vdf_parser::appinfo_vdf_parser::open_appinfo_vdf;
    use serde_json::{Map, Value};

    let steam_binary = find_steam_binary()?;
    let steam_path = steam_binary
        .parent()
        .ok_or_eyre("Steam binary has no parent directory")?;

    let appinfo_path = steam_path.join("appcache").join("appinfo.vdf");

    ensure!(
        appinfo_path.exists(),
        "Steam appinfo.vdf not found at {}",
        appinfo_path.display()
    );

    info!("Reading Steam app info from {}", appinfo_path.display());

    let appinfo_vdf: Map<String, Value> = open_appinfo_vdf(&appinfo_path);

    let entries = appinfo_vdf
        .get("entries")
        .and_then(|e| e.as_array())
        .ok_or_eyre("No entries found in appinfo.vdf")?;

    entries
        .iter()
        .find(|entry| {
            entry
                .get("appid")
                .and_then(|id| id.as_u64())
                .map_or(false, |id| id == app_id as u64)
        })
        .cloned()
        .ok_or_eyre(format!("App ID {} not found in Steam appinfo.vdf", app_id))
}

fn create_epic_command(game: Game) -> Result<Command> {
    let Some(epic) = &game.platforms.epic_games else {
        bail!("{} is not available on Epic Games", game.name)
    };

    let url = format!(
        "com.epicgames.launcher://apps/{}?action=launch&silent=true",
        epic.identifier.unwrap_or(game.name)
    );

    info!("launching from Epic Games with URL {}", url);

    open::commands(url)
        .into_iter()
        .next()
        .ok_or_eyre("open returned no commands to try")
}

pub fn locate_game_dir(platform: Option<Platform>, game: Game) -> Result<PathBuf> {
    match platform {
        Some(Platform::Steam) => steam_game_dir(game),
        #[cfg(windows)]
        Some(Platform::XboxStore) => xbox_game_dir(game),
        #[cfg(windows)]
        Some(Platform::EpicGames) => epic_game_dir(game),
        _ => bail!("game directory not found - you may need to specify it in the settings"),
    }
}

fn steam_game_dir(game: Game) -> Result<PathBuf> {
    let Some(steam) = &game.platforms.steam else {
        bail!("{} is not available on Steam", game.slug);
    };

    let steam_dir = steamlocate::SteamDir::locate().context("failed to find steam install")?;
    let (app, lib) = steam_dir
        .find_app(steam.id)?
        .ok_or_eyre("could not find app in steam library, is the game not installed?")?;

    Ok(lib.resolve_app_dir(&app))
}

#[cfg(windows)]
fn xbox_game_dir(game: Game) -> Result<PathBuf> {
    use std::process::Command;

    use eyre::{ensure, Context};

    let Some(xbox) = &game.platforms.xbox_store else {
        bail!("{} is not available on Xbox Store", game.name)
    };

    let name = xbox.identifier.unwrap_or(game.name);
    let mut query = Command::new("powershell.exe");
    query.args([
        "get-appxpackage",
        "-Name",
        name,
        "|",
        "select",
        "-expand",
        "InstallLocation",
    ]);

    info!("querying path for {} with command {:?}", game.slug, query);

    let out = query.output()?;

    ensure!(
        out.status.success(),
        "query returned with error code {}",
        out.status.code().unwrap_or(-1)
    );

    let str = String::from_utf8(out.stdout).context("query returned invalid UTF-8")?;

    Ok(PathBuf::from(str))
}

#[cfg(windows)]
fn epic_game_dir(game: Game) -> Result<PathBuf, eyre::Error> {
    use eyre::Context;
    use serde::Deserialize;

    use crate::util;

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

    info!(
        "reading Epic Games installations from {}",
        dat_path.display()
    );

    let list: Vec<ListItem> =
        util::fs::read_json(dat_path).context("failed to read LauncherInstalled.dat file")?;

    list.into_iter()
        .find(|item| item.app_name == name)
        .map(|item| item.install_location)
        .ok_or_eyre("could not find entry in the list of installed games")
}
