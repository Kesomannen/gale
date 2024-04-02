use std::{
    fs,
    sync::atomic::{self, AtomicUsize},
};

use anyhow::Context;
use itertools::Itertools;
use tauri::Manager;
use uuid::Uuid;

use crate::{
    manager::{self, commands::{save, save_unlocked}, downloader::ModDownloadData, ModManager},
    prefs::PrefsState,
    thunderstore::{self, BorrowedMod, ThunderstoreState},
    util, NetworkClient,
};

type Result<T> = util::CommandResult<T>;

#[tauri::command]
pub async fn install_mod(
    package_uuid: Uuid,
    app: tauri::AppHandle,
    manager: tauri::State<'_, ModManager>,
    prefs: tauri::State<'_, PrefsState>,
    thunderstore: tauri::State<'_, ThunderstoreState>,
    network_client: tauri::State<'_, NetworkClient>,
) -> Result<()> {
    let (to_download, total, target_path, cache_path) = {
        println!("installing mod: {}", package_uuid);
        let prefs = prefs.lock();
        let cache_path = prefs.cache_path.clone();

        let mut profiles = manager.profiles.lock().unwrap();
        let profile = manager::get_active_profile(&mut profiles, &manager)?;

        let packages = thunderstore.packages.lock().unwrap();
        let package = thunderstore::get_package(&package_uuid, &packages)?;
        let target_mod = BorrowedMod {
            package,
            version: &package.versions[0],
        };

        let to_install = profile.prepare_install(target_mod, &packages)?;
        let total = to_install.len();

        let to_download = profile.install_from_cache(to_install, &cache_path)
            .context("failed to install from cache")?
            .into_iter()
            .map(|mod_to_download| ModDownloadData::from(&mod_to_download))
            .collect_vec();

        (to_download, total, profile.path.clone(), cache_path)
    };

    save(&manager, &prefs)?;

    let completed = total - to_download.len();
    let _ = app.emit_all("install_progress", (completed, total));
    let completed = AtomicUsize::new(completed);

    Ok(super::install_by_download(
        to_download,
        &cache_path,
        &target_path,
        &network_client.0,
        |profile_mod| {
            let current = completed.fetch_add(1, atomic::Ordering::SeqCst) + 1;
            let _ = app.emit_all("install_progress", (current, total));

            {
                let mut profiles = manager.profiles.lock().unwrap();
                let profile = manager::get_active_profile(&mut profiles, &manager)?;
                
                profile.mods.push(profile_mod);
            }

            save_unlocked(&manager, &prefs.lock())?;
            Ok(())
        },
    )
    .await?)
}

#[tauri::command]
pub fn clear_download_cache(prefs: tauri::State<PrefsState>) -> Result<()> {
    let cache_path = prefs.lock().cache_path.clone();
    if cache_path.try_exists().unwrap_or(false) {
        fs::remove_dir_all(&cache_path).context("failed to delete cache dir")?;
    }

    fs::create_dir_all(&cache_path).context("failed to recreate cache dir")?;
    Ok(())
}
