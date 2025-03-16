use eyre::anyhow;
use tauri::{command, AppHandle};

use super::{
    models::FrontendMod,
    query::{self, QueryModsArgs},
};
use crate::{logger, state::ManagerExt, util::cmd::Result};

#[command]
pub fn query_thunderstore(args: QueryModsArgs, app: AppHandle) -> Vec<FrontendMod> {
    let manager = app.lock_manager();
    let mut thunderstore = app.lock_thunderstore();

    let result = query::query_frontend_mods(&args, thunderstore.latest(), manager.active_profile());

    if !thunderstore.packages_fetched {
        thunderstore.current_query = Some(args);
    }

    result
}

#[command]
pub fn stop_querying_thunderstore(app: AppHandle) {
    app.lock_thunderstore().current_query = None;
}

#[command]
pub fn trigger_mod_fetch(app: AppHandle) -> Result<()> {
    let write_directly = {
        let state = app.lock_thunderstore();

        if state.is_fetching {
            return Err(anyhow!("already fetching mods").into());
        }

        !state.packages_fetched
    };

    let game = app.lock_manager().active_game;

    tauri::async_runtime::spawn(async move {
        if let Err(err) = super::fetch::fetch_packages(game, write_directly, &app).await {
            logger::log_webview_err("error while fetching mods from Thunderstore", err, &app);
        }
    });

    Ok(())
}

#[command]
pub fn set_thunderstore_token(token: &str) -> Result<()> {
    super::token::set(token)?;
    Ok(())
}

#[command]
pub fn has_thunderstore_token() -> bool {
    super::token::get().is_ok_and(|token| token.is_some())
}

#[command]
pub fn clear_thunderstore_token() -> Result<()> {
    super::token::clear()?;
    Ok(())
}
