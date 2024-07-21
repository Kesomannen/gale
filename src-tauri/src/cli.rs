use anyhow::{anyhow, Context, Result};

use crate::{games, manager::ModManager, prefs::Prefs, thunderstore::Thunderstore};
use serde_json::Value;
use std::sync::Mutex;
use tauri::{api::cli::ArgData, App, Manager};

pub fn run(app: &App) -> Result<()> {
    match app.get_cli_matches() {
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

            if let Some(ArgData {
                value: Value::String(game),
                ..
            }) = matches.args.get("game")
            {
                let game = games::from_id(game).context("unknown game id")?;
                manager
                    .set_active_game(game, &mut thunderstore, &prefs, app.handle())
                    .context("failed to set game")?;
            }

            if let Some(ArgData {
                value: Value::String(profile),
                ..
            }) = matches.args.get("profile")
            {
                let game = manager.active_game_mut();
                let index = game
                    .profiles
                    .iter()
                    .position(|p| p.name == *profile)
                    .context("unknown profile")?;

                game.set_active_profile(index, Some(&thunderstore))
                    .context("failed to set profile")?;
            }

            if let Some(ArgData {
                value: Value::Bool(true),
                ..
            }) = matches.args.get("launch")
            {
                manager.active_game().launch(false, &prefs)
                    .context("failed to launch game")?;

                std::process::exit(0);
            }

            Ok(())
        }
        Err(err) => Err(anyhow!(err)),
    }
}
