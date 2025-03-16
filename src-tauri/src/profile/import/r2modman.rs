use std::{
    fs::{self},
    path::PathBuf,
};

use eyre::{bail, Context, Result};
use log::{info, warn};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use super::ImportData;
use crate::{
    logger,
    profile::{
        export::{ImportSource, R2Mod},
        install::InstallOptions,
    },
    state::ManagerExt,
    thunderstore::{self},
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

            let mut manager = app.lock_manager();

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

fn find_profiles(mut path: PathBuf, app: &AppHandle) -> Result<impl Iterator<Item = PathBuf>> {
    let manager = app.lock_manager();

    let game = &manager.active_game;

    path.push(&*game.r2_dir_name);
    path.push("profiles");

    if !path.exists() {
        bail!(
            "directory was either not a r2modman data folder, or no profiles for {} exist",
            game.name
        );
    }

    Ok(path
        .read_dir()
        .fs_context("reading profiles directory", &path)?
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
    let mut manager = app.lock_manager();
    let thunderstore = app.lock_thunderstore();

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
