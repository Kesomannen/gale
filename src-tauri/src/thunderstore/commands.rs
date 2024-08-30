use crate::{
    logger,
    manager::ModManager,
    util::cmd::{Result, StateMutex},
};

use super::{
    models::FrontendMod,
    query::{self, QueryModsArgs, QueryState},
    ModRef, Thunderstore,
};
use anyhow::anyhow;
use tauri::AppHandle;

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
    let mut state = state.lock().unwrap();
    state.current_query = None;
}

#[tauri::command]
pub fn trigger_mod_fetching(
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
        if let Err(err) = super::fetch_mods(&app, game, write_directly).await {
            logger::log_js_err("error while fetching mods from Thunderstore", &err, &app);
        }
    });

    Ok(())
}

#[tauri::command]
pub fn get_missing_deps(
    mod_ref: ModRef,
    thunderstore: StateMutex<Thunderstore>,
) -> Result<Vec<String>> {
    let thunderstore = thunderstore.lock().unwrap();

    let borrowed = mod_ref.borrow(&thunderstore)?;
    Ok(thunderstore
        .dependencies(borrowed.version)
        .1
        .into_iter()
        .map(String::from)
        .collect())
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
