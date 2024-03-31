// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod prefs;
mod manager;
mod thunderstore;
mod util;
mod fs_util;

use anyhow::Context;
use tauri::Manager;

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

#[tauri::command]
fn open(url: String) -> util::CommandResult<()> {
    open::that(&url).with_context(|| format!("failed to open {}", url))?;
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            open,

            thunderstore::commands::query_all_mods,
            
            prefs::commands::get_pref,
            prefs::commands::set_pref,

            manager::commands::query_mods_in_profile,
            manager::commands::get_profile_info,
            manager::commands::set_active_profile,
            manager::commands::get_download_size,
            manager::commands::create_profile,
            manager::commands::delete_profile,
            manager::commands::remove_mod,
            manager::commands::reveal_project_dir,
            manager::commands::start_game,
            
            manager::downloader::commands::install_mod,
            manager::downloader::commands::clear_download_cache,

            manager::importer::commands::export_pack,

            manager::config::commands::get_config_files,
            manager::config::commands::set_config_entry,
            manager::config::commands::reset_config_entry,
        ])
        .setup(|app| {
            app.manage(NetworkClient::create()?);
            app.manage(thunderstore::ThunderstoreState::new());
            app.manage(thunderstore::query::QueryState::new());

            let handle = app.handle();
            tauri::async_runtime::spawn(
                async move {
                    if let Err(e) = thunderstore::load_mods(handle).await {
                        eprintln!("Error while loading mods: {:?}", e);
                    }
                } 
            );

            let handle = app.handle();
            tauri::async_runtime::spawn(thunderstore::query::query_loop(handle.clone()));

            let config = prefs::PrefsState::init(&handle)?;
            app.manage(manager::ModManager::init(&config.lock())?);
            app.manage(config);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
