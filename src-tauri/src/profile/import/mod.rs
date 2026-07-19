use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::{BufReader, Cursor, Read, Seek},
    path::{Path, PathBuf},
    sync::LazyLock,
};

use base64::{Engine, prelude::BASE64_STANDARD};
use eyre::{Context, Result, eyre};
use futures_util::future;
use globset::{Glob, GlobSet, GlobSetBuilder};
use itertools::Itertools;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tempfile::tempdir;
use tracing::{debug, info, trace, warn};
use uuid::Uuid;
use walkdir::WalkDir;

use crate::{
    prefs::Backends,
    profile::{
        export::{PROFILE_DATA_PREFIX, ProfileManifest},
        install::{InstallOptions, ModInstall},
    },
    state::ManagerExt,
    thunderstore::{ModId, Thunderstore},
    util::{self, error::IoResultExt},
};

pub mod commands;
mod local;
mod r2modman;

use super::Profile;
pub use local::{import_local_mod, import_local_mod_base64};

pub fn read_file_at_path(path: PathBuf, thunderstore: &Thunderstore) -> Result<ImportData> {
    let file = File::open(&path).fs_context("opening file", &path)?;

    read_file(file, thunderstore)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImportData {
    pub manifest: ProfileManifest,
    pub path: PathBuf,
    pub delete_after_import: bool,
}

pub fn read_file_from(path: PathBuf, thunderstore: &Thunderstore) -> Result<ImportData> {
    let file = File::open(&path).fs_context("opening file", &path)?;

    read_file(file, thunderstore)
}

pub(super) fn read_file(
    source: impl Read + Seek,
    thunderstore: &Thunderstore,
) -> Result<ImportData> {
    let temp_dir = tempdir().context("failed to create temporary directory")?;
    util::zip::extract(source, temp_dir.path())?;

    let reader = File::open(temp_dir.path().join("export.r2x"))
        .map(BufReader::new)
        .context("failed to open profile manifest")?;

    let mut manifest: ProfileManifest =
        serde_yaml::from_reader(reader).context("failed to read profile manifest")?;

    for r2mod in manifest.mods.iter_mut() {
        // first try the backend stored in the manifest, if it's not there,
        // then try falling back to checking any other backend and update the source as needed
        if thunderstore
            .backend(r2mod.source)
            .find_ident(&r2mod.version_ident())
            .is_err()
        {
            if let Ok(package) = thunderstore.find_ident(&r2mod.version_ident()) {
                r2mod.source = package.package.backend;
            }
        }
    }

    Ok(ImportData {
        manifest,
        path: temp_dir.keep(),
        delete_after_import: true,
    })
}

fn read_base64(base64: &str, thunderstore: &Thunderstore) -> Result<ImportData> {
    let bytes = BASE64_STANDARD
        .decode(base64)
        .context("failed to decode base64 data")?;

    read_file(Cursor::new(bytes), thunderstore)
}

pub async fn read_code(key: Uuid, app: &AppHandle) -> Result<ImportData> {
    let response = future::join_all(Backends::All.into_backend_slice().iter().map(
        async |b| -> Result<String> {
            Ok(app
                .http()
                .get(b.profile_import(&key.to_string()))
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
                .await?)
        },
    ))
    .await
    .into_iter()
    .find_or_first(|r| r.is_ok())
    .unwrap()?;

    match response.strip_prefix(PROFILE_DATA_PREFIX) {
        Some(str) => read_base64(str, &*app.lock_thunderstore()),
        None => Err(eyre!("invalid profile data")),
    }
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct ImportOptions {
    import_all: bool,
    merge: bool,
}

pub(super) async fn import_profile(
    data: ImportData,
    options: ImportOptions,
    install_options: InstallOptions,
    app: &AppHandle,
) -> Result<i64> {
    info!(
        name = %data.manifest.name,
        options = ?options,
        install_options = ?install_options,
        "importing profile"
    );

    let (profile_id, profile_path, to_install) = prepare_import(&options, data.manifest, app)?;

    let result = app
        .install_queue()
        .install(to_install, profile_id, install_options, app)
        .await;

    let result = match result {
        Ok(()) => {
            import_config(&profile_path, &data.path, &options).context("error importing config")?;

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
    options: &ImportOptions,
    manifest: ProfileManifest,
    app: &AppHandle,
) -> Result<(i64, PathBuf, Vec<ModInstall>)> {
    let ProfileManifest {
        name,
        mods,
        ignored_version_updates,
        ignored_package_updates,
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
            let to_install = incremental_update(options.merge, installs, profile)?.collect_vec();

            (profile, to_install)
        }
        None => {
            let profile = game.create_profile(name, None, app.db())?;

            (profile, installs)
        }
    };

    profile.ignored_version_updates = ignored_version_updates.into_iter().collect();
    profile.ignored_package_updates = ignored_package_updates.into_iter().collect();

    let id = profile.id;
    let path = profile.path.clone();

    game.save(app)?;

    Ok((id, path, to_install))
}

fn cleanup_failed_profile(profile_id: i64, app: &AppHandle) -> Result<()> {
    let mut manager = app.lock_manager();

    let (game, _) = manager.profile_by_id(profile_id)?;
    let managed_game = manager.games.get_mut(game).unwrap();

    if managed_game.profiles.len() > 1 {
        managed_game.delete_profile(profile_id, false, app.db())?;
        managed_game.save(app)?;
    } else {
        warn!("import failed for last profile")
    }

    Ok(())
}

fn incremental_update(
    merge: bool,
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

    if merge {
        // remove only version mismatches
        let to_remove = current_mods.keys().filter(|id| {
            new_mods.values().any(|install| {
                install.mod_id().package_uuid == id.package_uuid
                    && install.mod_id().version_uuid != id.version_uuid
            })
        });
        for mod_id in to_remove {
            profile.force_remove_mod(mod_id.package_uuid)?;
        }
    } else {
        // remove all extra mods
        let to_remove = current_ids.difference(&new_ids);
        for mod_id in to_remove {
            profile.force_remove_mod(mod_id.package_uuid)?;
        }
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

pub fn import_config(dest: &Path, src: &Path, options: &ImportOptions) -> Result<()> {
    let src_files = WalkDir::new(src)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| {
            entry
                .into_path()
                .strip_prefix(src)
                .expect("path should be child of source")
                .to_path_buf()
        })
        .filter(|path| options.import_all || is_always_included(path));

    debug!("importing config files from source to destination");

    for file in src_files {
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
            trace!(
                relative_path = %file.display(),
                "copy file"
            );
            fs::create_dir_all(dest_path.parent().unwrap())?;
            fs::copy(src_path, dest_path)?;
        } else {
            trace!(
                relative_path = %file.display(),
                "file is identical, skipping copy"
            );
        }
    }

    Ok(())
}

fn is_always_included(path: impl AsRef<Path>) -> bool {
    static EXCLUDE_SET: LazyLock<GlobSet> = LazyLock::new(|| {
        GlobSetBuilder::new()
            .add(Glob::new("export.r2x").unwrap())
            .add(Glob::new("mods.yml").unwrap())
            .add(Glob::new("*.{dll,exe,scr,com,pif,bat,cmd,ps1,vbs,vbe,js,jse,wsf,wsh,hta,msi,msix,sys,drv,cpl,ocx,lnk,reg,inf}").unwrap())
            .build()
            .unwrap()
    });

    !EXCLUDE_SET.is_match(path.as_ref())
}
