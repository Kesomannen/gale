use super::modpack::{self, changelog, ModpackArgs};
use crate::{
    manager::{commands::save, ModManager},
    prefs::Prefs,
    thunderstore::{self, Thunderstore},
    util::{
        cmd::{Result, StateMutex},
        fs::PathExt,
    },
    NetworkClient,
};
use anyhow::{anyhow, Context};
use itertools::Itertools;
use log::warn;
use std::{fs, path::PathBuf};
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn export_code(
    client: State<'_, NetworkClient>,
    manager: StateMutex<'_, ModManager>,
    thunderstore: StateMutex<'_, Thunderstore>,
) -> Result<Uuid> {
    let key = super::export_code(&client.0, manager, thunderstore).await?;

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

    let profile = manager.active_profile();

    let mut path = dir;
    path.push(&profile.name);
    path.set_extension("r2z");

    let file = fs::File::create(&path).map_err(|err| anyhow!(err))?;
    super::export_zip(manager.active_profile(), file, &thunderstore)?;

    open::that(path).ok();

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

    let mut path = dir;
    path.push(format!("{}-{}", args.name, args.version_number));
    path.add_extension("zip");

    let file = fs::File::create(&path).map_err(|err| anyhow!(err))?;
    profile.export_pack(&args, file, &thunderstore)?;
    if let Err(err) = profile.take_snapshot(&args) {
        warn!("failed to take profile snapshot: {}", err);
    }

    open::that(path).ok();

    Ok(())
}

#[tauri::command]
pub async fn upload_pack(
    args: ModpackArgs,
    manager: StateMutex<'_, ModManager>,
    thunderstore: StateMutex<'_, Thunderstore>,
    client: tauri::State<'_, NetworkClient>,
) -> Result<()> {
    let mut temp_file = tempfile::Builder::new()
        .tempfile()
        .context("failed to create temporary directory")?;

    let (game_id, args, token) = {
        let manager = manager.lock().unwrap();
        let thunderstore = thunderstore.lock().unwrap();

        let token = thunderstore::token::get()
            .context("failed to get thunderstore API token")?
            .ok_or(anyhow!("no thunderstore API token found"))?;

        let profile = manager.active_profile();

        profile.export_pack(&args, &mut temp_file, &thunderstore)?;
        if let Err(err) = profile.take_snapshot(&args) {
            warn!("failed to take profile snapshot: {}", err);
        }

        (&manager.active_game.id, args, token)
    };

    let client = client.0.clone();
    modpack::publish(temp_file.path().to_path_buf(), game_id, args, token, client).await?;

    Ok(())
}

#[tauri::command]
pub fn export_dep_string(manager: StateMutex<ModManager>) -> Result<String> {
    let manager = manager.lock().unwrap();

    let result = manager
        .active_profile()
        .remote_mods()
        .map(|(_, full_name, _)| full_name)
        .join("\n");

    Ok(result)
}

#[tauri::command]
pub fn generate_changelog(
    mut args: ModpackArgs,
    all: bool,
    manager: StateMutex<ModManager>,
    thunderstore: StateMutex<Thunderstore>,
) -> Result<String> {
    let manager = manager.lock().unwrap();
    let thunderstore = thunderstore.lock().unwrap();

    if all {
        let changelog = changelog::generate_all(
            &args,
            manager.active_profile(),
            manager.active_game().game,
            &thunderstore,
        )?;

        Ok(changelog)
    } else {
        changelog::generate_latest(
            &mut args,
            manager.active_profile(),
            manager.active_game().game,
            &thunderstore,
        )?;

        Ok(args.changelog)
    }
}
