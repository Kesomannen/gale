use std::{
    fs,
    io::{Cursor, Read, Seek},
    path::{Path, PathBuf},
    sync::Mutex,
};

use anyhow::{anyhow, bail, ensure, Context, Result};
use base64::{prelude::BASE64_STANDARD, Engine};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use uuid::Uuid;

use crate::{
    manager::{commands::save, downloader::InstallOptions, installer, LocalMod, ProfileMod},
    prefs::Prefs,
    thunderstore::{models::PackageManifest, Thunderstore},
    util::{self, error::IoResultExt, fs::PathExt},
    NetworkClient,
};

use super::{
    downloader::{self, ModInstall},
    exporter::{self, ImportSource, LegacyProfileManifest, R2Mod, PROFILE_DATA_PREFIX},
    ModManager,
};
use tempfile::tempdir;

pub mod commands;
pub mod r2modman;

pub fn setup(_app: &AppHandle) -> Result<()> {
    Ok(())
}

pub async fn import_file_from_link(url: String, app: &AppHandle) -> Result<()> {
    let data = import_file_from_path(url.into(), app)?;
    import_data(data, InstallOptions::default(), app).await?;
    Ok(())
}

fn import_file_from_path(path: PathBuf, app: &AppHandle) -> Result<ImportData> {
    ensure!(path.is_file(), "path is not a file");

    let file = fs::File::open(&path).fs_context("opening file", &path)?;

    import_file(file, app)
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportData {
    pub name: String,
    pub mod_names: Option<Vec<String>>,
    pub mods: Vec<ModInstall>,
    pub path: PathBuf,
    pub delete_after_import: bool,
    pub includes: Vec<PathBuf>,
    pub ignored_updates: Vec<Uuid>,
    pub source: ImportSource,
}

impl ImportData {
    pub fn from_r2_mods(
        name: String,
        mods: Vec<R2Mod<'_>>,
        path: PathBuf,
        delete_after_import: bool,
        ignored_updates: Vec<Uuid>,
        source: ImportSource,
        thunderstore: &Thunderstore,
    ) -> Result<Self> {
        let includes = exporter::find_includes(&path).collect();
        let mod_names = mods.iter().map(|r2| r2.full_name()).collect();
        let mods = mods
            .into_iter()
            .map(|r2| r2.into_install(thunderstore))
            .collect::<Result<Vec<_>>>()
            .context("failed to resolve mod references")?;

        Ok(Self {
            name,
            mods,
            path,
            delete_after_import,
            includes,
            mod_names: Some(mod_names),
            ignored_updates,
            source,
        })
    }
}

fn import_file<S: Read + Seek>(source: S, app: &AppHandle) -> Result<ImportData> {
    let thunderstore = app.state::<Mutex<Thunderstore>>();
    let thunderstore = thunderstore.lock().unwrap();

    let temp_dir = tempdir().context("failed to create temporary directory")?;
    util::zip::extract(source, temp_dir.path())?;

    let manifest = fs::read_to_string(temp_dir.path().join("export.r2x"))
        .context("failed to read profile manifest")?;

    let manifest: LegacyProfileManifest =
        serde_yaml::from_str(&manifest).context("failed to parse profile manifest")?;

    ImportData::from_r2_mods(
        manifest.profile_name.to_owned(),
        manifest.mods,
        temp_dir.into_path(),
        true,
        manifest.ignored_updates,
        manifest.source,
        &thunderstore,
    )
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

        profile.ignored_updates.extend(data.ignored_updates);

        profile.path.clone()
    };

    downloader::install_mods(data.mods, options, app)
        .await
        .context("error while importing mods")?;

    import_config(&path, &data.path, data.includes.into_iter())
        .context("failed to import config")?;

    if data.delete_after_import {
        fs::remove_dir_all(&data.path).ok();
    }

    Ok(())
}

fn import_config(target: &Path, source: &Path, files: impl Iterator<Item = PathBuf>) -> Result<()> {
    for file in files {
        let source = source.join(&file);

        let target = match file.starts_with("config") {
            true => target.join("BepInEx").join(file),
            false => target.join(file),
        };

        let parent = target.parent().unwrap();
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }

        fs::copy(&source, &target)?;
    }

    Ok(())
}

