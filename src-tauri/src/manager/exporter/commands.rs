use super::modpack::{self, changelog, ModpackArgs};
use crate::{
    manager::{commands::save, ModManager},
    prefs::Prefs,
    thunderstore::{self, Thunderstore},
    util::{
        cmd::{Result, StateMutex},
        error::IoResultExt,
    },
    NetworkClient,
};
use anyhow::{anyhow, Context};
use std::{fs, path::PathBuf};
use tauri::State;
use uuid::Uuid;

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
    dir: PathBuf,
    manager: StateMutex<'_, ModManager>,
    thunderstore: StateMutex<'_, Thunderstore>,
) -> Result<()> {
    let manager = manager.lock().unwrap();
    let thunderstore = thunderstore.lock().unwrap();

    let path = super::export_file(manager.active_profile(), dir, &thunderstore)?;
    open::that(path.parent().unwrap()).ok();

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
    profile.export_pack(&args, &path, &thunderstore)?;
    if let Err(err) = profile.take_snapshot(&args) {
        log::warn!("failed to take profile snapshot: {}", err);
    }

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
        let manager = manager.lock().unwrap();
        let thunderstore = thunderstore.lock().unwrap();
        let prefs = prefs.lock().unwrap();

        let token = thunderstore::token::get()
            .context("failed to get thunderstore API token")?
            .ok_or(anyhow!("no thunderstore API token found"))?;

        let profile = manager.active_profile();

        let mut path = prefs.temp_dir.to_path_buf();
        path.push("modpacks");

        if !path.exists() {
            fs::create_dir_all(&path).fs_context("creating temp dir", &path)?;
        }

        path.push(&args.name);
        path.set_extension("zip");

        if path.exists() {
            fs::remove_file(&path).ok();
        }

        profile.export_pack(&args, &path, &thunderstore)?;
        if let Err(err) = profile.take_snapshot(&args) {
            log::warn!("failed to take profile snapshot: {}", err);
        }

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

#[tauri::command]
pub fn generate_changelog(
    mut args: ModpackArgs,
    manager: StateMutex<ModManager>,
    thunderstore: StateMutex<Thunderstore>,
) -> Result<String> {
    let manager = manager.lock().unwrap();
    let thunderstore = thunderstore.lock().unwrap();

    changelog::generate(
        &mut args,
        manager.active_profile(),
        manager.active_game().game,
        &thunderstore,
    )?;

    Ok(args.changelog)
}
