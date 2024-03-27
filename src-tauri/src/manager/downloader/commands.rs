use std::{fs, sync::atomic::{self, AtomicUsize}};

use anyhow::{anyhow, Context};
use tauri::Manager;
use uuid::Uuid;

use crate::{
    manager::{self, ModManager}, 
    prefs::PrefsState, 
    thunderstore::{BorrowedMod, ThunderstoreState}, 
    util, 
    NetworkClient
};

type Result<T> = util::CommandResult<T>;

#[tauri::command]
pub async fn install_mod(
    package_uuid: Uuid,
    app: tauri::AppHandle,
    manager: tauri::State<'_, ModManager>,
    config: tauri::State<'_, PrefsState>,
    thunderstore: tauri::State<'_, ThunderstoreState>,
    network_client: tauri::State<'_, NetworkClient>,
) -> Result<()> {
    if !*thunderstore.finished_loading.lock().unwrap() {
        return Err(anyhow!("all mods not loaded yet").into());
    }

    let (to_download, target_path, cache_path) = {
        println!("installing mod: {}", package_uuid);
        let config = config.lock();
        let cache_path = config.cache_path.clone();

        let mut profiles = manager.profiles.lock().unwrap();
        let profile = manager::get_active_profile(&mut profiles, &manager)?;

        let mod_map = thunderstore.packages.lock().unwrap();
        let package = mod_map.get(&package_uuid).context("mod not found")?;
        let target_mod = BorrowedMod {
            package,
            version: &package.versions[0],
        };

        let to_download = profile.install(target_mod, &cache_path, &mod_map)?;

        (to_download, profile.path.clone(), cache_path)
    };

    manager.save(&config.lock())?;

    let completed = AtomicUsize::new(0);
    let total = to_download.len();

    let _ = app.emit_all("install_progress", (0, total));

    Ok(
        super::install_by_download(
            to_download,
            &cache_path,
            &target_path,
            &network_client.0,
            || {
                let current = completed.fetch_add(1, atomic::Ordering::SeqCst) + 1;
                let _ = app.emit_all("install_progress", (current, total));
            },
        )
        .await?
    )
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
