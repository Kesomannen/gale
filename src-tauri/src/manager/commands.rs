use anyhow::{anyhow, Context};
use serde::Serialize;
use typeshare::typeshare;
use uuid::Uuid;

use crate::{
    games::{self, Game, GAMES},
    prefs::Prefs,
    thunderstore::{models::FrontendProfileMod, query::QueryModsArgs, Thunderstore},
    util::cmd::{Result, StateMutex},
};

use super::{Dependant, ModActionResponse, ModManager, Profile};
use itertools::Itertools;

#[typeshare]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameInfo<'a> {
    all: &'a [Game],
    active: &'a Game,
    favorites: Vec<&'a str>,
}

#[tauri::command]
pub fn get_game_info(manager: StateMutex<ModManager>) -> GameInfo<'static> {
    let manager = manager.lock().unwrap();

    let favorites = manager
        .games
        .iter()
        .filter_map(|(id, game)| match game.favorite {
            true => Some(id.as_str()),
            false => None,
        })
        .collect();

    GameInfo {
        all: &*GAMES,
        active: manager.active_game,
        favorites,
    }
}

#[tauri::command]
pub fn favorite_game(
    id: String,
    manager: StateMutex<ModManager>,
    prefs: StateMutex<Prefs>,
) -> Result<()> {
    let mut manager = manager.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    let game = games::from_id(&id).context("invalid game id")?;
    manager.ensure_game(game, &prefs)?;

    let manager_game = manager.games.get_mut(&id).unwrap();
    manager_game.favorite = !manager_game.favorite;

    save(&manager, &prefs)?;

    Ok(())
}

#[tauri::command]
pub fn set_active_game(
    id: &str,
    app: tauri::AppHandle,
    manager: StateMutex<ModManager>,
    thunderstore: StateMutex<Thunderstore>,
    prefs: StateMutex<Prefs>,
) -> Result<()> {
    let mut manager = manager.lock().unwrap();
    let mut thunderstore = thunderstore.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    let game = games::from_id(id).context("invalid game id")?;

    manager.set_active_game(game, &mut thunderstore, &prefs, app)?;
    save(&manager, &prefs)?;

    Ok(())
}

#[typeshare]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilesInfo {
    profiles: Vec<ProfileInfo>,
    active_index: usize,
}

#[typeshare]
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
            .map(|p| ProfileInfo {
                name: p.name.clone(),
                mod_count: p.mods.len(),
            })
            .collect(),
        active_index: game.active_profile_index,
    }
}

