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
use tracing::{trace, warn};
use uuid::Uuid;

use crate::{
    profile::{
        export::{ProfileManifest, PROFILE_DATA_PREFIX},
        install::{InstallOptions, ModInstall},
    },
    state::ManagerExt,
    thunderstore::ModId,
    util::{self, error::IoResultExt},
};

pub mod commands;
mod local;
mod r2modman;

pub use local::{import_local_mod, import_local_mod_base64};

use super::{
    export::{self, IncludeExtensions, IncludeGenerated},
    Profile,
};

pub fn read_file_at_path(path: PathBuf) -> Result<ImportData> {
    let file = File::open(&path).fs_context("opening file", &path)?;

    read_file(file)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImportData {
    pub manifest: ProfileManifest,
    pub path: PathBuf,
    pub delete_after_import: bool,
}

pub fn read_file_from(path: PathBuf) -> Result<ImportData> {
    let file = File::open(&path).fs_context("opening file", &path)?;

    read_file(file)
}

pub(super) fn read_file(source: impl Read + Seek) -> Result<ImportData> {
    let temp_dir = tempdir().context("failed to create temporary directory")?;
    util::zip::extract(source, temp_dir.path())?;

    let reader = File::open(temp_dir.path().join("export.r2x"))
        .map(BufReader::new)
        .context("failed to open profile manifest")?;

    let manifest: ProfileManifest =
        serde_yaml::from_reader(reader).context("failed to read profile manifest")?;

    Ok(ImportData {
        manifest,
        path: temp_dir.keep(),
        delete_after_import: true,
    })
}

fn read_base64(base64: &str) -> Result<ImportData> {
    let bytes = BASE64_STANDARD
        .decode(base64)
        .context("failed to decode base64 data")?;

    read_file(Cursor::new(bytes))
}

pub async fn read_code(key: Uuid, app: &AppHandle) -> Result<ImportData> {
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
        Some(str) => read_base64(str),
        None => Err(eyre!("invalid profile data")),
    }
}

#[derive(Debug, Default)]
pub struct ImportOptions {
    import_all: bool,
}

impl ImportOptions {
    fn included_extensions(&self) -> IncludeExtensions {
        if self.import_all {
            IncludeExtensions::All
        } else {
            IncludeExtensions::Default
        }
    }

    pub fn import_all(mut self, value: bool) -> Self {
        self.import_all = value;
        self
    }
}

pub(super) async fn import_profile(
    data: ImportData,
    options: ImportOptions,
    install_options: InstallOptions,
    app: &AppHandle,
) -> Result<i64> {
    let (profile_id, profile_path, to_install) = prepare_import(data.manifest, app)?;

    let result = app
        .install_queue()
        .install(to_install, profile_id, install_options, app)
        .await;

    let result = match result {
        Ok(()) => {
            import_config(
                &profile_path,
                &data.path,
                options.included_extensions(),
                IncludeGenerated::No,
            )
            .context("error importing config")?;

            Ok(profile_id)
        }
        Err(err) => {
            cleanup_failed_profile(profile_id, app).unwrap_or_else(|err| {
                warn!(
                    "failed to remove profile after failed or cancelled import: {}",
                    err
                );
            });

            Err(err.into())
        }
    };

    if data.delete_after_import {
        fs::remove_dir_all(&data.path).unwrap_or_else(|err| {
            warn!("failed to remove source folder after import: {}", err);
        });
    }

    result
}

