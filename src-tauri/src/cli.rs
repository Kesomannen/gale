use std::{path::PathBuf, sync::Mutex};

use eyre::{anyhow, Context, OptionExt, Result};
use log::error;
use serde_json::Value;
use tauri::{App, Manager};
use tauri_plugin_cli::CliExt;

use crate::{
    game::{self},
    prefs::Prefs,
    profile::{self, ModManager},
    thunderstore::Thunderstore,
};

pub fn run(app: &App) -> Result<()> {
    match app.cli().matches() {
        Ok(matches) => {
            if matches.args.is_empty() {
                return Ok(());
            }

            let manager = app.state::<Mutex<ModManager>>();
            let thunderstore = app.state::<Mutex<Thunderstore>>();
            let prefs = app.state::<Mutex<Prefs>>();

            let mut manager = manager.lock().unwrap();
            let mut thunderstore = thunderstore.lock().unwrap();
            let prefs = prefs.lock().unwrap();

            if let Some(Value::String(slug)) = matches.args.get("game").map(|arg| &arg.value) {
                let game = game::from_slug(slug).ok_or_eyre("unknown game id")?;

                manager
                    .set_active_game(game, &mut thunderstore, &prefs, app.handle().clone())
                    .context("failed to set game")?;
            }

            if let Some(Value::String(profile)) = matches.args.get("profile").map(|arg| &arg.value)
            {
                let game = manager.active_game_mut();

                let index = game.profile_index(profile).ok_or_eyre("unknown profile")?;

                game.set_active_profile(index)
                    .context("failed to set profile")?;
            }

            if let Some(Value::String(path)) = matches.args.get("install").map(|arg| &arg.value) {
                let path = PathBuf::from(path);
                let handle = app.handle().to_owned();
                tauri::async_runtime::spawn(async move {
                    if let Err(err) = profile::import::import_local_mod(path, &handle).await {
                        error!("failed to install mod from cli: {:#}", err)
                    }
                });
            }

            if let Some(Value::Bool(true)) = matches.args.get("launch").map(|arg| &arg.value) {
                manager
                    .active_game()
                    .launch(&prefs)
                    .context("failed to launch game")?;

                std::process::exit(0);
            }

            Ok(())
        }
        Err(err) => Err(anyhow!(err)),
    }
}
