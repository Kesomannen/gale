use std::sync::atomic::Ordering;

use tauri::{command, AppHandle};

use crate::{
    state::ManagerExt,
    thunderstore::ModId,
    util::{self, cmd::Result},
};

use super::{InstallOptions, ModInstall};

#[command]
pub async fn install_mod(mod_ref: ModId, app: AppHandle) -> Result<()> {
    super::install_with_deps(
        vec![ModInstall::new(mod_ref)],
        InstallOptions::default(),
        false,
        &app,
    )
    .await?;

    Ok(())
}

#[command]
pub fn cancel_install(app: AppHandle) -> Result<()> {
    app.app_state()
        .cancel_install_flag
        .store(true, Ordering::Relaxed);

    Ok(())
}

#[command]
pub async fn clear_download_cache(soft: bool, app: AppHandle) -> Result<u64> {
    if soft {
        let paths = super::cache::prepare_soft_clear(app)?;

        let size = paths
            .iter()
            .map(|path| util::fs::get_directory_size(path))
            .sum();

        tauri::async_runtime::spawn_blocking(|| super::cache::do_soft_clear(paths)).await??;

        Ok(size)
    } else {
        let path = app.lock_prefs().cache_dir();

        let size = util::fs::get_directory_size(&path);

        tauri::async_runtime::spawn_blocking(|| super::cache::clear(path)).await??;

        Ok(size)
    }
}

#[command]
pub fn get_download_size(mod_ref: ModId, app: AppHandle) -> Result<u64> {
    let prefs = app.lock_prefs();
    let manager = app.lock_manager();
    let thunderstore = app.lock_thunderstore();

    Ok(super::total_download_size(
        mod_ref.borrow(&thunderstore)?,
        manager.active_profile(),
        &prefs,
        &thunderstore,
    ))
}
