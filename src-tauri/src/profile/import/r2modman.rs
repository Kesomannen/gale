use std::{
    fs::{self},
    path::PathBuf,
    sync::Mutex,
    time::Duration,
};

use anyhow::{ensure, Context, Result};
use log::{debug, info, warn};
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
    thunderstore::Thunderstore,
    util::{self, error::IoResultExt, fs::PathExt},
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagerData<T> {
    r2modman: Option<T>,
    thunderstore: Option<T>,
}

impl<T> ManagerData<T> {
    pub fn and_then<U, F: FnOnce(T) -> Option<U> + Copy>(self, f: F) -> ManagerData<U> {
        ManagerData {
            r2modman: self.r2modman.and_then(f),
            thunderstore: self.thunderstore.and_then(f),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileImportData {
    path: PathBuf,
    profiles: Vec<String>,
}

pub(super) fn gather_info(app: &AppHandle) -> ManagerData<ProfileImportData> {
    find_paths().and_then(|path| {
        let profiles = find_profiles(path.clone(), app)
            .ok()?
            .map(util::fs::file_name_owned)
            .collect();
        Some(ProfileImportData { path, profiles })
    })
}

pub(super) async fn import(path: PathBuf, include: &[bool], app: &AppHandle) -> Result<()> {
    wait_for_mods(app).await;

    info!("importing profiles from {}", path.display());

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
                logger::log_js_err(
                    "Error while importing from r2modman",
                    &err.context(format!("Failed to prepare import of profile '{}'", name)),
                    app,
                );
                continue;
            }
        };

        if let Err(err) = import_profile(data, app).await {
            logger::log_js_err(
                "Error while importing from r2modman",
                &err.context(format!("Failed to import profile '{}'", name)),
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

fn find_profiles(mut path: PathBuf, app: &AppHandle) -> Result<impl Iterator<Item = PathBuf>> {
    let manager = app.state::<Mutex<ModManager>>();
    let manager = manager.lock().unwrap();

    path.push(&*manager.active_game.r2_dir_name);
    path.push("profiles");

    debug!("scanning {path:?}");

    ensure!(path.exists(), "no profiles found");

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
            .on_progress(move |progress, app| {
                let percentage = (progress.total_progress() * 100.0).round();
                emit_update(
                    &format!("Importing profile '{}'... {}%", name, percentage),
                    app,
                );
            }),
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

    if mods.is_empty() {
        info!("profile '{}' is empty, skipping", name);
        return Ok(None);
    }

    if let Some(index) = manager.active_game().profile_index(&name) {
        info!("deleting existing profile '{}'", name);

        manager
            .active_game_mut()
            .delete_profile(index, true)
            .context("failed to delete existing profile")?;
    }

    ImportData::from_r2_mods(
        name,
        mods,
        profile_dir,
        false,
        Vec::new(),
        ImportSource::R2,
        &thunderstore,
    )
    .map(Some)
}

async fn wait_for_mods(app: &AppHandle) {
    let thunderstore = app.state::<Mutex<Thunderstore>>();

    loop {
        {
            let thunderstore = thunderstore.lock().unwrap();
            if thunderstore.packages_fetched() {
                return;
            }
        }

        emit_update("Fetching mods from Thunderstore...", app);

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

fn find_paths() -> ManagerData<PathBuf> {
    let parent_dir = match cfg!(target_os = "linux") {
        // r2modman uses the config dir instead of the data dir on linux.
        true => dirs_next::config_dir(),
        false => dirs_next::data_dir(),
    }
    .unwrap();

    ManagerData {
        r2modman: parent_dir.join("r2modmanPlus-local").exists_or_none(),
        thunderstore: parent_dir
            .join("Thunderstore Mod Manager")
            .join("DataFolder")
            .exists_or_none(),
    }
}

fn emit_update(message: &str, app: &AppHandle) {
    app.emit("transfer_update", message).ok();
}
