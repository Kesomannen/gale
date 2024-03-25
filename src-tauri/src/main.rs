// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod prefs;
mod manager;
mod thunderstore;
mod util;
mod io_util;

use tauri::Manager;

pub struct NetworkClient {
    client: reqwest::Client,
}

impl NetworkClient {
    fn create() -> Result<Self, reqwest::Error> {
        let client = reqwest::Client::builder()
            .http1_only()
            .user_agent("Kesomannen-ModManager")
            .build()?;

        Ok(Self { client })
    }
}

#[tauri::command]
fn open(url: String) -> Result<(), String> {
    open::that(url).map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            open,

            thunderstore::commands::query_all_mods,
            thunderstore::commands::get_mod,
            thunderstore::commands::get_mod_by_id,

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
            manager::commands::clear_download_cache,

            manager::downloader::commands::install_mod,

            manager::importer::commands::export_pack
        ])
        .setup(|app| {
            app.manage(NetworkClient::create()?);
            app.manage(thunderstore::ThunderstoreState::new());

            let handle = app.handle();

            tauri::async_runtime::spawn(thunderstore::load_mods(handle.clone()));

            let config = prefs::PrefsState::init(&handle)?;
            app.manage(manager::ModManager::init(&config.lock())?);
            app.manage(config);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
