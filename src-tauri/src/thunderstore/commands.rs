use anyhow::anyhow;
use tauri::AppHandle;

use super::{
    models::FrontendMod,
    query::{self, QueryModsArgs, QueryState},
    Thunderstore,
};
use crate::{
    logger,
    profile::ModManager,
    util::cmd::{Result, StateMutex},
};

#[tauri::command]
pub fn query_thunderstore(
    args: QueryModsArgs,
    thunderstore: StateMutex<Thunderstore>,
    state: StateMutex<QueryState>,
    manager: StateMutex<ModManager>,
) -> Vec<FrontendMod> {
    let thunderstore = thunderstore.lock().unwrap();
    let manager = manager.lock().unwrap();

    let result = query::query_frontend_mods(&args, thunderstore.latest(), manager.active_profile());

    if !thunderstore.packages_fetched {
        let mut state = state.lock().unwrap();
        state.current_query = Some(args);
    }

    result
}

#[tauri::command]
pub fn stop_querying_thunderstore(state: StateMutex<QueryState>) {
    state.lock().unwrap().current_query = None;
}

#[tauri::command]
pub fn trigger_mod_fetch(
    app: AppHandle,
    state: StateMutex<Thunderstore>,
    manager: StateMutex<ModManager>,
) -> Result<()> {
    let state = state.lock().unwrap();

    if state.is_fetching {
        return Err(anyhow!("already fetching mods").into());
    }

    let write_directly = !state.packages_fetched;
    let game = manager.lock().unwrap().active_game;

    tauri::async_runtime::spawn(async move {
        if let Err(err) = super::fetch::fetch_packages(&app, game, write_directly).await {
            logger::log_js_err("error while fetching mods from Thunderstore", &err, &app);
        }
    });

    Ok(())
}

#[tauri::command]
pub fn set_thunderstore_token(token: &str) -> Result<()> {
    super::token::set(token)?;
    Ok(())
}

#[tauri::command]
pub fn has_thunderstore_token() -> bool {
    super::token::get().is_ok_and(|token| token.is_some())
}

#[tauri::command]
pub fn clear_thunderstore_token() -> Result<()> {
    super::token::clear()?;
    Ok(())
}
