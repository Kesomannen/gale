use eyre::{OptionExt, Result};
use log::{debug, warn};
use tauri::{AppHandle, Emitter, Manager};

use crate::{
    logger, profile,
    state::ManagerExt,
    thunderstore::{BorrowedMod, IntoFrontendMod, Thunderstore},
};

pub fn handle(app: &AppHandle, args: Vec<String>) {
    debug!("received deep link: {:?}", args);

    app.get_window("main")
        .expect("app should have main window")
        .set_focus()
        .ok();

    let Some(url) = args.into_iter().nth(1) else {
        warn!("deep link has too few arguments");
        return;
    };

    if url.starts_with("ror2mm://") {
        let thunderstore = app.lock_thunderstore();
        let borrowed_mod = match resolve_mod_url(&url, &thunderstore) {
            Ok(to_install) => to_install,
            Err(err) => {
                logger::log_webview_err("Failed to install mod from deep link", err, app);
                return;
            }
        };

        let frontend_mod = borrowed_mod.into_frontend(None);
        app.emit("install_mod", frontend_mod).ok();
    } else if url.ends_with("r2z") {
        let import_data = match profile::import::import_file_from_path(url.into(), app) {
            Ok(data) => data,
            Err(err) => {
                logger::log_webview_err("Failed to import profile from file", err, app);
                return;
            }
        };

        app.emit("import_profile", import_data).ok();
    } else {
        warn!("unsupported deep link protocol: {}", url);
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