async fn import_code(key: Uuid, app: &AppHandle) -> Result<ImportData> {
    let client = app.state::<NetworkClient>();
    let client = &client.0;

    let response = client
        .get(format!(
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

async fn import_local_mod(path: PathBuf, app: &AppHandle) -> Result<()> {
    let (mut local_mod, kind) = read_local_mod(&path)?;

    if let Some(deps) = &local_mod.dependencies {
        downloader::install_with_mods(
            |manager, thunderstore| {
                let profile = manager.active_profile();

                Ok(thunderstore
                    .resolve_deps(deps.iter())
                    .0
                    .into_iter()
                    .filter(|dep| !profile.has_mod(dep.package.uuid4))
                    .map(|borrowed| borrowed.into())
                    .collect::<Vec<_>>())
            },
            InstallOptions::default().can_cancel(false),
            app,
        )
        .await?;
    }

    let manager = app.state::<Mutex<ModManager>>();
    let prefs = app.state::<Mutex<Prefs>>();

    let mut manager = manager.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    let profile = manager.active_profile_mut();

    let existing = profile
        .local_mods()
        .find(|(LocalMod { name, .. }, _)| *name == local_mod.name);

    let existing = existing.map(|(LocalMod { uuid, .. }, _)| *uuid);

    if let Some(uuid) = existing {
        profile
            .force_remove_mod(uuid)
            .context("failed to remove existing mod")?;
    }

    let mut plugin_dir = profile.path.clone();
    plugin_dir.push("BepInEx");
    plugin_dir.push("plugins");
    plugin_dir.push(&local_mod.name);

    match kind {
        LocalModKind::Zip => {
            installer::install_from_zip(&path, &profile.path, &local_mod.name, &prefs)
                .context("failed to install local mod")?;

            local_mod.icon = plugin_dir.join("icon.png").exists_or_none();
        }
        LocalModKind::Dll => {
            let file_name = path.file_name().unwrap();

            fs::create_dir_all(&plugin_dir)?;
            fs::copy(&path, plugin_dir.join(file_name))?;
        }
    }

    profile.mods.push(ProfileMod::local_now(local_mod));

    save(&manager, &prefs)?;

    Ok(())
}

#[derive(PartialEq, Eq)]
enum LocalModKind {
    Zip,
    Dll,
}

fn read_local_mod(path: &Path) -> Result<(LocalMod, LocalModKind)> {
    ensure!(path.is_file(), "path is not a file");

    let kind = match path.extension().and_then(|ext| ext.to_str()) {
        Some("dll") => LocalModKind::Dll,
        Some("zip") => LocalModKind::Zip,
        _ => bail!("unsupported file type"),
    };

    let manifest = match kind {
        LocalModKind::Zip => read_zip_manifest(path)?,
        LocalModKind::Dll => None,
    };

    let uuid = Uuid::new_v4();
    let file_size = path.metadata()?.len();

    let local_mod = match manifest {
        Some(manifest) => LocalMod {
            uuid,
            file_size,
            name: manifest.name,
            author: manifest.author,
            description: Some(manifest.description),
            version: Some(manifest.version_number),
            dependencies: Some(manifest.dependencies),
            ..Default::default()
        },
        None => LocalMod {
            uuid,
            file_size,
            name: util::fs::file_name_owned(path.with_extension("")),
            ..Default::default()
        },
    };

    return Ok((local_mod, kind));

    fn read_zip_manifest(path: &Path) -> Result<Option<PackageManifest>> {
        let mut zip = util::fs::open_zip(path).context("failed to open zip archive")?;

        let manifest = zip.by_name("manifest.json");

        match manifest {
            Ok(mut file) => {
                let mut str = String::with_capacity(file.size() as usize);
                file.read_to_string(&mut str)
                    .context("failed to read manifest")?;

                // remove BOM
                if str.starts_with("\u{feff}") {
                    str.replace_range(0..3, "");
                }

                serde_json::from_str(&str)
                    .context("failed to parse manifest")
                    .map(Some)
            }
            Err(_) => Ok(None),
        }
    }
}
