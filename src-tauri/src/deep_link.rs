use std::{fmt::Debug, future::Future, path::PathBuf};

use eyre::{Context, OptionExt, Result};
use tauri::{AppHandle, Emitter, Manager, Url};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::{
    logger,
    profile::{self, import::commands::FrontendImportData},
    state::ManagerExt,
    thunderstore::{self, IntoFrontendMod},
};

/// A recognized deep link, parsed from a CLI argument or a macOS open event.
#[derive(Debug, Eq, PartialEq)]
enum DeepLink {
    R2Install,
    AuthCallback,
    ImportProfile,
    CloneSyncProfile,
    ProfileFile(PathBuf),
}

fn classify_url(url: &str) -> Option<DeepLink> {
    if url.starts_with("ror2mm://") {
        Some(DeepLink::R2Install)
    } else if url.starts_with("gale://auth/callback") {
        Some(DeepLink::AuthCallback)
    } else if url.starts_with("gale://profile/import") {
        Some(DeepLink::ImportProfile)
    } else if url.starts_with("gale://profile/sync/clone") {
        Some(DeepLink::CloneSyncProfile)
    } else {
        profile_file_path(url).map(DeepLink::ProfileFile)
    }
}

/// Interprets `url` as a local `.r2z` profile file, given either as a plain
/// filesystem path or a `file://` URL.
fn profile_file_path(url: &str) -> Option<PathBuf> {
    let path = if looks_like_windows_absolute_path(url) {
        // windows drive paths would otherwise parse as URLs with a one-letter scheme
        PathBuf::from(url)
    } else if let Ok(parsed) = Url::parse(url) {
        if parsed.scheme() != "file" {
            return None;
        }

        parsed.to_file_path().ok()?
    } else {
        PathBuf::from(url)
    };

    path.extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("r2z"))
        .then_some(path)
}

fn looks_like_windows_absolute_path(path: &str) -> bool {
    let bytes = path.as_bytes();

    bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && matches!(bytes[2], b'\\' | b'/')
}

pub fn handle(app: &AppHandle, args: Vec<String>) -> bool {
    let Some(url) = args.into_iter().nth(1) else {
        debug!("deep link has too few arguments");
        return false;
    };

    handle_url(app, url)
}

/// Handles deep links and file opens delivered as URL events on macOS.
#[cfg(target_os = "macos")]
pub fn handle_urls(app: &AppHandle, urls: Vec<Url>) {
    for url in urls {
        handle_url(app, url.to_string());
    }
}

fn handle_url(app: &AppHandle, url: String) -> bool {
    let Some(deep_link) = classify_url(&url) else {
        warn!("unsupported deep link protocol: {url}");
        return false;
    };

    app.get_webview_window("main")
        .expect("app should have main window")
        .set_focus()
        .ok();

    let app = app.to_owned();

    match deep_link {
        DeepLink::R2Install => handle_inner_task(app.clone(), handle_r2_install(url, app)),
        DeepLink::AuthCallback => handle_inner_task(app.clone(), async move {
            profile::sync::auth::handle_callback(url, &app).await
        }),
        DeepLink::ImportProfile => handle_inner_task(app.clone(), import_profile_code(url, app)),
        DeepLink::CloneSyncProfile => handle_inner_task(app.clone(), clone_sync_profile(url, app)),
        DeepLink::ProfileFile(path) => {
            handle_inner_task(app.clone(), async move { import_profile_file(path, &app) })
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

fn import_profile_file(path: PathBuf, app: &AppHandle) -> Result<()> {
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
    fn classifies_deep_link_urls() {
        assert_eq!(
            classify_url("ror2mm://v1/install/thunderstore.io/owner/name/version"),
            Some(DeepLink::R2Install)
        );
        assert_eq!(
            classify_url("gale://auth/callback?code=example"),
            Some(DeepLink::AuthCallback)
        );
        assert_eq!(
            classify_url("gale://profile/import/00000000-0000-0000-0000-000000000000"),
            Some(DeepLink::ImportProfile)
        );
        assert_eq!(
            classify_url("gale://profile/sync/clone/example"),
            Some(DeepLink::CloneSyncProfile)
        );
    }

    #[test]
    fn rejects_unknown_urls() {
        assert_eq!(classify_url("https://example.com"), None);
    }

    #[test]
    fn only_classifies_real_r2z_profile_files() {
        assert_eq!(
            classify_url("Default.r2z"),
            Some(DeepLink::ProfileFile(PathBuf::from("Default.r2z")))
        );
        assert_eq!(
            classify_url("Default.R2Z"),
            Some(DeepLink::ProfileFile(PathBuf::from("Default.R2Z")))
        );
        assert_eq!(classify_url("Default.notr2z"), None);
        assert_eq!(classify_url("https://example.com/Default.r2z"), None);
        assert_eq!(classify_url("https://example.com/somethingr2z"), None);
    }

    #[test]
    fn classifies_windows_absolute_paths() {
        assert_eq!(
            classify_url(r"C:\Profiles\Default.r2z"),
            Some(DeepLink::ProfileFile(PathBuf::from(
                r"C:\Profiles\Default.r2z"
            )))
        );
    }

    #[test]
    fn converts_file_url_profile_links_to_filesystem_paths() {
        let path = std::env::temp_dir().join("Default.r2z");
        let url = Url::from_file_path(&path).expect("temp path should convert to file URL");

        assert_eq!(
            classify_url(url.as_str()),
            Some(DeepLink::ProfileFile(path))
        );
    }
}
