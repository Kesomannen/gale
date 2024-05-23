// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Context;
use tauri::{AppHandle, Manager};
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};
use log::LevelFilter;
use std::fs::{self, File};

#[macro_use]
extern crate lazy_static;

#[cfg(target_os = "linux")]
extern crate webkit2gtk;

#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;

mod command_util;
mod fs_util;
mod games;
mod manager;
mod prefs;
mod thunderstore;
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

fn logger_setup(app: &AppHandle) -> anyhow::Result<()> {
    let log_dir = app.path_resolver()
        .app_log_dir()
        .context("failed to resolve log directory")?;

    fs::create_dir_all(&log_dir)
        .context("failed to create log directory")?;

    let log_file = File::create(log_dir.join("log.log"))
        .context("failed to create log file")?;

    let term_filter = match cfg!(debug_assertions) {
        true => LevelFilter::Debug,
        false => LevelFilter::Info,
    };

    CombinedLogger::init(
        vec![
            TermLogger::new(term_filter, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
            WriteLogger::new(LevelFilter::Info, Config::default(), log_file),
        ]
    )?;

    Ok(())
}

fn main() {
    if !cfg!(target_os = "linux") {
        // doesn't work on linux for some reason :/
        tauri_plugin_deep_link::prepare("com.kesomannen.modmanager");
    }

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![            
            thunderstore::commands::query_all_mods,

            prefs::commands::get_pref,
            prefs::commands::set_pref,

            manager::commands::get_game_info,
            manager::commands::favorite_game,
            manager::commands::set_active_game,
            manager::commands::get_profile_info,
            manager::commands::set_active_profile,
            manager::commands::query_mods_in_profile,
            manager::commands::is_mod_installed,
            manager::commands::create_profile,
            manager::commands::delete_profile,
            manager::commands::remove_mod,
            manager::commands::force_remove_mods,
            manager::commands::toggle_mod,
            manager::commands::force_toggle_mods,
            manager::commands::reveal_profile_dir,
            manager::commands::open_logs,

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

            manager::exporter::commands::export_code,
            manager::exporter::commands::export_file,
            manager::exporter::commands::export_pack,

            manager::config::commands::get_config_files,
            manager::config::commands::set_tagged_config_entry,
            manager::config::commands::set_untagged_config_entry,
            manager::config::commands::reset_config_entry,
            manager::config::commands::open_config_file,
            manager::config::commands::delete_config_file,
        ])
        .setup(|app| {
            let handle = app.handle();
            logger_setup(&handle).context("failed to initialize logger")?;

            app.manage(NetworkClient::create()?);

            prefs::setup(&handle).context("failed to initialize preferences")?;
            manager::setup(&handle).context("failed to initialize manager")?;
            thunderstore::setup(&handle).context("failed to initialize Thunderstore")?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
