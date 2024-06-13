use super::modpack::{self, ModpackArgs};
use crate::{
    manager::{commands::save, ModManager},
    prefs::Prefs,
    thunderstore::{self, Thunderstore},
    util::cmd::{Result, StateMutex},
    NetworkClient,
};
use std::{fs, path::PathBuf};
use tauri::State;
use uuid::Uuid;
use anyhow::{anyhow, Context};

#[tauri::command]
pub async fn export_code(
    client: State<'_, NetworkClient>,
    manager: StateMutex<'_, ModManager>,
    thunderstore: StateMutex<'_, Thunderstore>,
    prefs: StateMutex<'_, Prefs>,
) -> Result<Uuid> {
    let key = super::export_code(&client.0, manager, thunderstore, prefs).await?;

    Ok(key)
}

#[tauri::command]
pub fn export_file(
    mut dir: PathBuf,
    manager: StateMutex<'_, ModManager>,
    thunderstore: StateMutex<'_, Thunderstore>,
) -> Result<()> {
    let manager = manager.lock().unwrap();
    let thunderstore = thunderstore.lock().unwrap();

    super::export_file(manager.active_profile(), &mut dir, &thunderstore)?;
    open::that(dir.parent().unwrap()).ok();

    Ok(())
}

#[tauri::command]
pub fn get_pack_args(manager: StateMutex<'_, ModManager>) -> Result<Option<ModpackArgs>> {
    let mut manager = manager.lock().unwrap();
    let profile = manager.active_profile_mut();

    modpack::refresh_args(profile);

    Ok(profile.modpack.clone())
}

#[tauri::command]
pub fn set_pack_args(
    args: ModpackArgs,
    manager: StateMutex<'_, ModManager>,
    prefs: StateMutex<'_, Prefs>,
) -> Result<()> {
    let mut manager = manager.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    manager.active_profile_mut().modpack = Some(args);

    save(&manager, &prefs)?;

    Ok(())
}

#[tauri::command]
pub fn export_pack(
    dir: PathBuf,
    args: ModpackArgs,
    manager: StateMutex<'_, ModManager>,
    thunderstore: StateMutex<'_, Thunderstore>,
) -> Result<()> {
    let mut manager = manager.lock().unwrap();
    let thunderstore = thunderstore.lock().unwrap();

    let profile = manager.active_profile_mut();

    let path = dir.join(&args.name).with_extension("zip");
    modpack::export(profile, &path, &args, &thunderstore)?;

    open::that(&path).ok();

    Ok(())
}

#[tauri::command]
pub async fn upload_pack(
    args: ModpackArgs,
    manager: StateMutex<'_, ModManager>,
    thunderstore: StateMutex<'_, Thunderstore>,
    prefs: StateMutex<'_, Prefs>,
    client: tauri::State<'_, NetworkClient>,
) -> Result<()> {
    let (path, game_id, args, token) = {
        let mut manager = manager.lock().unwrap();
        let thunderstore = thunderstore.lock().unwrap();
        let prefs = prefs.lock().unwrap();

        let token = thunderstore::token::get()
            .context("failed to get thunderstore API token")?
            .ok_or(anyhow!("no thunderstore API token found"))?;

        let profile = manager.active_profile_mut();

        let path = prefs
            .get_path_or_err("temp_dir")?
            .join(&args.name)
            .with_extension("zip");

        if path.exists() {
            fs::remove_file(&path).ok();
        }

        modpack::export(profile, &path, &args, &thunderstore)?;

        (path, &manager.active_game.id, args, token)
    };

    let client = client.0.clone();
    modpack::publish(path, game_id, args, token, client).await?;

    Ok(())
}

#[tauri::command]
pub fn export_dep_string(
    manager: StateMutex<ModManager>,
    thunderstore: StateMutex<Thunderstore>,
) -> Result<String> {
    let manager = manager.lock().unwrap();
    let thunderstore = thunderstore.lock().unwrap();

    manager
        .active_profile()
        .remote_mods()
        .map(|(mod_ref, _)| {
            let borrowed = mod_ref.borrow(&thunderstore)?;
            Ok(borrowed.version.full_name.clone())
        })
        .collect::<Result<Vec<_>>>()
        .map(|deps| deps.join("\n"))
}
