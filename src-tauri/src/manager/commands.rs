use anyhow::Context;
use serde::Serialize;
use typeshare::typeshare;
use uuid::Uuid;

use crate::{
    command_util::{Result, StateMutex}, games::{self, Game, GAMES}, prefs::Prefs, thunderstore::{
        models::FrontendMod, query::QueryModsArgs, Thunderstore
    }
};

use super::{ModManager, RemoveModResponse};

#[typeshare]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameInfo {
    all: Vec<Game>,
    active: Game,
}

#[tauri::command]
pub fn get_game_info(manager: StateMutex<ModManager>) -> GameInfo {
    let manager = manager.lock().unwrap();

    GameInfo {
        all: GAMES.iter().cloned().collect(),
        active: games::from_steam_id(manager.active_game).unwrap().clone()
    }
}

#[tauri::command]
pub fn set_active_game(
    steam_id: u32,
    app: tauri::AppHandle,
    manager: StateMutex<ModManager>,
    thunderstore: StateMutex<Thunderstore>,
    prefs: StateMutex<Prefs>,
) -> Result<()> {
    let mut manager = manager.lock().unwrap();
    let mut thunderstore = thunderstore.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    manager.ensure_game(steam_id, &prefs)?;
    manager.active_game = steam_id;
    thunderstore.switch_game(steam_id, app);

    save(&manager, &prefs)?;

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

#[tauri::command]
pub fn query_mods_in_profile(
    args: QueryModsArgs,
    manager: StateMutex<'_, ModManager>,
    thunderstore: StateMutex<'_, Thunderstore>,
) -> Result<Vec<FrontendMod>> {
    let manager = manager.lock().unwrap();
    let thunderstore = thunderstore.lock().unwrap();

    let result = manager.active_profile().query_mods(&args, &thunderstore)?;

    Ok(result)
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
) -> Result<RemoveModResponse> {
    let mut manager = manager.lock().unwrap();
    let thunderstore = thunderstore.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    let response = manager
        .active_profile_mut()
        .remove_mod(&uuid, &thunderstore)?;

    if let RemoveModResponse::Removed = response {
        save(&manager, &prefs)?;
    }

    Ok(response)
}

#[tauri::command]
pub fn force_remove_mods(
    package_uuids: Vec<Uuid>,
    manager: StateMutex<ModManager>,
    thunderstore: StateMutex<Thunderstore>,
    prefs: StateMutex<Prefs>,
) -> Result<()> {
    let mut manager = manager.lock().unwrap();
    let thunderstore = thunderstore.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    let profile = manager.active_profile_mut();
    for package_uuid in &package_uuids {
        profile.force_remove_mod(package_uuid, &thunderstore)?;
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

#[tauri::command]
pub fn start_game(manager: StateMutex<ModManager>, prefs: StateMutex<Prefs>) -> Result<()> {
    let manager = manager.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    manager.run_game(&prefs)?;
    Ok(())
}

pub fn save(manager: &ModManager, prefs: &Prefs) -> Result<()> {
    manager
        .save(prefs)
        .context("failed to save manager state")?;

    Ok(())
}
