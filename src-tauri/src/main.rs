// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Context;
use tauri::Manager;

#[macro_use]
extern crate lazy_static;

#[cfg(target_os = "linux")]
extern crate webkit2gtk;

#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;

mod command_util;
mod games;
mod manager;
mod prefs;
mod thunderstore;
mod config;
mod log;
mod fs_util;
mod util;

#[derive(Debug)]
pub struct NetworkClient(reqwest::Client);

impl NetworkClient {
    fn create() -> Result<Self, reqwest::Error> {
        let client = reqwest::Client::builder()
            .http1_only()
            .user_agent("Kesomannen-gale")
            .build()?;

        Ok(Self(client))
    }
}

fn zoom_window(window: &tauri::Window, scale_factor: f64) -> tauri::Result<()> {
    window.with_webview(move |webview| {
        #[cfg(target_os = "linux")]
        {
            use webkit2gtk::traits::WebViewExt;
            webview.inner().set_zoom_level(scale_factor);
        }

        #[cfg(windows)]
        unsafe {
            webview.controller().SetZoomFactor(scale_factor).unwrap();
        }

        #[cfg(target_os = "macos")]
        unsafe {
            let () = msg_send![webview.inner(), setPageZoom: scale_factor];
        }
    })
}

fn main() {
    if !cfg!(target_os = "linux") {
        // doesn't work on linux for some reason :/
        tauri_plugin_deep_link::prepare("com.kesomannen.modmanager");
    }

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![        
            log::open_gale_log,
            
            thunderstore::commands::query_thunderstore,
            thunderstore::commands::stop_querying_thunderstore,
            thunderstore::commands::missing_deps,

            prefs::commands::get_pref,
            prefs::commands::set_pref,
            prefs::commands::is_first_run,

            manager::commands::get_game_info,
            manager::commands::favorite_game,
            manager::commands::set_active_game,
            manager::commands::get_profile_info,
            manager::commands::set_active_profile,
            manager::commands::is_mod_installed,
            manager::commands::query_profile,
            manager::commands::create_profile,
            manager::commands::delete_profile,
            manager::commands::remove_mod,
            manager::commands::force_remove_mods,
            manager::commands::toggle_mod,
            manager::commands::force_toggle_mods,
            manager::commands::set_all_mods_state,
            manager::commands::open_profile_dir,
            manager::commands::open_plugin_dir,
            manager::commands::open_bepinex_log,

            manager::launcher::commands::launch_game,

            manager::downloader::commands::install_mod,
            manager::downloader::commands::cancel_install,
            manager::downloader::commands::clear_download_cache,
            manager::downloader::commands::get_download_size,

            manager::downloader::updater::commands::update_mod,
            manager::downloader::updater::commands::update_all,

            manager::importer::commands::import_data,
            manager::importer::commands::import_code,
            manager::importer::commands::import_file,
            manager::importer::commands::import_local_mod,
            manager::importer::commands::get_r2modman_info,
            manager::importer::commands::import_r2modman,

            manager::exporter::commands::export_code,
            manager::exporter::commands::export_file,
            manager::exporter::commands::export_pack,

            config::commands::get_config_files,
            config::commands::set_tagged_config_entry,
            config::commands::set_untagged_config_entry,
            config::commands::reset_config_entry,
            config::commands::open_config_file,
            config::commands::delete_config_file,
        ])
        .setup(|app| {
            let handle = app.handle();
            log::setup(&handle).ok();

            app.manage(NetworkClient::create()?);

            prefs::setup(&handle).context("failed to initialize preferences")?;
            manager::setup(&handle).context("failed to initialize manager")?;
            config::setup(&handle).context("failed to initialize mod config")?;
            thunderstore::setup(&handle).context("failed to initialize Thunderstore")?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