fn prepare_import(
    manifest: ProfileManifest,
    app: &AppHandle,
) -> Result<(i64, PathBuf, Vec<ModInstall>)> {
    let ProfileManifest {
        name,
        mods,
        ignored_updates,
        ..
    } = manifest;

    let mut manager = app.lock_manager();
    let thunderstore = app.lock_thunderstore();

    let installs = mods
        .into_iter()
        .map(|r2_mod| r2_mod.into_install(&thunderstore))
        .collect::<Result<Vec<_>>>()?;

    let game = manager.active_game_mut();

    let (profile, to_install) = match game.find_profile_index(&name) {
        Some(profile_index) => {
            // overwrite an existing profile
            let profile = game.set_active_profile(profile_index)?;
            let to_install = incremental_update(installs, profile)?.collect_vec();

            (profile, to_install)
        }
        None => {
            let profile = game.create_profile(name, None, app.db())?;

            (profile, installs)
        }
    };

    profile.ignored_updates = ignored_updates.into_iter().collect();

    let id = profile.id;
    let path = profile.path.clone();

    game.save(app)?;

    Ok((id, path, to_install))
}

fn cleanup_failed_profile(profile_id: i64, app: &AppHandle) -> Result<()> {
    let mut manager = app.lock_manager();

    let (game, _) = manager.profile_by_id(profile_id)?;
    let managed_game = manager.games.get_mut(game).unwrap();

    let index = managed_game.index_of(profile_id)?;

    managed_game.delete_profile(index, false, app.db())?;
    managed_game.save(app)?;

    Ok(())
}

fn incremental_update(
    installs: impl IntoIterator<Item = ModInstall>,
    profile: &mut Profile,
) -> Result<impl Iterator<Item = ModInstall>> {
    let current_mods: HashMap<ModId, bool> = profile
        .thunderstore_mods()
        .map(|(ts_mod, enabled)| (ts_mod.id.clone(), enabled))
        .collect();

    let current_ids: HashSet<&ModId> = current_mods.keys().collect();

    let mut new_mods: HashMap<ModId, ModInstall> = installs
        .into_iter()
        .map(|install| (install.mod_id().clone(), install))
        .collect();

    let new_ids: HashSet<&ModId> = new_mods.keys().collect();

    let to_remove = current_ids.difference(&new_ids);
    for mod_id in to_remove {
        profile.force_remove_mod(mod_id.package_uuid)?;
    }

    let to_toggle = current_ids
        .intersection(&new_ids)
        .filter(|id| *current_mods.get(*id).unwrap() != new_mods.get(id).unwrap().enabled());
    for mod_id in to_toggle {
        profile.force_toggle_mod(mod_id.package_uuid)?;
    }

    // we have to clone and collect the ids because new_ids.difference() borrows new_mods,
    // which prevents us from getting the ModInstalls back (since that requires mutably borrowing the map)

    let ids_to_install: Vec<ModId> = new_ids
        .difference(&current_ids)
        .map(|id| (*id).clone())
        .collect();

    let to_install = ids_to_install
        .into_iter()
        .map(move |id| new_mods.remove(&id).unwrap());

    Ok(to_install)
}

pub fn import_config(
    dest: &Path,
    src: &Path,
    extensions: IncludeExtensions,
    generated: IncludeGenerated,
) -> Result<()> {
    let existing_files = export::find_config(dest, extensions, generated);
    let source_files = export::find_config(src, extensions, generated);

    if extensions != IncludeExtensions::All {
        for file in existing_files {
            let exists = src.join(&file).exists()
                || file
                    .strip_prefix("BepInEx/config")
                    .is_ok_and(|suffix| src.join("config").join(suffix).exists());

            if !exists {
                trace!("remove {}", file.display());
                fs::remove_file(dest.join(&file))?;
            }
        }
    }

    for file in source_files {
        let src_path = src.join(&file);
        let dest_path = if file.starts_with("config") {
            dest.join("BepInEx").join(&file)
        } else {
            dest.join(&file)
        };

        let need_copy = if dest_path.exists() {
            util::fs::checksum(&src_path)? != util::fs::checksum(&dest_path)?
        } else {
            true
        };

        if need_copy {
            trace!("copy {}", file.display());
            fs::create_dir_all(dest_path.parent().unwrap())?;
            fs::copy(src_path, dest_path)?;
        }
    }

    Ok(())
}
