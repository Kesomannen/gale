use std::{collections::HashSet, fs};

use anyhow::Context;

use crate::{
    command_util::{Result, StateMutex},
    fs_util,
    manager::ModManager,
    prefs::Prefs,
    thunderstore::{ModRef, Thunderstore},
};

use super::InstallState;
use itertools::Itertools;

#[tauri::command]
pub async fn install_mod(mod_ref: ModRef, app: tauri::AppHandle) -> Result<()> {
    super::install_with_deps(&mod_ref, true, &app).await?;

    Ok(())
}

#[tauri::command]
pub fn cancel_install(install_state: StateMutex<InstallState>) -> Result<()> {
    install_state.lock().unwrap().cancelled = true;

    Ok(())
}

#[tauri::command]
pub fn clear_download_cache(
    soft: bool,
    prefs: StateMutex<Prefs>,
    manager: StateMutex<ModManager>,
    thunderstore: StateMutex<Thunderstore>,
) -> Result<()> {
    let prefs = prefs.lock().unwrap();
    let cache_dir = prefs.get_path_or_err("cache_dir")?;

    if !cache_dir.exists() {
        return Ok(());
    }

    if soft {
        let manager = manager.lock().unwrap();
        let thunderstore = thunderstore.lock().unwrap();

        let installed_mods = manager
            .active_game()
            .installed_mods(&thunderstore)
            .map_ok(|borrowed| {
                (
                    &borrowed.package.full_name,
                    borrowed.version.version_number.to_string(),
                )
            })
            .collect::<anyhow::Result<HashSet<_>>>()
            .context("failed to resolve installed mods")?;

        let packages = fs::read_dir(cache_dir)
            .context("failed to read cache directory")?
            .filter_map(|e| e.ok());

        for entry in packages {
            let is_dir = match entry.file_type() {
                Ok(ty) => ty.is_dir(),
                Err(_) => false,
            };

            if !is_dir {
                continue;
            }

            let package = fs_util::file_name(&entry.path());

            if thunderstore.find_package(&package).is_err() {
                // package from a game other than the loaded one, skip
                continue;
            }

            let versions = fs::read_dir(entry.path())
                .with_context(|| format!("failed to read cache for {}", &package))?
                .filter_map(|e| e.ok());

            for entry in versions {
                let version = fs_util::file_name(&entry.path());

                if installed_mods.contains(&(&package, version)) {
                    // package is installed, skip
                    continue;
                }

                fs::remove_dir_all(entry.path())
                    .with_context(|| format!("failed to delete cache for {}", &package))?;
            }
        }
    } else {
        fs::remove_dir_all(cache_dir).context("failed to delete cache")?;
        fs::create_dir_all(cache_dir).context("failed to recreate cache directory")?;
    }

    Ok(())
}

#[tauri::command]
pub fn get_download_size(
    mod_ref: ModRef,
    manager: StateMutex<ModManager>,
    thunderstore: StateMutex<Thunderstore>,
    prefs: StateMutex<Prefs>,
) -> Result<u64> {
    let manager = manager.lock().unwrap();
    let thunderstore = thunderstore.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    let result = super::total_download_size(
        mod_ref.borrow(&thunderstore)?,
        manager.active_profile(),
        &prefs,
        &thunderstore,
    );

    Ok(result.unwrap_or(0))
}