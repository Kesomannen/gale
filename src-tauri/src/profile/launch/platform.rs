use eyre::{Context, OptionExt, Result, bail, ensure, eyre};
use std::{
    path::{Path, PathBuf},
    process::Command,
};
use tracing::info;

use crate::util;
use crate::{
    game::{
        Game,
        platform::{Platform, Platforms},
    },
    prefs::Prefs,
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

    let mut command = create_base_steam_command()?;

    command.arg("-applaunch").arg(steam.id.to_string());

    util::flatpak::wrap_command_if_needed(&mut command);

    Ok(command)
}

#[cfg(target_os = "windows")]
fn create_base_steam_command() -> Result<Command> {
    use crate::util::fs::PathExt;
    use tracing::warn;

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
            warn!(
                "failed to read steam installation path from registry: {err:#}, using fallback path"
            );

            r"C:\Program Files (x86)\Steam\steam.exe".into()
        }
    };

    let path = path
        .exists_or_none()
        .ok_or_eyre("failed to find Steam installation, is it not installed?")?;

    Ok(Command::new(path))
}

#[cfg(target_os = "windows")]
fn read_steam_registry() -> Result<PathBuf> {
    use tracing::debug;
    use winreg::RegKey;
    use winreg::enums::*;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hklm.open_subkey(r"SOFTWARE\WOW6432Node\Valve\Steam")?;

    debug!("reading InstallPath from {key:?}");

    let path: String = key.get_value("InstallPath")?;

    Ok(PathBuf::from(path))
}

#[cfg(target_os = "linux")]
fn create_base_steam_command() -> Result<Command> {
    use crate::util::fs::PathExt;
    use tracing::debug;

    debug!("checking for steam system installation with which");

    if let Ok(path) = which::which("steam") {
        info!("found steam installation via which: {}", path.display());
        return Ok(Command::new(path));
    }

    let mut flatpak_check = Command::new("flatpak");
    flatpak_check.args(["info", "com.valvesoftware.Steam"]);

    util::flatpak::wrap_command_if_needed(&mut flatpak_check);

    debug!("checking for steam flatpak installation with command {flatpak_check:?}");

    match flatpak_check.output() {
        Ok(output) if output.status.success() => {
            info!("using flatpak steam installation");

            let mut command = Command::new("flatpak");
            command.args(["run", "com.valvesoftware.Steam"]);

            return Ok(command);
        }
        Ok(output) => {
            debug!(
                "flatpak check returned with error code {:?}",
                output.status.code()
            );
        }
        Err(err) => {
            debug!("failed to run flatpak check: {:#}", err);
        }
    }

    debug!("checking for steam.sh script in steam installation directory");

    match locate_steam_script() {
        Ok(path) => {
            info!("found steam.sh script at {}", path.display());
            return Ok(Command::new(path));
        }
        Err(err) => {
            debug!("failed to locate steam.sh script: {:#}", err);
        }
    }

    let path = Path::new("/usr/bin/steam")
        .exists_or_none()
        .ok_or_eyre("failed to find Steam installation, is it not installed?")?;

    info!(
        "using steam installation at fallback path: {}",
        path.display()
    );

    Ok(Command::new(path))
}

#[cfg(target_os = "linux")]
fn locate_steam_script() -> Result<PathBuf> {
    use crate::util::fs::PathExt;

    steamlocate::locate()
        .context("failed to locate steam installation")
        .and_then(|steam_dir| {
            use eyre::eyre;

            steam_dir
                .path()
                .join("steam.sh")
                .exists_or_none()
                .ok_or_else(|| {
                    eyre!(
                        "steam.sh not present in steam install at {}",
                        steam_dir.path().display()
                    )
                })
        })
}

pub fn get_steam_launch_options(app_id: u32) -> Result<serde_json::Value> {
    let app_info = get_steam_app_info(app_id)?;

    app_info
        .get("config")
        .and_then(|config| config.get("launch"))
        .cloned()
        .ok_or_else(|| eyre!("no launch options found for app ID {}", app_id))
}

fn get_steam_app_info(app_id: u32) -> Result<serde_json::Value> {
    use new_vdf_parser::appinfo_vdf_parser::open_appinfo_vdf;
    use serde_json::{Map, Value};

    let steam_dir = steamlocate::locate().context("failed to locate steam installation")?;

    let appinfo_path = steam_dir.path().join("appcache").join("appinfo.vdf");

    ensure!(
        appinfo_path.exists(),
        "steam appinfo.vdf not found at {}",
        appinfo_path.display()
    );

    info!("reading Steam app info from {}", appinfo_path.display());

    let appinfo_vdf: Map<String, Value> = open_appinfo_vdf(&appinfo_path);

    let entries = appinfo_vdf
        .get("entries")
        .and_then(|e| e.as_array())
        .ok_or_eyre("no entries found in appinfo.vdf")?;

    entries
        .iter()
        .find(|entry| {
            entry.get("appid").and_then(serde_json::Value::as_u64) == Some(u64::from(app_id))
        })
        .cloned()
        .ok_or_else(|| eyre!("app ID {} not found in Steam appinfo.vdf", app_id))
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
    locate_dir(platform, &game.platforms, game.name)
}

pub(crate) fn locate_dir(
    platform: Option<Platform>,
    platforms: &Platforms<'_>,
    display_name: &str,
) -> Result<PathBuf> {
    match platform {
        Some(Platform::Steam) => steam_dir(platforms, display_name),
        #[cfg(windows)]
        Some(Platform::XboxStore) => xbox_dir(platforms, display_name),
        #[cfg(windows)]
        Some(Platform::EpicGames) => epic_dir(platforms, display_name),
        _ => bail!(
            "directory not found for {display_name} - the selected platform cannot be located automatically"
        ),
    }
}

fn steam_dir(platforms: &Platforms<'_>, display_name: &str) -> Result<PathBuf> {
    let Some(steam) = &platforms.steam else {
        bail!("{display_name} is not available on Steam");
    };

    let steam_dir = steamlocate::SteamDir::locate().context("failed to find Steam installation")?;
    let (app, library) = steam_dir.find_app(steam.id)?.ok_or_else(|| {
        eyre!(
            "could not find Steam app {} ({display_name}); is it installed?",
            steam.id
        )
    })?;

    Ok(library.resolve_app_dir(&app))
}

#[cfg(windows)]
fn xbox_dir(platforms: &Platforms<'_>, display_name: &str) -> Result<PathBuf> {
    use std::process::Command;

    use eyre::{Context, ensure};

    let Some(xbox) = &platforms.xbox_store else {
        bail!("{display_name} is not available on Xbox Store")
    };

    let name = xbox.identifier.unwrap_or(display_name);
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

    info!("querying path for {display_name} with command {query:?}");

    let out = query.output()?;

    ensure!(
        out.status.success(),
        "query returned with error code {}",
        out.status.code().unwrap_or(-1)
    );

    let value = String::from_utf8(out.stdout).context("query returned invalid UTF-8")?;

    Ok(PathBuf::from(value.trim()))
}

#[cfg(windows)]
fn epic_dir(platforms: &Platforms<'_>, display_name: &str) -> Result<PathBuf> {
    use eyre::Context;
    use serde::Deserialize;

    use crate::util;

    let Some(epic) = &platforms.epic_games else {
        bail!("{display_name} is not available on Epic Games")
    };

    let name = epic.identifier.unwrap_or(display_name);
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
