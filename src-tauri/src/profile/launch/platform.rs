use std::{path::PathBuf, process::Command};

use eyre::{bail, ensure, OptionExt, Result};
use log::info;

use crate::{
    game::{Game, Platform},
    prefs::Prefs,
};

pub fn launch_command(platform: Platform, game: Game, prefs: &Prefs) -> Result<Option<Command>> {
    match platform {
        Platform::Steam => steam_command(game, prefs).map(Some),
        Platform::EpicGames => epic_command(game).map(Some),
        _ => Ok(None),
    }
}

fn steam_command(game: Game, prefs: &Prefs) -> Result<Command> {
    let Some(steam) = &game.platforms.steam else {
        bail!("{} is not available on Steam", game.name)
    };

    let steam_path = prefs
        .steam_exe_path
        .as_ref()
        .ok_or_eyre("steam executable path not set")?;

    ensure!(
        steam_path.exists(),
        "steam executable not found at {}",
        steam_path.display()
    );

    let mut command = Command::new(steam_path);
    command.arg("-applaunch").arg(steam.id.to_string());

    Ok(command)
}

fn epic_command(game: Game) -> Result<Command> {
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

pub fn game_dir(platform: Option<Platform>, game: Game, prefs: &Prefs) -> Result<PathBuf> {
    match platform {
        Some(Platform::Steam) => steam_game_dir(game, prefs),
        #[cfg(windows)]
        Some(Platform::XboxStore) => xbox_game_dir(game),
        #[cfg(windows)]
        Some(Platform::EpicGames) => epic_game_dir(game),
        _ => bail!("game directory not found - you may need to specify it in the settings"),
    }
}

fn steam_game_dir(game: Game, prefs: &Prefs) -> Result<PathBuf> {
    let Some(steam) = &game.platforms.steam else {
        bail!("{} is not available on Steam", game.name)
    };

    let mut path = prefs
        .steam_library_dir
        .as_ref()
        .ok_or_eyre("steam library directory not set")?
        .to_path_buf();

    if !path.ends_with("common") {
        if !path.ends_with("steamapps") {
            path.push("steamapps");
        }

        path.push("common");
    }

    info!(
        "using {} path from steam library at {}",
        game.slug,
        path.display()
    );

    path.push(steam.dir_name.unwrap_or(game.name));

    Ok(path)
}

#[cfg(windows)]
fn xbox_game_dir(game: Game) -> Result<PathBuf> {
    use std::process::Command;

    use eyre::Context;

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
fn epic_game_dir(game: &crate::game::GameData<'_>) -> Result<PathBuf, eyre::Error> {
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
