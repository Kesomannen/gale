use std::{
    fs,
    io::{Cursor, Read, Seek},
    path::{Path, PathBuf},
    sync::Mutex,
};

use anyhow::{anyhow, ensure, Context, Result};
use tauri::{AppHandle, Manager};

use crate::{
    manager::{commands::save, downloader::InstallOptions, installer, LocalMod, ProfileMod},
    prefs::Prefs,
    thunderstore::{models::PackageManifest, Thunderstore},
    util::{self, error::IoResultExt},
    NetworkClient,
};

use super::{
    downloader::{self, ModInstall},
    exporter::{self, R2Manifest, R2Mod, PROFILE_DATA_PREFIX},
    ModManager,
};
use base64::{prelude::BASE64_STANDARD, Engine};
use log::debug;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod commands;
pub mod r2modman;

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
    pub mods: Vec<ModInstall>,
    pub path: PathBuf,
    pub includes: Vec<PathBuf>,
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

    util::zip::extract(source, &temp_path)?;

    let manifest = fs::read_to_string(temp_path.join("export.r2x"))
        .context("failed to read profile manifest")?;

    let manifest: R2Manifest =
        serde_yaml::from_str(&manifest).context("failed to parse profile manifest")?;

    let includes = exporter::find_includes(&temp_path).collect();

    Ok(ImportData {
        mods: resolve_r2mods(manifest.mods.into_iter(), &thunderstore)?,
        name: manifest.profile_name.to_owned(),
        path: temp_path,
        includes,
    })
}

fn resolve_r2mods<'a>(
    mods: impl Iterator<Item = R2Mod<'a>>,
    thunderstore: &Thunderstore,
) -> Result<Vec<ModInstall>> {
    mods.map(|r2| r2.into_install(thunderstore))
        .collect::<Result<Vec<_>>>()
        .context("failed to resolve mod references")
}

async fn import_data(data: ImportData, options: InstallOptions, app: &AppHandle) -> Result<()> {
    let path = {
        let manager = app.state::<Mutex<ModManager>>();
        let mut manager = manager.lock().unwrap();

        let game = manager.active_game_mut();
        if let Some(index) = game.profiles.iter().position(|p| p.name == data.name) {
            game.delete_profile(index, true)
                .context("failed to delete existing profile")?;
        }

        let profile = game.create_profile(data.name)?;

        profile.path.clone()
    };

    downloader::install_mods(data.mods, options, app)
        .await
        .context("error while importing mods")?;

    import_config(&path, &data.path, data.includes.into_iter())
        .context("failed to import config")?;

    Ok(())
}

fn import_config(target: &Path, source: &Path, files: impl Iterator<Item = PathBuf>) -> Result<()> {
    for file in files {
        let source = source.join(&file);

        let target = match file.starts_with("config") {
            true => target.join("BepInEx").join(file),
            false => target.join(file),
        };

        fs::create_dir_all(target.parent().unwrap())?;
        fs::copy(&source, &target)?;

        debug!("copied {} to {}", source.display(), target.display());
    }

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

    path.push("manifest.json");

    let json = match path.exists() {
        true => Some(fs::read_to_string(&*path).fs_context("reading manifest", &path)?),
        false => None,
    };

    let manifest = match &json {
        Some(json) => Some(
            serde_json::from_str::<PackageManifest>(json).context("failed to parse manifest")?,
        ),
        None => None,
    };

    path.pop();

    let uuid = Uuid::new_v4();

    let mut local_mod = match manifest {
        Some(manifest) => LocalMod {
            uuid,
            name: manifest.name.to_owned(),
            author: manifest.author.map(|a| a.to_owned()),
            description: Some(manifest.description.to_owned()),
            version: Some(manifest.version_number),
            dependencies: Some(
                manifest
                    .dependencies
                    .into_iter()
                    .map(|s| s.to_owned())
                    .collect(),
            ),
            ..Default::default()
        },
        None => LocalMod {
            uuid,
            name: util::fs::file_name_lossy(&path),
            ..Default::default()
        },
    };

    if let Some(ref deps) = local_mod.dependencies {
        downloader::install_with_mods(
            |manager, thunderstore| {
                let profile = manager.active_profile();

                Ok(thunderstore
                    .resolve_deps(deps.iter())
                    .0
                    .into_iter()
                    .filter(|dep| !profile.has_mod(&dep.package.uuid4))
                    .map(|borrowed| borrowed.into())
                    .collect::<Vec<_>>())
            },
            InstallOptions::default().can_cancel(false),
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

    installer::from_disk(&path, &profile.path, &local_mod.name)
        .context("failed to install local mod")?;

    let mut mod_path = profile.path.clone();
    mod_path.push("BepInEx");
    mod_path.push("plugins");
    mod_path.push(&local_mod.name);

    installer::normalize_mod_structure(&mut mod_path)?;

    mod_path.push("icon.png");
    if mod_path.exists() {
        local_mod.icon = Some(mod_path);
    }

    profile.mods.push(ProfileMod::local_now(local_mod));

    save(&manager, &prefs)?;

    Ok(())
}
