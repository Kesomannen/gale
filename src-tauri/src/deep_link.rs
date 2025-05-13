use eyre::{OptionExt, Result};
use tauri::{AppHandle, Emitter, Manager};
use tracing::{debug, info, warn};

use crate::{
    logger,
    profile::{self},
    state::ManagerExt,
    thunderstore::{BorrowedMod, IntoFrontendMod, Thunderstore},
};

pub fn handle(app: &AppHandle, args: Vec<String>) -> bool {
    info!("received deep link with {} arguments", args.len());

    let Some(url) = args.into_iter().nth(1) else {
        debug!("deep link has too few arguments");
        return false;
    };

    app.get_webview_window("main")
        .expect("app should have main window")
        .set_focus()
        .ok();

    if url.starts_with("ror2mm://") {
        let thunderstore = app.lock_thunderstore();
        let borrowed_mod = match resolve_mod_url(&url, &thunderstore) {
            Ok(to_install) => to_install,
            Err(err) => {
                logger::log_webview_err("Failed to install mod from link", err, app);
                return true;
            }
        };

        let frontend_mod = borrowed_mod.into_frontend(None);
        app.emit("install_mod", frontend_mod).ok();
        true
    } else if url.starts_with("gale://auth/callback") {
        let handle = app.to_owned();
        tauri::async_runtime::spawn(async move {
            if let Err(err) = profile::sync::auth::handle_callback(url, &handle).await {
                warn!("failed to handle auth callback: {:#}", err);
            }
        });

        true
    } else if url.ends_with("r2z") {
        let import_data = match profile::import::read_file_at_path(url.into()) {
            Ok(data) => data,
            Err(err) => {
                logger::log_webview_err("Failed to import profile from file", err, app);
                return true;
            }
        };

        app.emit("import_profile", import_data).ok();
        true
    } else {
        warn!("unsupported deep link protocol: {}", url);
        false
    }
}

fn resolve_mod_url<'a>(url: &str, thunderstore: &'a Thunderstore) -> Result<BorrowedMod<'a>> {
    let (owner, name, version) = url
        .strip_prefix("ror2mm://v1/install/thunderstore.io/")
        .and_then(|path| {
            let mut split = path.split('/');

            Some((split.next()?, split.next()?, split.next()?))
        })
        .ok_or_eyre("invalid package url")?;

    thunderstore.find_mod(owner, name, version)
}
