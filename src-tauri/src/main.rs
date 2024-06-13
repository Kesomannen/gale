// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use ::log::error;
use anyhow::Context;
use tauri::{
    api::dialog::{blocking::MessageDialogBuilder, MessageDialogKind},
    AppHandle, Manager,
};

#[macro_use]
extern crate lazy_static;

#[cfg(target_os = "linux")]
extern crate webkit2gtk;

#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;

mod config;
mod games;
mod log;
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

fn setup(app: AppHandle) -> anyhow::Result<()> {
    app.manage(NetworkClient::create()?);

    prefs::setup(&app).context("Failed to read settings")?;
    manager::setup(&app).context("Failed to initialize mod manager")?;
    thunderstore::setup(&app);

    Ok(())
}

fn main() {
    if !cfg!(target_os = "linux") {
        // doesn't work on linux for some reason :/
        tauri_plugin_deep_link::prepare("com.kesomannen.modmanager");
    }

    let mut builder = tauri::Builder::default();
    
    if cfg!(debug_assertions) {
        builder = builder.plugin(devtools::init());
    }

    builder
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            log::open_gale_log,
            thunderstore::commands::query_thunderstore,
            thunderstore::commands::stop_querying_thunderstore,
            thunderstore::commands::get_missing_deps,
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
            manager::commands::get_dependants,
            manager::commands::create_profile,
            manager::commands::delete_profile,
            manager::commands::rename_profile,
            manager::commands::duplicate_profile,
            manager::commands::remove_mod,
            manager::commands::force_remove_mods,
            manager::commands::toggle_mod,
            manager::commands::force_toggle_mods,
            manager::commands::reorder_mod,
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
            manager::exporter::commands::export_dep_string,
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

            if let Err(err) = setup(handle) {
                error!("Could not start app! {:#}", err);

                MessageDialogBuilder::new("Error while launching Gale!", format!("{:#}", err))
                    .kind(MessageDialogKind::Error)
                    .show();

                return Err(err.into());
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
