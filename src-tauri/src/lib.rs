use std::{env, time::Instant};

use itertools::Itertools;
use log::{error, info};
use tauri::{App, AppHandle};
use tauri_plugin_dialog::DialogExt;

#[cfg(target_os = "linux")]
extern crate webkit2gtk;

mod cli;
mod config;
mod db;
mod deep_link;
mod game;
mod logger;
mod prefs;
mod profile;
mod state;
mod telemetry;
mod thunderstore;
mod util;

fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();

    info!(
        "gale v{} running on {}",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
    );

    if let Err(err) = state::setup(app.handle()) {
        error!("failed to start app: {:#}", err);

        app.dialog()
            .message(format!("Failed to launch Gale: {:#}", err))
            .blocking_show();

        return Err(err.into());
    }

    cli::run_from_args(app.handle());

    let args = env::args().collect_vec();
    if args.len() > 1 {
        deep_link::handle(app.handle(), args);
    }

    let handle = app.handle().to_owned();
    tauri::async_runtime::spawn(async move { telemetry::send_app_start_event(handle).await });

    info!("setup done in {:?}", start.elapsed());

    Ok(())
}

fn handle_single_instance(app: &AppHandle, args: Vec<String>, _cwd: String) {
    if !deep_link::handle(app, args.clone()) {
        cli::run(app, args.clone());
    }
}

pub fn run() {
    logger::setup().unwrap_or_else(|err| {
        eprintln!("failed to set up logger: {:#}", err);
    });

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            logger::open_gale_log,
            logger::log_err,
            state::is_first_run,
            thunderstore::commands::query_thunderstore,
            thunderstore::commands::stop_querying_thunderstore,
            thunderstore::commands::set_thunderstore_token,
            thunderstore::commands::has_thunderstore_token,
            thunderstore::commands::clear_thunderstore_token,
            thunderstore::commands::trigger_mod_fetch,
            prefs::commands::get_prefs,
            prefs::commands::set_prefs,
            prefs::commands::zoom_window,
            profile::commands::get_game_info,
            profile::commands::favorite_game,
            profile::commands::set_active_game,
            profile::commands::get_profile_info,
            profile::commands::set_active_profile,
            profile::commands::is_mod_installed,
            profile::commands::query_profile,
            profile::commands::get_dependants,
            profile::commands::create_profile,
            profile::commands::delete_profile,
            profile::commands::rename_profile,
            profile::commands::duplicate_profile,
            profile::commands::remove_mod,
            profile::commands::force_remove_mods,
            profile::commands::toggle_mod,
            profile::commands::force_toggle_mods,
            profile::commands::set_all_mods_state,
            profile::commands::remove_disabled_mods,
            profile::commands::open_profile_dir,
            profile::commands::open_mod_dir,
            profile::commands::open_game_log,
            profile::launch::commands::launch_game,
            profile::launch::commands::get_launch_args,
            profile::launch::commands::open_game_dir,
            profile::install::commands::install_mod,
            profile::install::commands::cancel_install,
            profile::install::commands::clear_download_cache,
            profile::install::commands::get_download_size,
            profile::update::commands::change_mod_version,
            profile::update::commands::update_mods,
            profile::update::commands::ignore_update,
            profile::import::commands::import_data,
            profile::import::commands::import_code,
            profile::import::commands::import_file,
            profile::import::commands::import_base64,
            profile::import::commands::import_local_mod,
            profile::import::commands::get_r2modman_info,
            profile::import::commands::import_r2modman,
            profile::export::commands::export_code,
            profile::export::commands::export_file,
            profile::export::commands::export_pack,
            profile::export::commands::upload_pack,
            profile::export::commands::get_pack_args,
            profile::export::commands::set_pack_args,
            profile::export::commands::generate_changelog,
            profile::export::commands::copy_dependency_strings,
            profile::export::commands::copy_debug_info,
            config::commands::get_config_files,
            config::commands::set_config_entry,
            config::commands::reset_config_entry,
            config::commands::open_config_file,
            config::commands::delete_config_file,
        ])
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(handle_single_instance))
        .setup(setup)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
