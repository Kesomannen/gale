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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DeepLinkKind {
    R2Install,
    AuthCallback,
    ImportProfile,
    CloneSyncProfile,
    ProfileFile,
}

fn classify_url(url: &str) -> Option<DeepLinkKind> {
    if url.starts_with("ror2mm://") {
        Some(DeepLinkKind::R2Install)
    } else if url.starts_with("gale://auth/callback") {
        Some(DeepLinkKind::AuthCallback)
    } else if url.starts_with("gale://profile/import") {
        Some(DeepLinkKind::ImportProfile)
    } else if url.starts_with("gale://profile/sync/clone") {
        Some(DeepLinkKind::CloneSyncProfile)
    } else if url.ends_with("r2z") {
        Some(DeepLinkKind::ProfileFile)
    } else {
        None
    }
}

pub fn handle(app: &AppHandle, args: Vec<String>) -> bool {
    let Some(url) = args.into_iter().nth(1) else {
        debug!("deep link has too few arguments");
        return false;
    };

    handle_url(app, url)
}

pub fn handle_urls<I, S>(app: &AppHandle, urls: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut handled = false;
    for url in urls {
        handled |= handle_url(app, url.as_ref().to_owned());
    }
    handled
}

fn handle_url(app: &AppHandle, url: String) -> bool {
    let Some(kind) = classify_url(&url) else {
        warn!("unsupported deep link protocol: {}", url);
        return false;
    };

    app.get_webview_window("main")
        .expect("app should have main window")
        .set_focus()
        .ok();

    let app = app.to_owned();

    match kind {
        DeepLinkKind::R2Install => {
            handle_inner_task(app.clone(), handle_r2_install(url, app));
        }
        DeepLinkKind::AuthCallback => {
            handle_inner_task(app.clone(), async move {
                profile::sync::auth::handle_callback(url, &app).await
            });
        }
        DeepLinkKind::ImportProfile => {
            handle_inner_task(app.clone(), import_profile_code(url, app));
        }
        DeepLinkKind::CloneSyncProfile => {
            handle_inner_task(app.clone(), clone_sync_profile(url, app));
        }
        DeepLinkKind::ProfileFile => {
            handle_inner_task(app.clone(), async move { import_profile_file(&url, &app) });
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_urls_delivered_by_macos_events() {
        assert_eq!(
            classify_url("ror2mm://v1/install/thunderstore.io/owner/name/version"),
            Some(DeepLinkKind::R2Install)
        );
        assert_eq!(
            classify_url("gale://auth/callback?code=example"),
            Some(DeepLinkKind::AuthCallback)
        );
        assert_eq!(
            classify_url("gale://profile/import/00000000-0000-0000-0000-000000000000"),
            Some(DeepLinkKind::ImportProfile)
        );
        assert_eq!(
            classify_url("gale://profile/sync/clone/example"),
            Some(DeepLinkKind::CloneSyncProfile)
        );
    }

    #[test]
    fn rejects_unknown_urls() {
        assert_eq!(classify_url("https://example.com"), None);
    }
}
