use anyhow::Context;
use serde::Serialize;
use typeshare::typeshare;
use uuid::Uuid;

use crate::{
    command_util::{Result, StateMutex},
    games::{self, Game, GAMES},
    prefs::Prefs,
    thunderstore::{models::FrontendProfileMod, query::QueryModsArgs, Thunderstore},
};

use super::{ModActionResponse, ModManager, Profile};
use itertools::Itertools;

#[typeshare]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameInfo {
    all: &'static [Game],
    active: &'static Game,
    favorites: Vec<&'static str>,
}

#[tauri::command]
pub fn get_game_info(manager: StateMutex<ModManager>) -> GameInfo {
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

    let game = games::from_name(&id).context("invalid game id")?;
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

    let game = games::from_name(id).context("invalid game id")?;

    manager.ensure_game(game, &prefs)?;

    if manager.active_game.id != game.id {
        manager.active_game = game;
        thunderstore.switch_game(game, app);

        save(&manager, &prefs)?;
    }

    Ok(())
}

#[typeshare]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileInfo {
    names: Vec<String>,
    active_index: usize,
}

#[tauri::command]
pub fn get_profile_info(manager: StateMutex<ModManager>) -> ProfileInfo {
    let manager = manager.lock().unwrap();
    let game = manager.active_game();

    ProfileInfo {
        names: game.profiles.iter().map(|p| p.name.clone()).collect(),
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
    save(&manager, &prefs)?;
    Ok(())
}

#[typeshare]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontendAvailableUpdate {
    pub name: String,
    pub uuid: Uuid,
    pub old: semver::Version,
    pub new: semver::Version,
}

#[typeshare]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileQuery {
    pub updates: Vec<FrontendAvailableUpdate>,
    pub mods: Vec<FrontendProfileMod>,
}

#[tauri::command]
pub fn query_mods_in_profile(
    args: QueryModsArgs,
    manager: StateMutex<'_, ModManager>,
    thunderstore: StateMutex<'_, Thunderstore>,
) -> Result<ProfileQuery> {
    let manager = manager.lock().unwrap();
    let thunderstore = thunderstore.lock().unwrap();

    let profile = manager.active_profile();
    let mods = profile.query_mods(&args, &thunderstore)?;
    let updates = profile
        .available_updates(&thunderstore)
        .map_ok(|update| {
            let borrow = update.mod_ref.borrow(&thunderstore)?;
            Ok::<_, anyhow::Error>(FrontendAvailableUpdate {
                name: borrow.package.name.clone(),
                uuid: borrow.package.uuid4,
                old: update.current.clone(),
                new: update.latest.version.version_number.clone(),
            })
        })
        .flatten_ok()
        .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(ProfileQuery { updates, mods })
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

    manager.active_game_mut().delete_profile(index)?;
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
    thunderstore: StateMutex<Thunderstore>,
    prefs: StateMutex<Prefs>,
) -> Result<()> {
    let mut manager = manager.lock().unwrap();
    let thunderstore = thunderstore.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    let profile = manager.active_profile_mut();
    for package_uuid in &uuids {
        profile.force_remove_mod(package_uuid, &thunderstore)?;
    }

    save(&manager, &prefs)?;

    Ok(())
}

#[tauri::command]
pub fn force_toggle_mods(
    uuids: Vec<Uuid>,
    manager: StateMutex<ModManager>,
    thunderstore: StateMutex<Thunderstore>,
    prefs: StateMutex<Prefs>,
) -> Result<()> {
    let mut manager = manager.lock().unwrap();
    let thunderstore = thunderstore.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    let profile = manager.active_profile_mut();
    for package_uuid in &uuids {
        profile.force_toggle_mod(package_uuid, &thunderstore)?;
    }

    save(&manager, &prefs)?;

    Ok(())
}

#[tauri::command]
pub fn reveal_profile_dir(manager: StateMutex<ModManager>) -> Result<()> {
    let manager = manager.lock().unwrap();

    let path = &manager.active_profile().path;
    open::that(path).with_context(|| format!("failed to open dir {}", path.display()))?;

    Ok(())
}

pub fn save(manager: &ModManager, prefs: &Prefs) -> Result<()> {
    manager
        .save(prefs)
        .context("failed to save manager state")?;

    Ok(())
}
