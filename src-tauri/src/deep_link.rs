use std::{fmt::Debug, future::Future, path::PathBuf};

use eyre::{Context, OptionExt, Result};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::{
    logger,
    profile::{self},
    state::ManagerExt,
    thunderstore::{self, IntoFrontendMod},
};

pub fn handle(app: &AppHandle, args: Vec<String>) -> bool {
    let Some(url) = args.into_iter().nth(1) else {
        debug!("deep link has too few arguments");
        return false;
    };

    app.get_webview_window("main")
        .expect("app should have main window")
        .set_focus()
        .ok();

    let app = app.to_owned();

    if url.starts_with("ror2mm://") {
        handle_inner_task(app.clone(), handle_r2_install(url, app));
    } else if url.starts_with("gale://auth/callback") {
        handle_inner_task(app.clone(), async move {
            profile::sync::auth::handle_callback(url, &app).await
        });
    } else if url.starts_with("gale://profile/import") {
        handle_inner_task(app.clone(), import_profile_code(url, app));
    } else if url.starts_with("gale://profile/sync/clone") {
        handle_inner_task(app.clone(), clone_sync_profile(url, app));
    } else if url.ends_with("r2z") {
        handle_inner_task(app.clone(), async move { import_profile_file(&url, &app) });
    } else {
        warn!("unsupported deep link protocol: {}", url);
        return false;
    }

    true
}

fn handle_inner_task<T, Fut>(app: AppHandle, task: Fut)
where
    Fut: Future<Output = Result<T>> + Send + 'static,
    T: Debug,
{
    tauri::async_runtime::spawn(async move {
        if let Err(err) = task.await {
            logger::log_webview_err("Failed to handle deep link", err, &app);
        }
    });
}

async fn handle_r2_install(url: String, app: AppHandle) -> Result<()> {
    thunderstore::wait_for_fetch(&app).await;

    let (owner, name, version) = url
        .strip_prefix("ror2mm://v1/install/thunderstore.io/")
        .and_then(|path| {
            let mut split = path.split('/');

            Some((split.next()?, split.next()?, split.next()?))
        })
        .ok_or_eyre("invalid package url")?;

    let thunderstore = app.lock_thunderstore();
    let borrowed_mod = thunderstore.find_mod(owner, name, version)?;
    let frontend_mod = borrowed_mod.into_frontend(None);

    app.emit("install_mod", frontend_mod)?;

    Ok(())
}

fn import_profile_file(url: &str, app: &AppHandle) -> Result<()> {
    let path = PathBuf::from(url);

    info!(
        "importing profile file from deep link at {}",
        path.display()
    );

    let import_data = profile::import::read_file_at_path(path)?;

    app.emit("import_profile", ImportProfilePayload::Normal(import_data))?;

    Ok(())
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum ImportProfilePayload {
    Normal(profile::import::ImportData),
    Sync(profile::sync::SyncProfileMetadata),
}

async fn import_profile_code(url: String, app: AppHandle) -> Result<()> {
    let key = url
        .strip_prefix("gale://profile/import/")
        .ok_or_eyre("invalid url format")
        .and_then(|str| Uuid::parse_str(str).context("invalid UUID"))?;

    let import_data = profile::import::read_code(key, &app).await?;

    app.emit("import_profile", ImportProfilePayload::Normal(import_data))?;

    Ok(())
}

async fn clone_sync_profile(url: String, app: AppHandle) -> Result<()> {
    let id = url
        .strip_prefix("gale://profile/sync/clone/")
        .ok_or_eyre("invalid url format")?;

    let import_data = profile::sync::read_profile(id, &app).await?;

    app.emit("import_profile", ImportProfilePayload::Sync(import_data))?;

    Ok(())
}
