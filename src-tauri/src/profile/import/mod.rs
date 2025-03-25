use std::{
    fs::{self, File},
    io::{BufReader, Cursor, Read, Seek},
    path::{Path, PathBuf},
};

use base64::{prelude::BASE64_STANDARD, Engine};
use eyre::{anyhow, Context, Result};
use itertools::Itertools;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tempfile::tempdir;
use uuid::Uuid;

use crate::{
    profile::{
        export::{self, ImportSource, LegacyProfileManifest, R2Mod, PROFILE_DATA_PREFIX},
        install::{self, InstallOptions, ModInstall},
    },
    state::ManagerExt,
    thunderstore::Thunderstore,
    util::{self, error::IoResultExt},
};

pub mod commands;
mod local;
mod r2modman;

pub use local::import_local_mod;

use super::export::{IncludeExtensions, IncludeGenerated};

pub fn import_file_from_path(path: PathBuf, app: &AppHandle) -> Result<ImportData> {
    let file = File::open(&path).fs_context("opening file", &path)?;

    import_file(file, app)
}

#[derive(Serialize, Deserialize, Clone)]
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
}

fn import_file(source: impl Read + Seek, app: &AppHandle) -> Result<ImportData> {
    let thunderstore = app.lock_thunderstore();

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

async fn import_data(
    data: ImportData,
    options: InstallOptions,
    import_all: bool,
    app: &AppHandle,
) -> Result<()> {
    let path = {
        let mut manager = app.lock_manager();

        let game = manager.active_game_mut();
        if let Some(index) = game.profiles.iter().position(|p| p.name == data.name) {
            game.delete_profile(index, true, app.db())
                .context("failed to delete existing profile")?;
        }

        let profile = game.create_profile(data.name, None, app.db())?;
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
    let response = app
        .http()
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
