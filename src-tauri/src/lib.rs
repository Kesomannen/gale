use std::env;

use itertools::Itertools;
use state::ManagerExt;
use tauri::{App, AppHandle, RunEvent};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_dialog::DialogExt;
use tracing::{error, info, warn};

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
    info!(
        "gale v{} running on {}",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
    );

    if let Err(err) = state::setup(app.handle()) {
        error!("setup error: {:?}", err);

        app.dialog()
            .message(format!("Failed to launch Gale: {err:?}"))
            .blocking_show();

        return Err(err.into());
    }

    if let Err(err) = app.deep_link().register("ror2mm") {
        warn!("failed to register ror2mm deep link protocol: {:#}", err);
    }

    if let Err(err) = app.deep_link().register("gale") {
        warn!("failed to register gale deep link protocol: {:#}", err);
    }

    let args = env::args().collect_vec();
    if !args.is_empty() && !deep_link::handle(app.handle(), args.clone()) {
        cli::run(args, app.handle());
    }

    let handle = app.handle().to_owned();
    tauri::async_runtime::spawn(async move { telemetry::send_app_start_event(handle).await });

    let handle = app.handle().to_owned();
    tauri::async_runtime::spawn(async move {
        tokio::task::spawn_blocking(move || {
            handle
                .db()
                .evict_outdated_cache()
                .unwrap_or_else(|err| warn!("failed to evict outdated cache: {err:#}"))
        })
        .await
    });

    info!("setup done");

    Ok(())
}

fn event_handler(app: &AppHandle, event: RunEvent) {
    if let RunEvent::ExitRequested { api, .. } = event {
        if !app.install_queue().handle().is_processing() {
            return;
        }

        api.prevent_exit();

        tauri::async_runtime::spawn(profile::install::handle_exit(app.to_owned()));
    }
}

fn handle_single_instance(app: &AppHandle, args: Vec<String>, _cwd: String) {
    if !deep_link::handle(app, args.clone()) {
        cli::run(args, app);
    }
}

pub fn run() {
    logger::setup().unwrap_or_else(|err| {
        eprintln!("failed to set up logger: {err:#}");
    });

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            logger::open_gale_log,
            logger::log_err,
            state::is_first_run,
            thunderstore::commands::query_thunderstore,
            thunderstore::commands::stop_querying_thunderstore,
            thunderstore::commands::get_markdown,
            thunderstore::commands::set_thunderstore_token,
            thunderstore::commands::has_thunderstore_token,
            thunderstore::commands::clear_thunderstore_token,
            thunderstore::commands::trigger_mod_fetch,
            prefs::commands::get_prefs,
            prefs::commands::set_prefs,
            prefs::commands::zoom_window,
            prefs::commands::get_system_fonts,
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
            profile::commands::create_desktop_shortcut,
            profile::launch::commands::launch_game,
            profile::launch::commands::get_launch_args,
            profile::launch::commands::open_game_dir,
            profile::install::commands::install_all_mods,
            profile::install::commands::install_mod,
            profile::install::commands::cancel_all_installs,
            profile::install::commands::has_pending_installations,
            profile::install::commands::clear_download_cache,
            profile::install::commands::get_download_size,
            profile::update::commands::change_mod_version,
            profile::update::commands::update_mods,
            profile::update::commands::ignore_update,
            profile::import::commands::import_profile,
            profile::import::commands::read_profile_code,
            profile::import::commands::read_profile_file,
            profile::import::commands::read_profile_base64,
            profile::import::commands::import_local_mod,
            profile::import::commands::import_local_mod_base64,
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
            profile::sync::commands::read_sync_profile,
            profile::sync::commands::create_sync_profile,
            profile::sync::commands::disconnect_sync_profile,
            profile::sync::commands::delete_sync_profile,
            profile::sync::commands::push_sync_profile,
            profile::sync::commands::clone_sync_profile,
            profile::sync::commands::pull_sync_profile,
            profile::sync::commands::fetch_sync_profile,
            profile::sync::commands::get_owned_sync_profiles,
            profile::sync::commands::login,
            profile::sync::commands::logout,
            profile::sync::commands::get_user,
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
        // TODO .plugin(tauri_plugin_oauth::Builder)
        .plugin(tauri_plugin_single_instance::init(handle_single_instance))
        .setup(setup)
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(event_handler);
}
