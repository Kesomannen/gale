use std::{fmt::Debug, future::Future, path::PathBuf};

use eyre::{Context, OptionExt, Result};
use tauri::{AppHandle, Manager, Url};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::{
    logger,
    profile::{self, import::commands::FrontendImportData},
    state::ManagerExt,
    thunderstore::{self, Backend, FrontendMod},
};

pub fn handle_urls(app: &AppHandle, urls: Vec<Url>) -> bool {
    let mut handled = false;
    for url in urls {
        handled |= handle(app, url.to_string());
    }
    handled
}

pub fn handle(app: &AppHandle, url: String) -> bool {
    debug!("handling deep link URL: {}", url);

    // Normalize URLs before dispatch so the individual handlers accept both forms.
    let url = if let Some(path) = url.strip_prefix("gale:///") {
        format!("gale://{path}")
    } else {
        url
    };
    if let Some(window) = app.get_webview_window("main") {
        window.show().ok();
        window.unminimize().ok();
        window.set_focus().ok();
    }

    let app = app.to_owned();

    if url.starts_with("ror2mm://") {
        handle_inner_task(app.clone(), handle_r2_install(url, app));
    } else if url.starts_with("gale://install/") {
        handle_inner_task(app.clone(), handle_gale_install(url, app));
    } else if url.starts_with("gale://auth/callback") {
        handle_inner_task(app.clone(), async move {
            profile::sync::auth::handle_callback(url, &app)
        });
    } else if url.starts_with("gale://profile/import") {
        handle_inner_task(app.clone(), import_profile_code(url, app));
    } else if url.starts_with("gale://profile/sync/clone") {
        handle_inner_task(app.clone(), clone_sync_profile(url, app));
    } else if url.ends_with("r2z") {
        handle_inner_task(
            app.clone(),
            async move { import_profile_file(&url, &app).await },
        );
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
    backend: Backend,
}

impl<'a> InstallPackage<'a> {
    fn parse(path: &'a str, backend: Backend) -> Option<Self> {
        let mut split = path.split('/');
        let (owner, name, version) = (split.next()?, split.next()?, split.next()?);

        Some(Self {
            owner,
            name,
            version,
            backend,
        })
    }
}

async fn handle_r2_install(url: String, app: AppHandle) -> Result<()> {
    let package = url
        .strip_prefix("ror2mm://v1/install/thunderstore.io/")
        .and_then(|path| InstallPackage::parse(path, Backend::Thunderstore))
        .ok_or_eyre("invalid package url")?;

    handle_install(package, app).await
}

async fn handle_gale_install(url: String, app: AppHandle) -> Result<()> {
    let package = url
        .strip_prefix("gale://install/")
        .and_then(|rest| {
            let (platform, package_path) = rest.split_once('/')?;
            let backend = platform.parse::<Backend>().ok()?;

            InstallPackage::parse(package_path, backend)
        })
        .ok_or_eyre("invalid package url")?;

    handle_install(package, app).await
}

async fn handle_install(package: InstallPackage<'_>, app: AppHandle) -> Result<()> {
    thunderstore::wait_for_fetch(&app).await;

    let thunderstore = app.lock_thunderstore();
    let borrowed_mod = thunderstore.find_mod(
        package.owner,
        package.name,
        package.version,
        package.backend,
    )?;

    app.emit_buffered("install_mod", &FrontendMod::from(borrowed_mod));

    Ok(())
}

async fn import_profile_file(url: &str, app: &AppHandle) -> Result<()> {
    let path = profile_file_path(url)?;

    info!(
        "importing profile file from deep link at {}",
        path.display()
    );

    thunderstore::wait_for_fetch(app).await;

    let import_data = profile::import::read_file_at_path(path, &app.lock_thunderstore())?;

    app.emit_buffered("import_profile", &FrontendImportData::new(import_data, app));

    Ok(())
}

async fn import_profile_code(url: String, app: AppHandle) -> Result<()> {
    let key = url
        .strip_prefix("gale://profile/import/")
        .ok_or_eyre("invalid url format")
        .and_then(|str| Uuid::parse_str(str).context("invalid UUID"))?;

    thunderstore::wait_for_fetch(&app).await;

    let import_data = profile::import::read_code(key, &app).await?;

    app.emit_buffered(
        "import_profile",
        &FrontendImportData::new(import_data, &app),
    );

    Ok(())
}

async fn clone_sync_profile(url: String, app: AppHandle) -> Result<()> {
    let id = url
        .strip_prefix("gale://profile/sync/clone/")
        .ok_or_eyre("invalid url format")?;

    let import_data = profile::sync::read_profile(id, &app).await?;

    app.emit_buffered("import_profile", &import_data);

    Ok(())
}

fn profile_file_path(url: &str) -> Result<PathBuf> {
    if url.starts_with("file:") {
        Url::parse(url)?
            .to_file_path()
            .map_err(|_| eyre::eyre!("invalid profile file URL"))
    } else {
        Ok(PathBuf::from(url))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(unix)]
    fn profile_file_urls_decode_spaces_and_unicode() {
        assert_eq!(
            profile_file_path("file:///tmp/My%20Profile%20%C3%A9.r2z").unwrap(),
            PathBuf::from("/tmp/My Profile é.r2z")
        );
    }

    #[test]
    fn profile_file_paths_accept_cli_paths() {
        assert_eq!(
            profile_file_path("/tmp/My Profile.r2z").unwrap(),
            PathBuf::from("/tmp/My Profile.r2z")
        );
    }
}
