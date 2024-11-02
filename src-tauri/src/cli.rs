use anyhow::{anyhow, Context, Result};

use crate::{games, manager::ModManager, prefs::Prefs, thunderstore::Thunderstore};
use serde_json::Value;
use std::sync::Mutex;
use tauri::{App, Manager};
use tauri_plugin_cli::CliExt;

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

            if let Some(arg) = matches.args.get("game") {
                if let Value::String(game) = &arg.value {
                    let game = games::from_slug(game).context("unknown game id")?;

                    manager
                        .set_active_game(game, &mut thunderstore, &prefs, app.handle().clone())
                        .context("failed to set game")?;
                }
            }

            if let Some(arg) = matches.args.get("profile") {
                if let Value::String(profile) = &arg.value {
                    let game = manager.active_game_mut();

                    let index = game
                        .profiles
                        .iter()
                        .position(|p| p.name == *profile)
                        .context("unknown profile")?;

                    game.set_active_profile(index, Some(&thunderstore))
                        .context("failed to set profile")?;
                }
            }

            if let Some(arg) = matches.args.get("launch") {
                if let Value::Bool(true) = &arg.value {
                    manager
                        .active_game()
                        .launch(&prefs)
                        .context("failed to launch game")?;

                    std::process::exit(0);
                }
            }

            Ok(())
        }
        Err(err) => Err(anyhow!(err)),
    }
}
