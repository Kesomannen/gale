use std::{path::PathBuf, process};

use clap::Parser;
use eyre::{Context, OptionExt, Result};
use log::{debug, error, info};
use tauri::AppHandle;

use crate::{
    game::{self},
    profile::{self, install::InstallOptions, ModManager},
    state::ManagerExt,
};

pub fn run_from_args(app: &AppHandle) {
    Cli::parse().run(app).unwrap_or_else(|err| {
        error!("failed to run cli: {:#}", err);
    })
}

pub fn run(app: &AppHandle, args: Vec<String>) {
    Cli::parse_from(args).run(app).unwrap_or_else(|err| {
        error!("failed to run cli: {:#}", err);
    })
}

#[derive(Debug, Parser)]
#[command(version)]
struct Cli {
    #[arg(short, long, value_name = "SLUG")]
    game: Option<String>,

    #[arg(short, long, value_name = "NAME")]
    profile: Option<String>,

    #[arg(short, long, value_name = "PATH")]
    install: Option<PathBuf>,

    #[arg(short, long)]
    launch: bool,

    #[arg(long)]
    no_gui: bool,
}

impl Cli {
    fn run(self, app: &AppHandle) -> Result<()> {
        let mut manager = app.lock_manager();

        let Cli {
            game,
            profile,
            install,
            launch,
            no_gui,
        } = self;

        if let Some(slug) = &game {
            let game = game::from_slug(slug).ok_or_eyre("unknown game slug")?;

            manager
                .set_active_game(game, app)
                .context("failed to set game")?;

            debug!("set active game to {}", slug);
        }

        if let Some(name) = profile {
            let game = manager.active_game_mut();

            let index = game.profile_index(&name).ok_or_eyre("unknown profile")?;

            game.set_active_profile(index)
                .context("failed to set profile")?;

            debug!("set profile index to {}", index);
        }

        if let Some(path) = install {
            drop(manager);

            let handle = app.to_owned();
            tauri::async_runtime::spawn(async move {
                if let Err(err) = install_local_mod(path, &handle).await {
                    error!("failed to install mod from cli: {:#}", err)
                }

                let manager = handle.lock_manager();
                if let Err(err) = handle_launch_and_no_gui(launch, no_gui, &manager, &handle) {
                    error!("{:#}", err);
                }
            });
        } else {
            handle_launch_and_no_gui(launch, no_gui, &manager, app)?;
        }

        debug!("cli finished");
        return Ok(());

        fn handle_launch_and_no_gui(
            launch: bool,
            no_gui: bool,
            manager: &ModManager,
            app: &AppHandle,
        ) -> Result<()> {
            if launch {
                manager
                    .active_game()
                    .launch(&app.lock_prefs(), app)
                    .context("failed to launch game")?;
            }

            if no_gui {
                process::exit(0);
            }

            Ok(())
        }
    }
}

async fn install_local_mod(path: PathBuf, app: &AppHandle) -> Result<()> {
    profile::import::import_local_mod(
        path,
        None,
        app,
        InstallOptions::default().on_progress(Box::new(|progress, _| {
            info!(
                "{} {} ({}%)",
                progress.task,
                progress.current_name,
                (progress.total_progress * 100.0).round()
            )
        })),
    )
    .await
}
