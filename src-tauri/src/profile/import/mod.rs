use std::{
    fs::{self, File},
    io::{BufReader, Cursor, Read, Seek},
    path::{Path, PathBuf},
    sync::Mutex,
};

use base64::{prelude::BASE64_STANDARD, Engine};
use eyre::{anyhow, Context, Result};
use itertools::Itertools;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tempfile::tempdir;
use uuid::Uuid;

use crate::{
    profile::{
        export::{self, ImportSource, LegacyProfileManifest, R2Mod, PROFILE_DATA_PREFIX},
        install::{self, InstallOptions, ModInstall},
        ModManager,
    },
    thunderstore::Thunderstore,
    util::{self, error::IoResultExt},
    NetworkClient,
};

pub mod commands;
mod local;
mod r2modman;

pub use local::import_local_mod;

use super::export::{IncludeExtensions, IncludeGenerated};

pub async fn import_file_from_deep_link(url: String, app: &AppHandle) -> Result<()> {
    let data = import_file_from_path(url.into(), app)?;
    import_data(data, InstallOptions::default(), false, app).await?;
    Ok(())
}

fn import_file_from_path(path: PathBuf, app: &AppHandle) -> Result<ImportData> {
    let file = File::open(&path).fs_context("opening file", &path)?;

    import_file(file, app)
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportData {
    name: String,
    mod_names: Vec<String>,
    mods: Vec<ModInstall>,
    path: PathBuf,
    delete_after_import: bool,
    ignored_updates: Vec<Uuid>,
    source: ImportSource,
}

impl ImportData {
    pub fn create_r2(
        name: String,
        mods: Vec<R2Mod>,
        ignored_updates: Vec<Uuid>,
        path: PathBuf,
        delete_after_import: bool,
        source: ImportSource,
        thunderstore: &Thunderstore,
    ) -> Result<Self> {
        let mod_names = mods.iter().map(|r2| r2.ident()).collect();
        let mods = mods
            .into_iter()
            .map(|r2| r2.into_install(thunderstore))
            .filter_map(Result::ok)
            .collect_vec();

        Ok(Self {
            name,
            mod_names,
            mods,
            path,
            delete_after_import,
            ignored_updates,
            source,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

pub(super) fn import_file(source: impl Read + Seek, app: &AppHandle) -> Result<ImportData> {
    let thunderstore = app.state::<Mutex<Thunderstore>>();
    let thunderstore = thunderstore.lock().unwrap();

    let temp_dir = tempdir().context("failed to create temporary directory")?;
    util::zip::extract(source, temp_dir.path())?;

    let reader = File::open(temp_dir.path().join("export.r2x"))
        .map(BufReader::new)
        .context("failed to open profile manifest")?;

    let manifest: LegacyProfileManifest =
        serde_yaml::from_reader(reader).context("failed to read profile manifest")?;

    ImportData::create_r2(
        manifest.profile_name,
        manifest.mods,
        manifest.ignored_updates,
        temp_dir.into_path(),
        true,
        manifest.source,
        &thunderstore,
    )
}

pub(super) async fn import_data(
    data: ImportData,
    options: InstallOptions,
    import_all: bool,
    app: &AppHandle,
) -> Result<()> {
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

    install::install_mods(data.mods, options, app)
        .await
        .context("error while importing mods")?;

    let includes = export::find_config(
        &data.path,
        if import_all {
            IncludeExtensions::All
        } else {
            IncludeExtensions::Default
        },
        IncludeGenerated::No,
    );
    import_config(&path, &data.path, includes).context("failed to import config")?;

    if data.delete_after_import {
        fs::remove_dir_all(&data.path).ok();
    }

    Ok(())
}

pub fn import_config(
    target: &Path,
    source: &Path,
    files: impl Iterator<Item = PathBuf>,
) -> Result<()> {
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
