use std::{fmt::Debug, future::Future, path::PathBuf};

use eyre::{Context, OptionExt, Result};
use tauri::{AppHandle, Emitter, Manager};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::{
    logger,
    profile::{self, import::commands::FrontendImportData},
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
    } else if url.starts_with("gale://install/") {
        handle_inner_task(app.clone(), handle_gale_install(url, app));
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

struct InstallPackage<'a> {
    owner: &'a str,
    name: &'a str,
    version: &'a str,
}

impl<'a> InstallPackage<'a> {
    fn split(string: &'a str) -> Option<Self> {
        let mut split = string.split('/');
        let (owner, name, version) = (split.next()?, split.next()?, split.next()?);

        Some(Self { owner, name, version })
    }
}

async fn handle_r2_install(url: String, app: AppHandle) -> Result<()> {
    let package = url
        .strip_prefix("ror2mm://v1/install/thunderstore.io/")
        .and_then(InstallPackage::split)
        .ok_or_eyre("invalid package url")?;

    handle_install(package, app).await
}

async fn handle_gale_install(url: String, app: AppHandle) -> Result<()> {
    let package = url
        .strip_prefix("gale://install/hexium/")
        .or_else(|| url.strip_prefix("gale://install/thunderstore/"))
        .and_then(InstallPackage::split)
        .ok_or_eyre("invalid package url")?;

    handle_install(package, app).await
}

async fn handle_install(package: InstallPackage<'_>, app: AppHandle) -> Result<()> {
    thunderstore::wait_for_fetch(&app).await;

    let thunderstore = app.lock_thunderstore();
    let borrowed_mod = thunderstore.find_mod(package.owner, package.name, package.version)?;
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

    app.emit("import_profile", FrontendImportData::new(import_data, app))?;

    Ok(())
}

async fn import_profile_code(url: String, app: AppHandle) -> Result<()> {
    let key = url
        .strip_prefix("gale://profile/import/")
        .ok_or_eyre("invalid url format")
        .and_then(|str| Uuid::parse_str(str).context("invalid UUID"))?;

    let import_data = profile::import::read_code(key, &app).await?;

    app.emit("import_profile", FrontendImportData::new(import_data, &app))?;

    Ok(())
}

async fn clone_sync_profile(url: String, app: AppHandle) -> Result<()> {
    let id = url
        .strip_prefix("gale://profile/sync/clone/")
        .ok_or_eyre("invalid url format")?;

    let import_data = profile::sync::read_profile(id, &app).await?;

    app.emit("import_profile", import_data)?;

    Ok(())
}
