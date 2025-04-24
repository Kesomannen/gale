use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::{BufReader, Cursor, Read, Seek},
    path::{Path, PathBuf},
};

use base64::{prelude::BASE64_STANDARD, Engine};
use eyre::{eyre, Context, Result};
use itertools::Itertools;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tempfile::tempdir;
use tracing::info;
use uuid::Uuid;

use crate::{
    profile::{
        export::{self, LegacyProfileManifest, R2Mod, PROFILE_DATA_PREFIX},
        install::{self, InstallOptions, ModInstall},
    },
    state::ManagerExt,
    thunderstore::{Thunderstore, VersionIdent},
    util::{self, error::IoResultExt},
};

pub mod commands;
mod local;
mod r2modman;

pub use local::{import_local_mod, import_local_mod_base64};

use super::export::{IncludeExtensions, IncludeGenerated};

pub fn import_file_from_path(path: PathBuf, app: &AppHandle) -> Result<ImportData> {
    let file = File::open(&path).fs_context("opening file", &path)?;

    read_file(file, app)
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImportData {
    pub name: String,
    game: Option<String>,
    mod_names: Vec<VersionIdent>,
    mods: Vec<ModInstall>,
    path: PathBuf,
    delete_after_import: bool,
    ignored_updates: Vec<Uuid>,
}

impl ImportData {
    pub fn create_r2(
        name: String,
        game: Option<String>,
        mods: Vec<R2Mod>,
        ignored_updates: Vec<Uuid>,
        path: PathBuf,
        delete_after_import: bool,
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
            game,
            mod_names,
            mods,
            path,
            delete_after_import,
            ignored_updates,
        })
    }
}

pub(super) fn read_file(source: impl Read + Seek, app: &AppHandle) -> Result<ImportData> {
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
        manifest.community,
        manifest.mods,
        manifest.ignored_updates,
        temp_dir.into_path(),
        true,
        &thunderstore,
    )
}

pub(super) async fn import_profile(
    data: ImportData,
    options: InstallOptions,
    import_all: bool,
    app: &AppHandle,
) -> Result<()> {
    let (path, to_install) = {
        let mut manager = app.lock_manager();
        let game = manager.active_game_mut();

        let (profile, to_install) = if let Some(index) = game.profile_index(&data.name) {
            game.set_active_profile(index)?;

            let profile = &mut game.profiles[index];
            let to_install = incremental_update(data.mods, &data.mod_names, profile)?;
            (profile, to_install)
        } else {
            (game.create_profile(data.name, None, app.db())?, data.mods)
        };

        profile.ignored_updates = data.ignored_updates.into_iter().collect();
        (profile.path.clone(), to_install)
    };

    install::install_mods(to_install, options, app)
        .await
        .context("error while importing mods")?;

    /*
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
    */

    if data.delete_after_import {
        fs::remove_dir_all(&data.path).ok();
    }

    Ok(())
}

fn incremental_update(
    mods: Vec<ModInstall>,
    mod_names: &[VersionIdent],
    profile: &mut super::Profile,
) -> Result<Vec<ModInstall>> {
    let old: HashMap<_, _> = profile
        .thunderstore_mods()
        .map(|(ts_mod, enabled)| (ts_mod.ident.clone(), (ts_mod.id.package_uuid, enabled)))
        .collect();

    let mut new: HashMap<_, _> = mod_names
        .iter()
        .zip(mods)
        .map(|(ident, install)| (ident, install))
        .collect();

    let old_keys: HashSet<_> = old.keys().collect();
    let new_keys: HashSet<_> = new.keys().map(|key| *key).collect();
    let to_remove = old_keys.difference(&new_keys);

    for ident in to_remove {
        info!("removing {}", ident);
        let (uuid, _) = old.get(&ident).unwrap();
        profile.force_remove_mod(*uuid)?;
    }

    let to_toggle = old_keys
        .intersection(&new_keys)
        .filter(|k| old.get(k).unwrap().1 != new.get(*k).unwrap().enabled());

    for ident in to_toggle {
        info!("toggling {}", ident);
        let (uuid, _) = old.get(&ident).unwrap();
        profile.force_toggle_mod(*uuid)?;
    }

    let to_install = new_keys
        .difference(&old_keys)
        .map(|key| new.remove(key).unwrap())
        .collect_vec();

    Ok(to_install)
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

async fn read_code(key: Uuid, app: &AppHandle) -> Result<ImportData> {
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
                eyre!("profile code is expired or invalid")
            }
            _ => err.into(),
        })?
        .text()
        .await?;

    match response.strip_prefix(PROFILE_DATA_PREFIX) {
        Some(str) => read_base64(str, app),
        None => Err(eyre!("invalid profile data")),
    }
}

fn read_base64(base64: &str, app: &AppHandle) -> Result<ImportData> {
    let bytes = BASE64_STANDARD
        .decode(base64)
        .context("failed to decode base64 data")?;

    read_file(Cursor::new(bytes), app)
}
