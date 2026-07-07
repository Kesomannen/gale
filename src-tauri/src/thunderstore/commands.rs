use eyre::anyhow;
use tauri::{AppHandle, command};

use super::{
    models::FrontendMod,
    query::{self, QueryModsArgs},
};
use crate::{
    logger,
    state::ManagerExt,
    thunderstore::{ModId, cache::MarkdownKind},
    util::cmd::Result,
};

#[command]
pub fn query_thunderstore(args: QueryModsArgs, app: AppHandle) -> Vec<FrontendMod> {
    let manager = app.lock_manager();
    let mut thunderstore = app.lock_thunderstore();

    let result = query::query_frontend_mods(&args, thunderstore.latest(), manager.active_profile());

    if !thunderstore.packages_fetched(&app, manager.active_game) {
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
    let game = app.lock_manager().active_game;

    let write_directly = {
        let state = app.lock_thunderstore();

        if state.is_fetching {
            return Err(anyhow!("already fetching mods").into());
        }

        !state.packages_fetched(&app, game)
    };

    tauri::async_runtime::spawn(async move {
        for (backend, err) in super::fetch::fetch_packages(game, write_directly, &app).await {
            logger::log_webview_err(
                format!("error while fetching mods from {:?}", backend),
                err,
                &app,
            );
        }
    });

    Ok(())
}

#[command]
pub async fn get_markdown(
    mod_ref: ModId,
    kind: MarkdownKind,
    app: AppHandle,
) -> Result<Option<String>> {
    let content = super::cache::get_markdown(kind, mod_ref, &app).await?;
    Ok(content)
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
