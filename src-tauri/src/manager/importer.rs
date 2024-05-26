use std::{
    fs,
    io::{Cursor, Read, Seek},
    path::PathBuf,
    sync::Mutex,
};

use anyhow::{anyhow, ensure, Context, Result};
use tauri::{AppHandle, Manager};

use crate::{
    fs_util,
    manager::{commands::save, LocalMod, ProfileMod},
    prefs::Prefs,
    thunderstore::{models::PackageManifest, ModRef, Thunderstore},
    util::IoResultExt,
    NetworkClient,
};

use super::{
    downloader,
    exporter::{ExportManifest, PROFILE_DATA_PREFIX},
    ModManager,
};
use base64::{prelude::BASE64_STANDARD, Engine};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use itertools::Itertools;

pub mod commands;

pub fn setup(_app: &AppHandle) -> Result<()> {
    Ok(())
}

fn import_file_from_path(path: PathBuf, app: &AppHandle) -> Result<ImportData> {
    ensure!(path.exists(), "file does not exist");
    ensure!(path.is_file(), "path is not a file");

    let file = fs::File::open(&path).fs_context("opening file", &path)?;
    import_file(file, app)
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportData {
    pub name: String,
    pub temp_path: PathBuf,
    pub mods: Vec<ProfileMod>,
}

fn import_file<S: Read + Seek>(source: S, app: &AppHandle) -> Result<ImportData> {
    let thunderstore = app.state::<Mutex<Thunderstore>>();
    let prefs = app.state::<Mutex<Prefs>>();

    let thunderstore = thunderstore.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    // extract archive to temp path
    let temp_path = prefs.get_path_or_err("temp_dir")?.join("imports");
    if temp_path.exists() {
        fs::remove_dir_all(&temp_path)?;
    }
    fs::create_dir_all(&temp_path)?;

    zip_extract::extract(source, &temp_path, true)?;

    let manifest = fs::read_to_string(temp_path.join("export.r2x"))
        .context("failed to read profile manifest")?;

    // manifest is in r2modman's yaml format
    let manifest: ExportManifest =
        serde_yaml::from_str(&manifest).context("failed to parse profile manifest")?;

    let mods = manifest
        .mods
        .into_iter()
        .map(|export_mod| export_mod.into_profile_mod(&thunderstore))
        .collect::<Result<Vec<_>>>()
        .context("failed to resolve mod references")?;

    Ok(ImportData {
        mods,
        temp_path,
        name: manifest.profile_name.to_owned(),
    })
}

async fn import_data(data: ImportData, app: &AppHandle) -> Result<()> {
    {
        let manager = app.state::<Mutex<ModManager>>();
        let mut manager = manager.lock().unwrap();

        let game = manager.active_game_mut();
        if let Some(index) = game.profiles.iter().position(|p| p.name == data.name) {
            game.delete_profile(index, true).context("failed to delete existing profile")?;
        }

        let profile = game.create_profile(data.name)?;

        let mut config_dir = profile.path.clone();
        config_dir.push("BepInEx");
        config_dir.push("config");
        fs::create_dir_all(&config_dir)?;

        fs_util::copy_contents(&data.temp_path.join("config"), &config_dir, true)
            .context("error while importing config")?;
    };

    let mod_refs = data
        .mods
        .into_iter()
        .filter_map(|profile_mod| profile_mod.into_remote())
        .collect_vec();

    downloader::install_mod_refs(&mod_refs, app)
        .await
        .context("error while importing mods")?;

    Ok(())
}

async fn import_code(key: Uuid, app: &AppHandle) -> Result<ImportData> {
    let client = app.state::<NetworkClient>();
    let client = &client.0;

    let response = client
        .get(&format!(
            "https://thunderstore.io/api/experimental/legacyprofile/get/{key}/"
        ))
        .send()
        .await?
        .error_for_status()
        .map_err(|err| match err.status() {
            Some(status) if status == StatusCode::NOT_FOUND => {
                anyhow!("profile code is expired or invalid")
            }
            _ => err.into(),
        })?
        .text()
        .await?;

    match response.strip_prefix(PROFILE_DATA_PREFIX) {
        Some(data) => {
            let bytes = BASE64_STANDARD
                .decode(data)
                .context("failed to decode base64 data")?;

            import_file(Cursor::new(bytes), app)
        }
        None => Err(anyhow!("invalid profile data")),
    }
}

async fn import_local_mod(mut path: PathBuf, app: &AppHandle) -> Result<()> {
    ensure!(path.is_dir(), "mod path is not a directory");

    let manifest = read_manifest(&mut path)?;

    let uuid = Uuid::new_v4();

    let mut local_mod = match manifest {
        Some(manifest) => LocalMod {
            uuid,
            name: manifest.name,
            author: manifest.author,
            description: Some(manifest.description),
            version: Some(manifest.version_number),
            dependencies: Some(manifest.dependencies),
            ..Default::default()
        },
        None => LocalMod {
            uuid,
            name: fs_util::file_name(&path),
            ..Default::default()
        },
    };

    if let Some(ref deps) = local_mod.dependencies {
        downloader::install_mods(
            |manager, thunderstore| {
                let profile = manager.active_profile();

                Ok(thunderstore
                    .resolve_deps(deps.iter())
                    .0
                    .into_iter()
                    .filter(|dep| !profile.has_mod(&dep.package.uuid4))
                    .map(|borrowed_mod| (ModRef::from(borrowed_mod), true))
                    .collect::<Vec<_>>())
            },
            app,
        )
        .await?;
    }

    let manager = app.state::<Mutex<ModManager>>();
    let thunderstore = app.state::<Mutex<Thunderstore>>();
    let prefs = app.state::<Mutex<Prefs>>();

    let mut manager = manager.lock().unwrap();
    let thunderstore = thunderstore.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    let profile = manager.active_profile_mut();

    if profile.local_mods().any(|(m, _)| m.name == local_mod.name) {
        profile
            .force_remove_mod(&uuid, &thunderstore)
            .context("failed to remove existing mod")?;
    }

    downloader::install_from_disk(&path, &profile.path, &local_mod.name)
        .context("failed to install local mod")?;

    let mut mod_path = profile.path.clone();
    mod_path.push("BepInEx");
    mod_path.push("plugins");
    mod_path.push(&local_mod.name);

    downloader::normalize_mod_structure(&mut mod_path)?;

    mod_path.push("icon.png");
    if mod_path.exists() {
        local_mod.icon = Some(mod_path);
    }

    profile.mods.push(ProfileMod::local_now(local_mod));

    save(&manager, &prefs)?;

    Ok(())
}

fn read_manifest(path: &mut PathBuf) -> Result<Option<PackageManifest>> {
    path.push("manifest.json");

    let manifest = match path.exists() {
        true => {
            let json = fs::read_to_string(&*path).fs_context("reading manifest", &*path)?;

            let manifest: PackageManifest =
                serde_json::from_str(&json).context("failed to parse manifest")?;

            Some(manifest)
        }
        false => None,
    };
    path.pop();

    Ok(manifest)
}
