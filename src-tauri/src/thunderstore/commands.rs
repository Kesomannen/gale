use eyre::{anyhow, OptionExt};
use tauri::{command, AppHandle};

use super::{
    cache::MarkdownCache,
    models::FrontendMod,
    query::{self, QueryModsArgs},
    ModId,
};
use crate::{
    game::{self},
    logger,
    state::ManagerExt,
    util::cmd::Result,
};

#[command]
pub fn query_thunderstore(
    args: QueryModsArgs,
    profile_id: i64,
    app: AppHandle,
) -> Result<Vec<FrontendMod>> {
    let manager = app.lock_manager();
    let mut thunderstore = app.lock_thunderstore();

    let mods =
        query::query_frontend_mods(&args, thunderstore.latest(), manager.profile(profile_id)?);

    if !thunderstore.packages_fetched {
        thunderstore.current_query = Some((profile_id, args));
    }

    Ok(mods)
}

#[command]
pub fn stop_querying_thunderstore(app: AppHandle) {
    app.lock_thunderstore().current_query = None;
}

#[command]
pub fn trigger_mod_fetch(game_slug: String, app: AppHandle) -> Result<()> {
    let write_directly = {
        let state = app.lock_thunderstore();

        if state.is_fetching {
            return Err(anyhow!("already fetching mods").into());
        }

        !state.packages_fetched
    };

    let game = game::from_slug(&game_slug).ok_or_eyre("game not found")?;

    tauri::async_runtime::spawn(async move {
        if let Err(err) = super::fetch::fetch_packages(game, write_directly, &app).await {
            logger::log_webview_err("error while fetching mods from Thunderstore", err, &app);
        }
    });

    Ok(())
}

#[command]
pub async fn get_markdown(
    mod_ref: ModId,
    kind: MarkdownCache,
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
