use crate::{
    prefs::Prefs,
    profile::ModManager,
    thunderstore::{ModId, Thunderstore},
    util::cmd::{Result, StateMutex},
};

use super::{InstallOptions, InstallState, ModInstall};

#[tauri::command]
pub async fn install_mod(mod_ref: ModId, app: tauri::AppHandle) -> Result<()> {
    super::install_with_deps(
        vec![ModInstall::new(mod_ref)],
        InstallOptions::default(),
        false,
        &app,
    )
    .await?;

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

    if soft {
        let manager = manager.lock().unwrap();
        let thunderstore = thunderstore.lock().unwrap();

        super::cache::soft_clear(&manager, &thunderstore, &prefs)?;
    } else {
        super::cache::clear(&prefs)?;
    }

    Ok(())
}

#[tauri::command]
pub fn get_download_size(
    mod_ref: ModId,
    manager: StateMutex<ModManager>,
    thunderstore: StateMutex<Thunderstore>,
    prefs: StateMutex<Prefs>,
) -> Result<u64> {
    let manager = manager.lock().unwrap();
    let thunderstore = thunderstore.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    Ok(super::total_download_size(
        mod_ref.borrow(&thunderstore)?,
        manager.active_profile(),
        &prefs,
        &thunderstore,
    ))
}
