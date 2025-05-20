use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::{self, BufReader, Cursor, Read, Seek},
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
        export::{ProfileManifest, R2Mod, PROFILE_DATA_PREFIX},
        install::{self, InstallOptions, ModInstall},
    },
    state::ManagerExt,
    thunderstore::VersionIdent,
    util::{self, error::IoResultExt},
};

pub mod commands;
mod local;
mod r2modman;

pub use local::{import_local_mod, import_local_mod_base64};

use super::export::{self, IncludeExtensions, IncludeGenerated};

pub fn read_file_at_path(path: PathBuf) -> Result<ImportData> {
    let file = File::open(&path).fs_context("opening file", &path)?;

    read_file(file)
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImportData {
    pub manifest: ProfileManifest,
    pub path: PathBuf,
    pub delete_after_import: bool,
}

pub fn import_file_from_path(path: PathBuf) -> Result<ImportData> {
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
        path: temp_dir.into_path(),
        delete_after_import: true,
    })
}

fn read_base64(base64: &str) -> Result<ImportData> {
    let bytes = BASE64_STANDARD
        .decode(base64)
        .context("failed to decode base64 data")?;

    read_file(Cursor::new(bytes))
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
        Some(str) => read_base64(str),
        None => Err(eyre!("invalid profile data")),
    }
}

pub(super) async fn import_profile(
    data: ImportData,
    options: InstallOptions,
    import_all: bool,
    app: &AppHandle,
) -> Result<usize> {
    let ImportData {
        manifest:
            ProfileManifest {
                name,
                mods,
                ignored_updates,
                ..
            },
        path,
        delete_after_import,
    } = data;

    let (index, profile_path, to_install) = {
        let (names, installs) = resolve_mods(mods, app)?;

        let mut manager = app.lock_manager();
        let game = manager.active_game_mut();

        let (index, profile, to_install) = if let Some(index) = game.profile_index(&name) {
            game.set_active_profile(index)?;

            let profile = &mut game.profiles[index];
            let to_install = incremental_update(installs, &names, profile)?;

            (index, profile, to_install)
        } else {
            let index = game.profiles.len();
            let profile = game.create_profile(name, None, app.db())?;

            (index, profile, installs)
        };

        profile.ignored_updates = ignored_updates.into_iter().collect();
        (index, profile.path.clone(), to_install)
    };

    install::install_mods(to_install, options, app)
        .await
        .context("error while importing mods")?;

    import_config(
        &profile_path,
        &path,
        if import_all {
            IncludeExtensions::All
        } else {
            IncludeExtensions::Default
        },
        IncludeGenerated::No,
    )
    .context("failed to import config")?;

    if delete_after_import {
        fs::remove_dir_all(path).ok();
    }

    Ok(index)
}

fn resolve_mods(mods: Vec<R2Mod>, app: &AppHandle) -> Result<(Vec<VersionIdent>, Vec<ModInstall>)> {
    let thunderstore = app.lock_thunderstore();

    let mut names = Vec::with_capacity(mods.len());
    let mut installs = Vec::with_capacity(mods.len());

    for r2_mod in mods {
        let name = r2_mod.ident();
        if let Ok(install) = r2_mod.into_install(&thunderstore) {
            names.push(name);
            installs.push(install);
        } else {
            warn!("failed to resolve mod from import: {}", name);
        }
    }

    Ok((names, installs))
}

fn file_checksum(path: &Path) -> io::Result<blake3::Hash> {
    let mut file = File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 4096];

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    Ok(hasher.finalize())
}

fn incremental_update(
    installs: Vec<ModInstall>,
    mod_names: &[VersionIdent],
    profile: &mut super::Profile,
) -> Result<Vec<ModInstall>> {
    let old: HashMap<_, _> = profile
        .thunderstore_mods()
        .map(|(ts_mod, enabled)| (ts_mod.ident.clone(), (ts_mod.id.package_uuid, enabled)))
        .collect();

    let mut new: HashMap<_, _> = mod_names.iter().zip(installs).collect();

    let old_keys: HashSet<_> = old.keys().collect();
    let new_keys: HashSet<_> = new.keys().copied().collect();
    let to_remove = old_keys.difference(&new_keys);

    for ident in to_remove {
        let (uuid, _) = old.get(ident).unwrap();
        profile.force_remove_mod(*uuid)?;
    }

    let to_toggle = old_keys
        .intersection(&new_keys)
        .filter(|k| old.get(k).unwrap().1 != new.get(*k).unwrap().enabled());

    for ident in to_toggle {
        let (uuid, _) = old.get(ident).unwrap();
        profile.force_toggle_mod(*uuid)?;
    }

    let to_install = new_keys
        .difference(&old_keys)
        .map(|key| new.remove(key).unwrap())
        .collect_vec();

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
            file_checksum(&src_path)? != file_checksum(&dest_path)?
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
