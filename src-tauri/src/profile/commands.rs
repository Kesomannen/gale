use anyhow::Context;
use itertools::Itertools;
use log::warn;
use serde::Serialize;
use uuid::Uuid;

use super::{actions::ActionResult, Dependant, ModManager, Profile};
use crate::{
    game::{self, Game},
    prefs::Prefs,
    thunderstore::{query::QueryModsArgs, FrontendProfileMod, Thunderstore, VersionIdent},
    util::cmd::{Result, StateMutex},
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontendGame {
    name: &'static str,
    slug: &'static str,
    popular: bool,
}

impl From<Game> for FrontendGame {
    fn from(value: Game) -> Self {
        Self {
            name: value.name,
            slug: &*value.slug,
            popular: value.popular,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameInfo {
    all: Vec<FrontendGame>,
    active: FrontendGame,
    favorites: Vec<&'static str>,
}

#[tauri::command]
pub fn get_game_info(manager: StateMutex<ModManager>) -> GameInfo {
    let manager = manager.lock().unwrap();

    let favorites = manager
        .games
        .iter()
        .filter_map(|(game, managed_game)| match managed_game.favorite {
            true => Some(game.name),
            false => None,
        })
        .collect();

    GameInfo {
        all: game::all().map_into().collect(),
        active: manager.active_game.into(),
        favorites,
    }
}

#[tauri::command]
pub fn favorite_game(
    slug: String,
    manager: StateMutex<ModManager>,
    prefs: StateMutex<Prefs>,
) -> Result<()> {
    let mut manager = manager.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    let game = game::from_slug(&slug).context("unknown game")?;
    manager.ensure_game(game, &prefs)?;

    let managed_game = manager.games.get_mut(&game).unwrap();
    managed_game.favorite = !managed_game.favorite;

    manager.save(&prefs)?;

    Ok(())
}

#[tauri::command]
pub fn set_active_game(
    slug: &str,
    app: tauri::AppHandle,
    manager: StateMutex<ModManager>,
    thunderstore: StateMutex<Thunderstore>,
    prefs: StateMutex<Prefs>,
) -> Result<()> {
    let mut manager = manager.lock().unwrap();
    let mut thunderstore = thunderstore.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    let game = game::from_slug(slug).context("unknown game")?;

    manager.set_active_game(game, &mut thunderstore, &prefs, app)?;
    manager.save(&prefs)?;

    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilesInfo {
    profiles: Vec<ProfileInfo>,
    active_index: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileInfo {
    name: String,
    mod_count: usize,
}

#[tauri::command]
pub fn get_profile_info(manager: StateMutex<ModManager>) -> ProfilesInfo {
    let manager = manager.lock().unwrap();
    let game = manager.active_game();

    ProfilesInfo {
        profiles: game
            .profiles
            .iter()
            .map(|profile| ProfileInfo {
                name: profile.name.clone(),
                mod_count: profile.mods.len(),
            })
            .collect(),
        active_index: game.active_profile_index,
    }
}

#[tauri::command]
pub fn set_active_profile(
    index: usize,
    manager: StateMutex<ModManager>,
    prefs: StateMutex<Prefs>,
) -> Result<()> {
    let mut manager = manager.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    manager.active_game_mut().set_active_profile(index)?;
    manager.save(&prefs)?;

    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontendAvailableUpdate {
    full_name: VersionIdent,
    ignore: bool,
    package_uuid: Uuid,
    version_uuid: Uuid,
    old: semver::Version,
    new: semver::Version,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileQuery {
    updates: Vec<FrontendAvailableUpdate>,
    mods: Vec<FrontendProfileMod>,
    unknown_mods: Vec<Dependant>,
}

#[tauri::command]
pub fn query_profile(
    args: QueryModsArgs,
    manager: StateMutex<ModManager>,
    thunderstore: StateMutex<Thunderstore>,
) -> Result<ProfileQuery> {
    let manager = manager.lock().unwrap();
    let thunderstore = thunderstore.lock().unwrap();

    let profile = manager.active_profile();
    let (mods, unknown_mods) = profile.query_mods(&args, &thunderstore);
    let updates = profile
        .mods
        .iter()
        .filter_map(|profile_mod| {
            profile
                .check_update(profile_mod.uuid(), false, &thunderstore)
                .transpose()
        })
        .map_ok(|update| {
            let ignore = profile.ignored_updates.contains(&update.latest.uuid);

            FrontendAvailableUpdate {
                full_name: update.latest.ident.clone(),
                package_uuid: update.package.uuid,
                version_uuid: update.latest.uuid,
                old: update.current.parsed_version().clone(),
                new: update.latest.parsed_version().clone(),
                ignore,
            }
        })
        .collect::<anyhow::Result<Vec<_>>>()
        .unwrap_or_else(|err| {
            warn!("failed to check for updates: {:#}", err);
            Vec::new()
        });

    Ok(ProfileQuery {
        updates,
        mods,
        unknown_mods,
    })
}

#[tauri::command]
pub fn is_mod_installed(uuid: Uuid, manager: StateMutex<ModManager>) -> Result<bool> {
    let manager = manager.lock().unwrap();

    let result = manager.active_profile().has_mod(uuid);

    Ok(result)
}

#[tauri::command]
pub fn create_profile(
    name: String,
    manager: StateMutex<ModManager>,
    prefs: StateMutex<Prefs>,
) -> Result<()> {
    let mut manager = manager.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    manager.active_game_mut().create_profile(name)?;
    manager.save(&prefs)?;

    Ok(())
}

#[tauri::command]
pub fn delete_profile(
    index: usize,
    manager: StateMutex<ModManager>,
    prefs: StateMutex<Prefs>,
) -> Result<()> {
    let mut manager = manager.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    manager.active_game_mut().delete_profile(index, false)?;
    manager.save(&prefs)?;

    Ok(())
}

#[tauri::command]
pub fn rename_profile(
    name: String,
    manager: StateMutex<ModManager>,
    prefs: StateMutex<Prefs>,
) -> Result<()> {
    let mut manager = manager.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    manager.active_profile_mut().rename(name)?;
    manager.save(&prefs)?;

    Ok(())
}

#[tauri::command]
pub fn duplicate_profile(
    name: String,
    manager: StateMutex<ModManager>,
    prefs: StateMutex<Prefs>,
) -> Result<()> {
    let mut manager = manager.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    let game = manager.active_game_mut();
    game.duplicate_profile(name, game.active_profile_index)?;
    manager.save(&prefs)?;

    Ok(())
}

#[tauri::command]
pub fn remove_mod(
    uuid: Uuid,
    manager: StateMutex<ModManager>,
    thunderstore: StateMutex<Thunderstore>,
    prefs: StateMutex<Prefs>,
) -> Result<ActionResult> {
    mod_action_command(manager, thunderstore, prefs, |profile, thunderstore| {
        profile.remove_mod(uuid, thunderstore)
    })
}

#[tauri::command]
pub fn toggle_mod(
    uuid: Uuid,
    manager: StateMutex<ModManager>,
    thunderstore: StateMutex<Thunderstore>,
    prefs: StateMutex<Prefs>,
) -> Result<ActionResult> {
    mod_action_command(manager, thunderstore, prefs, |profile, thunderstore| {
        profile.toggle_mod(uuid, thunderstore)
    })
}

fn mod_action_command<F>(
    manager: StateMutex<ModManager>,
    thunderstore: StateMutex<Thunderstore>,
    prefs: StateMutex<Prefs>,
    action: F,
) -> Result<ActionResult>
where
    F: FnOnce(&mut Profile, &Thunderstore) -> anyhow::Result<ActionResult>,
{
    let mut manager = manager.lock().unwrap();
    let thunderstore = thunderstore.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    let response = action(manager.active_profile_mut(), &thunderstore)?;

    if let ActionResult::Done = response {
        manager.save(&prefs)?;
    }

    Ok(response)
}

#[tauri::command]
pub fn force_remove_mods(
    uuids: Vec<Uuid>,
    manager: StateMutex<ModManager>,
    prefs: StateMutex<Prefs>,
) -> Result<()> {
    let mut manager = manager.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    let profile = manager.active_profile_mut();
    for package_uuid in uuids {
        profile.force_remove_mod(package_uuid)?;
    }

    manager.save(&prefs)?;

    Ok(())
}

#[tauri::command]
pub fn set_all_mods_state(
    enable: bool,
    manager: StateMutex<ModManager>,
    prefs: StateMutex<Prefs>,
) -> Result<()> {
    let mut manager = manager.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    let profile = manager.active_profile_mut();
    let uuids = profile
        .mods
        .iter()
        .filter(|profile_mod| profile_mod.enabled != enable)
        .map(|profile_mod| profile_mod.uuid())
        .collect_vec();

    for uuid in uuids {
        profile.force_toggle_mod(uuid)?;
    }

    manager.save(&prefs)?;

    Ok(())
}

#[tauri::command]
pub fn remove_disabled_mods(
    manager: StateMutex<ModManager>,
    prefs: StateMutex<Prefs>,
) -> Result<usize> {
    let mut manager = manager.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    let profile = manager.active_profile_mut();
    let uuids = profile
        .mods
        .iter()
        .filter(|profile_mod| !profile_mod.enabled)
        .map(|profile_mod| profile_mod.uuid())
        .collect_vec();

    let len = uuids.len();

    for uuid in uuids {
        profile.force_remove_mod(uuid)?;
    }

    manager.save(&prefs)?;

    Ok(len)
}

#[tauri::command]
pub fn force_toggle_mods(
    uuids: Vec<Uuid>,
    manager: StateMutex<ModManager>,
    prefs: StateMutex<Prefs>,
) -> Result<()> {
    let mut manager = manager.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    let profile = manager.active_profile_mut();
    for package_uuid in uuids {
        profile.force_toggle_mod(package_uuid)?;
    }

    manager.save(&prefs)?;

    Ok(())
}

#[tauri::command]
pub fn get_dependants(
    uuid: Uuid,
    manager: StateMutex<ModManager>,
    thunderstore: StateMutex<Thunderstore>,
) -> Result<Vec<VersionIdent>> {
    let manager = manager.lock().unwrap();
    let thunderstore = thunderstore.lock().unwrap();

    let dependants = manager
        .active_profile()
        .dependants(uuid, &thunderstore)
        .map(|profile_mod| profile_mod.ident().into_owned())
        .collect();

    Ok(dependants)
}

#[tauri::command]
pub fn open_profile_dir(manager: StateMutex<ModManager>) -> Result<()> {
    let manager = manager.lock().unwrap();

    let path = &manager.active_profile().path;
    open::that(path).context("failed to open directory")?;

    Ok(())
}

#[tauri::command]
pub fn open_mod_dir(uuid: Uuid, manager: StateMutex<ModManager>) -> Result<()> {
    let manager = manager.lock().unwrap();

    let profile = manager.active_profile();

    profile.scan_mod(profile.get_mod(uuid)?, |path| {
        if path.is_dir() {
            open::that(path).context("failed to open directory")?;
        }

        Ok(())
    })?;

    Ok(())
}

#[tauri::command]
pub fn open_bepinex_log(manager: StateMutex<ModManager>) -> Result<()> {
    let manager = manager.lock().unwrap();

    let path = manager.active_profile().log_path()?;
    open::that(path).context("failed to open log file")?;

    Ok(())
}
