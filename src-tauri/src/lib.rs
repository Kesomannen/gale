use ::log::error;
use anyhow::Context;
use tauri::{AppHandle, Manager};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_dialog::DialogExt;

#[macro_use]
extern crate lazy_static;

#[cfg(target_os = "linux")]
extern crate webkit2gtk;

mod cli;
mod config;
mod games;
mod logger;
mod prefs;
mod profile;
mod thunderstore;
mod util;

#[derive(Debug)]
pub struct NetworkClient(reqwest::Client);

impl NetworkClient {
    fn create() -> Result<Self, reqwest::Error> {
        let client = reqwest::Client::builder()
            .user_agent("Kesomannen-gale")
            .build()?;

        Ok(Self(client))
    }
}

fn setup(app: &AppHandle) -> anyhow::Result<()> {
    app.manage(NetworkClient::create()?);

    prefs::setup(app).context("Failed to read settings")?;
    profile::setup(app).context("Failed to initialize mod manager")?;
    thunderstore::setup(app);

    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            logger::open_gale_log,
            logger::log_err,
            thunderstore::commands::query_thunderstore,
            thunderstore::commands::stop_querying_thunderstore,
            thunderstore::commands::get_missing_deps,
            thunderstore::commands::set_thunderstore_token,
            thunderstore::commands::has_thunderstore_token,
            thunderstore::commands::clear_thunderstore_token,
            thunderstore::commands::trigger_mod_fetch,
            prefs::commands::get_prefs,
            prefs::commands::set_prefs,
            prefs::commands::is_first_run,
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
            profile::commands::open_plugin_dir,
            profile::commands::open_bepinex_log,
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
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            app.get_window("main")
                .expect("app should have main window")
                .set_focus()
                .ok();

            let url = match args.into_iter().nth(1) {
                Some(arg) => arg,
                None => return,
            };

            if url.starts_with("ror2mm://") {
                profile::install::deep_link::handle(&url, app);
            } else if url.ends_with("r2z") {
                let app = app.to_owned();
                tauri::async_runtime::spawn(async move {
                    profile::import::import_file_from_link(url, &app)
                        .await
                        .unwrap_or_else(|err| {
                            logger::log_js_err("Failed to import profile file", &err, &app);
                        })
                });
            }
        }))
        .plugin(tauri_plugin_cli::init())
        .setup(|app| {
            let handle = app.handle().clone();
            logger::setup().ok();

            if let Err(err) = setup(&handle) {
                error!("failed to launch Gale! {:#}", err);

                app.dialog()
                    .message(format!("{:#}", err))
                    .title("Error while launching Gale!")
                    .blocking_show();

                return Err(err.into());
            }

            app.deep_link().register("ror2mm").unwrap_or_else(|err| {
                error!("failed to register deep link: {:#}", err);
            });

            cli::run(app).unwrap_or_else(|err| {
                error!("failed to run CLI: {:#}", err);
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