#[tauri::command]
pub fn set_active_profile(
    index: usize,
    manager: StateMutex<ModManager>,
    thunderstore: StateMutex<Thunderstore>,
    prefs: StateMutex<Prefs>,
) -> Result<()> {
    let mut manager = manager.lock().unwrap();
    let thunderstore = thunderstore.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    manager
        .active_game_mut()
        .set_active_profile(index, Some(&thunderstore))?;
    save(&manager, &prefs)?;

    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontendAvailableUpdate {
    pub full_name: String,
    pub ignore: bool,
    pub package_uuid: Uuid,
    pub version_uuid: Uuid,
    pub old: semver::Version,
    pub new: semver::Version,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileQuery {
    pub updates: Vec<FrontendAvailableUpdate>,
    pub mods: Vec<FrontendProfileMod>,
    pub unknown_mods: Vec<Dependant>,
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
            let ignore = profile.ignored_updates.contains(&update.latest.uuid4);

            Ok::<_, anyhow::Error>(FrontendAvailableUpdate {
                full_name: update.latest.full_name.clone(),
                package_uuid: update.package.uuid4,
                version_uuid: update.latest.uuid4,
                old: update.current.version_number.clone(),
                new: update.latest.version_number.clone(),
                ignore,
            })
        })
        .flatten_ok()
        .collect::<anyhow::Result<Vec<_>>>()
        .unwrap_or_else(|err| {
            log::warn!("failed to check for updates: {:#}", err);
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

    let result = manager.active_profile().has_mod(&uuid);

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
    save(&manager, &prefs)?;

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
    save(&manager, &prefs)?;

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
    save(&manager, &prefs)?;

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
    save(&manager, &prefs)?;

    Ok(())
}

#[tauri::command]
pub fn remove_mod(
    uuid: Uuid,
    manager: StateMutex<ModManager>,
    thunderstore: StateMutex<Thunderstore>,
    prefs: StateMutex<Prefs>,
) -> Result<ModActionResponse> {
    mod_action_command(
        uuid,
        manager,
        thunderstore,
        prefs,
        |profile, uuid, thunderstore| profile.remove_mod(uuid, thunderstore),
    )
}

#[tauri::command]
pub fn reorder_mod(
    uuid: Uuid,
    delta: i32,
    manager: StateMutex<ModManager>,
    prefs: StateMutex<Prefs>,
) -> Result<()> {
    let mut manager = manager.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    manager.active_profile_mut().reorder_mod(&uuid, delta)?;

    save(&manager, &prefs)?;

    Ok(())
}

#[tauri::command]
pub fn toggle_mod(
    uuid: Uuid,
    manager: StateMutex<ModManager>,
    thunderstore: StateMutex<Thunderstore>,
    prefs: StateMutex<Prefs>,
) -> Result<ModActionResponse> {
    mod_action_command(
        uuid,
        manager,
        thunderstore,
        prefs,
        |profile, uuid, thunderstore| profile.toggle_mod(uuid, thunderstore),
    )
}

fn mod_action_command<F>(
    uuid: Uuid,
    manager: StateMutex<ModManager>,
    thunderstore: StateMutex<Thunderstore>,
    prefs: StateMutex<Prefs>,
    action: F,
) -> Result<ModActionResponse>
where
    F: FnOnce(&mut Profile, &Uuid, &Thunderstore) -> anyhow::Result<ModActionResponse>,
{
    let mut manager = manager.lock().unwrap();
    let thunderstore = thunderstore.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    let response = action(manager.active_profile_mut(), &uuid, &thunderstore)?;

    if let ModActionResponse::Done = response {
        save(&manager, &prefs)?;
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
    for package_uuid in &uuids {
        profile.force_remove_mod(package_uuid)?;
    }

    save(&manager, &prefs)?;

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
        .map(|profile_mod| *profile_mod.uuid())
        .collect_vec();

    for uuid in uuids {
        profile.force_toggle_mod(&uuid)?;
    }

    save(&manager, &prefs)?;

    Ok(())
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
    for package_uuid in &uuids {
        profile.force_toggle_mod(package_uuid)?;
    }

    save(&manager, &prefs)?;

    Ok(())
}

#[tauri::command]
pub fn get_dependants(
    uuid: Uuid,
    manager: StateMutex<ModManager>,
    thunderstore: StateMutex<Thunderstore>,
) -> Result<Vec<String>> {
    let manager = manager.lock().unwrap();
    let thunderstore = thunderstore.lock().unwrap();

    let profile = manager.active_profile();
    let target = profile
        .get_mod(&uuid)?
        .as_remote()
        .ok_or_else(|| anyhow!("cannot find dependants of local mod"))?
        .0
        .borrow(&thunderstore)?;

    let dependants = manager
        .active_profile()
        .dependants(target, &thunderstore)
        .map(|borrowed| borrowed.version.full_name.to_owned())
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
pub fn open_plugin_dir(uuid: Uuid, manager: StateMutex<ModManager>) -> Result<()> {
    let manager = manager.lock().unwrap();

    let profile = manager.active_profile();

    profile.scan_mod(&profile.get_mod(&uuid)?.kind, |dir| {
        if dir.exists() {
            open::that(dir).context("failed to open directory")?;
        }

        Ok(())
    })?;

    Ok(())
}

#[tauri::command]
pub fn open_bepinex_log(manager: StateMutex<ModManager>) -> Result<()> {
    let manager = manager.lock().unwrap();

    let path = manager.active_profile().bepinex_log_path()?;
    open::that(path).context("failed to open log file")?;

    Ok(())
}

pub fn save(manager: &ModManager, prefs: &Prefs) -> Result<()> {
    manager
        .save(prefs)
        .context("failed to save manager state")?;

    Ok(())
}
