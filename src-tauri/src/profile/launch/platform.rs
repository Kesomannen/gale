use std::{
    borrow::Cow,
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use eyre::{bail, ensure, Context, OptionExt, Result};
use keyvalues_serde::parser::Vdf;
use serde::Deserialize;
use tracing::{debug, info};

use crate::{
    game::{Game, Platform},
    prefs::Prefs,
};

pub fn launch_command(
    game_dir: &Path,
    platform: Platform,
    game: Game,
    prefs: &Prefs,
) -> Result<Option<Command>> {
    match platform {
        Platform::Steam => steam_command(game_dir, game, prefs).map(Some),
        Platform::EpicGames => epic_command(game).map(Some),
        _ => Ok(None),
    }
}

#[allow(unused_variables)] // allow unused game_dir on windows
fn steam_command(game_dir: &Path, game: Game, prefs: &Prefs) -> Result<Command> {
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
            linux::ensure_wine_override(steam.id as u64, proxy_dll, prefs).unwrap_or_else(|err| {
                warn!("failed to ensure wine dll override: {:#}", err);
            });
        }
    }

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

    let mut path = find_steam_library_for_game(steam.id as u64, prefs)
        .context("failed to find steam library location")?;

    path.push("steamapps");
    path.push("common");
    path.push(steam.dir_name.unwrap_or(game.name));

    info!(
        "using {} path from steam library (at {})",
        game.slug,
        path.display()
    );

    Ok(path)
}

pub fn find_steam_library_for_game(game_id: u64, prefs: &Prefs) -> Result<PathBuf> {
    #[derive(Deserialize, Debug)]
    struct LibraryFolders {
        libraries: Vec<Library>,
    }

    #[derive(Deserialize, Debug)]
    struct Library {
        path: PathBuf,
        apps: HashMap<u64, u64>,
    }

    // we should always base this off the .exe location, since this should have the config folder
    let mut path = default_steam_library_dir(prefs.steam_exe_path.as_deref())
        .ok_or_eyre("steam exe path is not set")?;

    path.push("config");
    path.push("libraryfolders.vdf");

    let file_contents = fs::read_to_string(&path).context("failed to read libraryfolders.vdf")?;
    let mut vdf = Vdf::parse(&file_contents).context("failed to parse libraryfolders.vdf")?;

    debug!("read vdf: {:?}", vdf);

    let obj = vdf.value.get_mut_obj().unwrap();

    let mut index = 0;
    while let Some(mut library) = obj.remove(index.to_string().as_str()) {
        obj.entry(Cow::from("libraries"))
            .or_insert(Vec::new())
            .push(library.pop().unwrap());

        index += 1;
    }

    let folders: LibraryFolders = keyvalues_serde::from_vdf(vdf)?;

    folders
        .libraries
        .into_iter()
        .find(|lib| lib.apps.contains_key(&game_id))
        .map(|lib| lib.path)
        .ok_or_eyre("game is not installed")
}

pub fn default_steam_library_dir(exe_path: Option<&Path>) -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        exe_path.and_then(|exe| exe.parent().map(|path| path.to_path_buf()))
    }

    #[cfg(target_os = "linux")]
    {
        [
            dirs_next::data_dir().map(|data_dir| data_dir.join("Steam")),
            dirs_next::home_dir().map(|home_dir| home_dir.join(".steam/steam")),
        ]
        .into_iter()
        .filter_map(|opt| opt)
        .find(|path| path.exists())
    }
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
