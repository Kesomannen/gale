use std::{
    fs::{self},
    path::PathBuf,
    sync::Mutex,
};

use eyre::{bail, Context, Result};
use log::{info, warn};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use super::ImportData;
use crate::{
    logger,
    profile::{
        export::{ImportSource, R2Mod},
        install::InstallOptions,
        ModManager,
    },
    thunderstore::{self, Thunderstore},
    util::{self, error::IoResultExt, fs::PathExt},
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileImportData {
    path: PathBuf,
    profiles: Vec<String>,
}

pub(super) fn gather_info(
    path: Option<PathBuf>,
    app: &AppHandle,
) -> Result<Option<ProfileImportData>> {
    let Some(path) = path.or_else(find_path) else {
        return Ok(None);
    };

    let profiles = find_profiles(path.clone(), app)?
        .map(util::fs::file_name_owned)
        .collect();

    Ok(Some(ProfileImportData { path, profiles }))
}

pub(super) async fn import(path: PathBuf, include: &[bool], app: &AppHandle) -> Result<()> {
    emit_update("Fetching mods from Thunderstore...", app);

    thunderstore::wait_for_fetch(app).await;

    info!("importing r2modman profiles from {}", path.display());

    for (i, profile_dir) in find_profiles(path, app)?.enumerate() {
        if !include[i] {
            continue;
        }

        let name = profile_dir.file_name().unwrap().to_string_lossy();

        let data = match prepare_import(profile_dir.clone(), app) {
            Ok(Some(data)) => data,
            Ok(None) => {
                continue;
            }
            Err(err) => {
                logger::log_webview_err(
                    "Error while importing from r2modman",
                    err.wrap_err(format!("Failed to prepare import of profile '{}'", name)),
                    app,
                );
                continue;
            }
        };

        if let Err(err) = import_profile(data, app).await {
            logger::log_webview_err(
                "Error while importing from r2modman",
                err.wrap_err(format!("Failed to import profile '{}'", name)),
                app,
            );

            let manager = app.state::<Mutex<ModManager>>();
            let mut manager = manager.lock().unwrap();

            let game = manager.active_game_mut();

            if let Some(index) = game.profile_index(&name) {
                game.delete_profile(index, true).unwrap_or_else(|_| {
                    warn!("failed to delete possibly corrupted profile '{}'", name)
                });
            }
        };
    }

    Ok(())
}

fn find_dir(path: &PathBuf, dir: &str) -> Result<Option<PathBuf>> {
    // Search for the game directory in a case-insensitive fashion
    let dir = dir.to_lowercase();

    Ok(path
        .read_dir()
        .fs_context("searching for r2modman game directory", &path)?
        .filter_map(Result::ok)
        .find(|entry| {
            entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
                && entry
                    .file_name()
                    .to_str()
                    .map(|name| name.to_lowercase() == dir)
                    .unwrap_or(false)
        })
        .map(|entry| entry.path()))
}

fn find_profiles(path: PathBuf, app: &AppHandle) -> Result<impl Iterator<Item = PathBuf>> {
    let manager = app.state::<Mutex<ModManager>>();
    let manager = manager.lock().unwrap();

    let game = &manager.active_game;

    // The r2_dir_name we have may not match the case of the directory
    // on the filesystem. This matters for case sensitive filesystems
    let Some(game_dir) = find_dir(&path, &game.r2_dir_name)? else {
        bail!(
            "no r2modman game directory found for game {} at {}",
            game.name,
            path.display()
        );
    };

    let profiles_path = game_dir.join("profiles");
    if !profiles_path.exists() {
        bail!(
            "no profiles found for game {} at {}",
            game.name,
            game_dir.display()
        );
    }

    Ok(profiles_path
        .read_dir()
        .fs_context("reading profiles directory", &profiles_path)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().unwrap().is_dir())
        .map(|entry| entry.path()))
}

async fn import_profile(data: ImportData, app: &AppHandle) -> Result<()> {
    info!("importing profile '{}'", data.name);
    emit_update(&format!("Importing profile '{}'... 0%", data.name), app);

    let name = data.name.clone();

    super::import_data(
        data,
        InstallOptions::default()
            .can_cancel(false)
            .send_progress(false)
            .on_progress(Box::new(move |progress, app| {
                let percentage = (progress.total_progress * 100.0).round();
                emit_update(
                    &format!("Importing profile '{}'... {}%", name, percentage),
                    app,
                );
            })),
        false,
        app,
    )
    .await
}

fn prepare_import(mut profile_dir: PathBuf, app: &AppHandle) -> Result<Option<ImportData>> {
    let manager = app.state::<Mutex<ModManager>>();
    let thunderstore = app.state::<Mutex<Thunderstore>>();

    let mut manager = manager.lock().unwrap();
    let thunderstore = thunderstore.lock().unwrap();

    let name = util::fs::file_name_owned(&profile_dir);

    profile_dir.push("mods.yml");

    if !profile_dir.exists() {
        info!("no mods.yml in '{}', skipping", name);
        return Ok(None);
    }
    let yaml = fs::read_to_string(&profile_dir).fs_context("reading mods.yml", &profile_dir)?;
    let mods = serde_yaml::from_str::<Vec<R2Mod>>(&yaml).context("failed to parse mods.yml")?;

    profile_dir.pop();

    if let Some(index) = manager.active_game().profile_index(&name) {
        info!("deleting existing profile '{}'", name);

        manager
            .active_game_mut()
            .delete_profile(index, true)
            .context("failed to delete existing profile")?;
    }

    ImportData::create_r2(
        name,
        mods,
        Vec::new(),
        profile_dir,
        false,
        ImportSource::R2,
        &thunderstore,
    )
    .map(Some)
}

fn find_path() -> Option<PathBuf> {
    let parent_dir = match cfg!(target_os = "linux") {
        // r2modman uses the config dir instead of the data dir on linux.
        true => dirs_next::config_dir(),
        false => dirs_next::data_dir(),
    }
    .unwrap();

    parent_dir
        .join("r2modmanPlus-local")
        .exists_or_none()
        .or_else(|| {
            parent_dir
                .join("Thunderstore Mod Manager")
                .join("DataFolder")
                .exists_or_none()
        })
}

fn emit_update(message: &str, app: &AppHandle) {
    app.emit("transfer_update", message).ok();
}
