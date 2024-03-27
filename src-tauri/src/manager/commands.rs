use anyhow::Context;
use serde::Serialize;
use uuid::Uuid;

use crate::{
    prefs::{Prefs, PrefsState},
    thunderstore::{query::QueryModsArgs, BorrowedMod, OwnedMod, ThunderstoreState}, util,
};

use super::ModManager;

type Result<T> = util::CommandResult<T>;

#[derive(Serialize)]
pub struct ProfileInfo {
    names: Vec<String>,
    active_index: usize,
}

#[tauri::command]
pub fn get_profile_info(manager: tauri::State<ModManager>) -> ProfileInfo {
    let profiles = manager.profiles.lock().unwrap();
    let active_profile_index = *manager.active_profile_index.lock().unwrap();

    ProfileInfo {
        names: profiles.iter().map(|p| p.name.clone()).collect(),
        active_index: active_profile_index,
    }
}

#[tauri::command]
pub fn set_active_profile(
    index: usize,
    manager: tauri::State<'_, ModManager>,
    prefs: tauri::State<'_, PrefsState>,
) -> Result<()> {
    manager.set_active_profile(index)?;
    save(&manager, &prefs)
}

#[tauri::command]
pub async fn query_mods_in_profile(
    args: QueryModsArgs,
    manager: tauri::State<'_, ModManager>,
    thunderstore: tauri::State<'_, ThunderstoreState>,
) -> Result<Vec<OwnedMod>> {
    thunderstore.wait_for_load().await;

    let mod_map = thunderstore.packages.lock().unwrap();
    let mut profiles = manager.profiles.lock().unwrap();
    let profile = super::get_active_profile(&mut profiles, &manager)?;

    Ok(profile.query_mods(args, &mod_map)?)
}

#[tauri::command]
pub fn get_download_size(
    package_uuid: Uuid,
    manager: tauri::State<'_, ModManager>,
    prefs: tauri::State<'_, PrefsState>,
    thunderstore: tauri::State<'_, ThunderstoreState>,
) -> Result<u64> {
    let mut profiles = manager.profiles.lock().unwrap();
    let profile = super::get_active_profile(&mut profiles, &manager)?;

    let mod_map = thunderstore.packages.lock().unwrap();
    let package = mod_map.get(&package_uuid).context("package not found")?;
    let target_mod = BorrowedMod {
        package,
        version: &package.versions[0],
    };

    Ok(profile.total_download_size(&prefs.lock(),target_mod, &mod_map)?)
}

#[tauri::command]
pub fn create_profile(
    name: String,
    manager: tauri::State<'_, ModManager>,
    prefs: tauri::State<'_, PrefsState>,
) -> Result<()> {
    let prefs = prefs.lock();

    let index = manager.create_profile(name, &prefs)?;
    manager.set_active_profile(index)?;
    save_unlocked(&manager, &prefs)
}

#[tauri::command]
pub fn delete_profile(
    index: usize,
    manager: tauri::State<'_, ModManager>,
    prefs: tauri::State<'_, PrefsState>,
) -> Result<()> {
    manager.delete_profile(index)?;
    save(&manager, &prefs)
}

#[tauri::command]
pub fn remove_mod(
    index: usize,
    manager: tauri::State<ModManager>,
    prefs: tauri::State<PrefsState>,
) -> Result<()> {
    let mut profiles = manager.profiles.lock().unwrap();
    let profile = super::get_active_profile(&mut profiles, &manager)?;

    profile.mods.remove(index);
    save(&manager, &prefs)
}

#[tauri::command]
pub fn reveal_project_dir(manager: tauri::State<ModManager>) -> Result<()> {
    let mut profiles = manager.profiles.lock().unwrap();
    let profile = super::get_active_profile(&mut profiles, &manager)?;
    open::that(&profile.path).context("failed to open directory")?;
    Ok(())
}

#[tauri::command]
pub fn start_game(
    manager: tauri::State<ModManager>,
    prefs: tauri::State<PrefsState>,
) -> Result<()> {
    let mut profiles = manager.profiles.lock().unwrap();
    let profile = super::get_active_profile(&mut profiles, &manager)?;
    profile.run_game(&prefs.lock())?;
    Ok(())
}

fn save(manager: &ModManager, prefs: &PrefsState) -> Result<()> {
    save_unlocked(manager, &prefs.lock())
}

fn save_unlocked(manager: &ModManager, prefs: &Prefs) -> Result<()> {
    manager.save(prefs).context("failed to save manager state")?;
    Ok(())
}