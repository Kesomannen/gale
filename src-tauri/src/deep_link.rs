use std::{
    fmt::Debug,
    future::Future,
    path::{Path, PathBuf},
};

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DeepLinkKind {
    R2Install,
    AuthCallback,
    ImportProfile,
    CloneSyncProfile,
    ProfileFile,
}

#[derive(Clone, Debug)]
enum DeepLinkInput {
    Text(String),
    Url(Url),
}

impl DeepLinkInput {
    fn as_str(&self) -> &str {
        match self {
            Self::Text(url) => url,
            Self::Url(url) => url.as_str(),
        }
    }

    fn into_string(self) -> String {
        match self {
            Self::Text(url) => url,
            Self::Url(url) => url.to_string(),
        }
    }

    fn profile_file_path(&self) -> Option<PathBuf> {
        match self {
            Self::Text(url) => profile_file_path_from_str(url),
            Self::Url(url) => profile_file_path_from_url(url),
        }
    }
}

fn is_profile_file_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("r2z"))
}

fn looks_like_windows_absolute_path(path: &str) -> bool {
    let bytes = path.as_bytes();

    bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && matches!(bytes[2], b'\\' | b'/')
}

fn profile_file_path_from_url(url: &Url) -> Option<PathBuf> {
    if url.scheme() != "file" {
        return None;
    }

    url.to_file_path()
        .ok()
        .filter(|path| is_profile_file_path(path))
}

fn profile_file_path_from_str(url: &str) -> Option<PathBuf> {
    let path = PathBuf::from(url);

    if looks_like_windows_absolute_path(url) {
        return is_profile_file_path(&path).then_some(path);
    }

    if let Ok(url) = Url::parse(url) {
        return profile_file_path_from_url(&url);
    }

    is_profile_file_path(&path).then_some(path)
}

fn classify_deep_link(input: &DeepLinkInput) -> Option<DeepLinkKind> {
    let url = input.as_str();

    if url.starts_with("ror2mm://") {
        Some(DeepLinkKind::R2Install)
    } else if url.starts_with("gale://auth/callback") {
        Some(DeepLinkKind::AuthCallback)
    } else if url.starts_with("gale://profile/import") {
        Some(DeepLinkKind::ImportProfile)
    } else if url.starts_with("gale://profile/sync/clone") {
        Some(DeepLinkKind::CloneSyncProfile)
    } else if input.profile_file_path().is_some() {
        Some(DeepLinkKind::ProfileFile)
    } else {
        None
    }
}

#[cfg(test)]
fn classify_url(url: &str) -> Option<DeepLinkKind> {
    classify_deep_link(&DeepLinkInput::Text(url.to_owned()))
}

pub fn handle(app: &AppHandle, args: Vec<String>) -> bool {
    let Some(url) = args.into_iter().nth(1) else {
        debug!("deep link has too few arguments");
        return false;
    };

    handle_url(app, url)
}

pub fn handle_tauri_urls<'a, I>(app: &AppHandle, urls: I) -> bool
where
    I: IntoIterator<Item = &'a Url>,
{
    let mut handled = false;
    for url in urls {
        handled |= handle_deep_link(app, DeepLinkInput::Url(url.to_owned()));
    }
    handled
}

fn handle_url(app: &AppHandle, url: String) -> bool {
    handle_deep_link(app, DeepLinkInput::Text(url))
}

fn handle_deep_link(app: &AppHandle, input: DeepLinkInput) -> bool {
    let Some(kind) = classify_deep_link(&input) else {
        warn!("unsupported deep link protocol: {}", input.as_str());
        return false;
    };

    app.get_webview_window("main")
        .expect("app should have main window")
        .set_focus()
        .ok();

    let app = app.to_owned();

    match kind {
        DeepLinkKind::R2Install => {
            let url = input.into_string();
            let task_app = app.clone();
            handle_inner_task(app, handle_r2_install(url, task_app));
        }
        DeepLinkKind::AuthCallback => {
            let url = input.into_string();
            let task_app = app.clone();
            handle_inner_task(app, async move {
                profile::sync::auth::handle_callback(url, &task_app).await
            });
        }
        DeepLinkKind::ImportProfile => {
            let url = input.into_string();
            let task_app = app.clone();
            handle_inner_task(app, import_profile_code(url, task_app));
        }
        DeepLinkKind::CloneSyncProfile => {
            let url = input.into_string();
            let task_app = app.clone();
            handle_inner_task(app, clone_sync_profile(url, task_app));
        }
        DeepLinkKind::ProfileFile => {
            let path = input
                .profile_file_path()
                .expect("classified profile file should resolve to a path");
            let task_app = app.clone();
            handle_inner_task(app, async move { import_profile_file(path, &task_app) });
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

    #[test]
    fn only_classifies_real_r2z_profile_files() {
        assert_eq!(classify_url("Default.r2z"), Some(DeepLinkKind::ProfileFile));
        assert_eq!(classify_url("Default.R2Z"), Some(DeepLinkKind::ProfileFile));
        assert_eq!(classify_url("Default.notr2z"), None);
        assert_eq!(classify_url("https://example.com/Default.r2z"), None);
        assert_eq!(classify_url("https://example.com/somethingr2z"), None);
    }

    #[test]
    fn converts_file_url_profile_links_to_filesystem_paths() {
        let path = std::env::temp_dir().join("Default.r2z");
        let url = tauri::Url::from_file_path(&path).expect("temp path should convert to file URL");

        assert_eq!(classify_url(url.as_str()), Some(DeepLinkKind::ProfileFile));
        assert_eq!(profile_file_path_from_str(url.as_str()), Some(path));
    }
}
