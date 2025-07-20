use tauri::{command, AppHandle};

use crate::{
    profile::install::InstallResultExt,
    state::ManagerExt,
    thunderstore::ModId,
    util::{self, cmd::Result},
};

use super::{InstallOptions, ModInstall};

#[command]
pub async fn install_mod(id: ModId, app: AppHandle) -> Result<()> {
    let profile_id = app.lock_manager().active_profile().id;
    let install = ModInstall::try_from_id(id, &app.lock_thunderstore())?;

    app.install_queue()
        .install_with_deps(
            vec![install],
            profile_id,
            InstallOptions::default(),
            false,
            &app,
        )?
        .await
        .ignore_cancel()?;

    Ok(())
}

#[command]
pub fn cancel_all_installs(app: AppHandle) -> Result<()> {
    app.app_state().install_queue.cancel_all();

    Ok(())
}

#[command]
pub fn has_pending_installations(app: AppHandle) -> Result<bool> {
    let profile_id = app.lock_manager().active_profile().id;

    let result = app.install_queue().handle().has_any_for_profile(profile_id);

    Ok(result)
}

#[command]
pub async fn clear_download_cache(soft: bool, app: AppHandle) -> Result<u64> {
    if soft {
        let paths = super::cache::prepare_soft_clear(app)?;

        let size = paths.iter().map(util::fs::get_directory_size).sum();

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
    let queue = app.install_queue().handle();

    let size = super::total_download_size(
        mod_ref.borrow(&thunderstore)?,
        manager.active_profile(),
        &prefs,
        &thunderstore,
        &queue,
    );

    Ok(size)
}
