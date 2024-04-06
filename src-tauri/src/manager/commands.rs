use anyhow::Context;
use serde::Serialize;
use typeshare::typeshare;
use uuid::Uuid;

use crate::{
    prefs::Prefs,
    thunderstore::{query::QueryModsArgs, OwnedMod, Thunderstore},
    command_util::{Result, StateMutex},
};

use super::{ModManager, RemoveModResponse};

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

    ProfileInfo {
        names: manager.profiles.iter().map(|p| p.name.clone()).collect(),
        active_index: manager.active_profile_index,
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

    manager.set_active_profile(index)?;
    save(&manager, &prefs)?;
    Ok(())
}

#[tauri::command]
pub async fn query_mods_in_profile<'r>(
    args: QueryModsArgs,
    manager: StateMutex<'_, ModManager>,
    thunderstore: StateMutex<'_, Thunderstore>,
) -> Result<Vec<OwnedMod>> {
    let manager = manager.lock().unwrap();
    let thunderstore = thunderstore.lock().unwrap();

    let result = manager
        .active_profile()
        .query_mods(&args, &thunderstore)?;

    Ok(result)
}

#[tauri::command]
pub fn is_mod_installed(package_uuid: Uuid, manager: StateMutex<ModManager>) -> Result<bool> {
    let manager = manager.lock().unwrap();

    let result = manager
        .active_profile()
        .mods
        .iter()
        .any(|m| m.package_uuid == package_uuid);

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

    manager.create_profile(name, &prefs)?;
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

    manager.delete_profile(index)?;
    save(&manager, &prefs)?;

    Ok(())
}

#[tauri::command]
pub fn remove_mod(
    package_uuid: Uuid,
    manager: StateMutex<ModManager>,
    thunderstore: StateMutex<Thunderstore>,
    prefs: StateMutex<Prefs>,
) -> Result<RemoveModResponse> {
    let mut manager = manager.lock().unwrap();
    let thunderstore = thunderstore.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    let response = manager
        .active_profile_mut()
        .remove_mod(&package_uuid, &thunderstore)?;

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
pub fn start_game(
    manager: StateMutex<ModManager>,
    prefs: StateMutex<Prefs>,
) -> Result<()> {
    let manager = manager.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    manager.active_profile().run_game(&prefs)?;
    Ok(())
}

pub fn save(manager: &ModManager, prefs: &Prefs) -> Result<()> {
    manager.save(prefs).context("failed to save manager state")?;

    Ok(())
}
