// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod prefs;
mod manager;
mod thunderstore;
mod util;
mod command_util;
mod fs_util;

use anyhow::Context;
use tauri::Manager;

#[derive(Debug)]
pub struct NetworkClient(reqwest::Client);

impl NetworkClient {
    fn create() -> Result<Self, reqwest::Error> {
        let client = reqwest::Client::builder()
            .http1_only()
            .user_agent("Kesomannen-ModManager")
            .build()?;

        Ok(Self(client))
    }
}

fn main() {
    tauri_plugin_deep_link::prepare("com.kesomannen.modmanager");

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            thunderstore::commands::query_all_mods,
            
            prefs::commands::get_pref,
            prefs::commands::set_pref,

            manager::commands::query_mods_in_profile,
            manager::commands::get_profile_info,
            manager::commands::set_active_profile,
            manager::commands::is_mod_installed,
            manager::commands::create_profile,
            manager::commands::delete_profile,
            manager::commands::remove_mod,
            manager::commands::force_remove_mods,
            manager::commands::reveal_profile_dir,
            manager::commands::start_game,
            
            manager::downloader::commands::install_mod,
            manager::downloader::commands::clear_download_cache,
            manager::downloader::commands::get_download_size,

            manager::importer::commands::export_code,
            manager::importer::commands::import_code,
            manager::importer::commands::export_pack,

            manager::config::commands::get_config_files,
            manager::config::commands::set_config_entry,
            manager::config::commands::reset_config_entry,
            manager::config::commands::open_config_file,
            manager::config::commands::delete_config_file,
        ])
        .setup(|app| {
            app.manage(NetworkClient::create()?);

            let handle = app.handle();
            prefs::setup(&handle).context("failed to initialize preferences")?;
            thunderstore::setup(&handle).context("failed to initialize Thunderstore")?;
            manager::setup(&handle).context("failed to initialize manager")?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
