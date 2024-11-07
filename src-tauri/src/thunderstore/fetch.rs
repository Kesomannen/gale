use core::str;
use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

use anyhow::Result;
use indexmap::IndexMap;
use log::{debug, warn};
use tauri::{AppHandle, Emitter, Manager};

use crate::{
    game::Game,
    logger,
    prefs::Prefs,
    profile::ModManager,
    thunderstore::{PackageListing, Thunderstore},
    util::cmd::StateMutex,
    NetworkClient,
};

pub(super) async fn fetch_package_loop(app: AppHandle, game: Game) {
    const FETCH_INTERVAL: Duration = Duration::from_secs(60 * 15);

    let manager = app.state::<Mutex<ModManager>>();
    let thunderstore = app.state::<Mutex<Thunderstore>>();
    let prefs = app.state::<Mutex<Prefs>>();

    read_and_insert_cache(manager, thunderstore.clone());

    let mut is_first = true;

    loop {
        let fetch_automatically = prefs.lock().unwrap().fetch_mods_automatically();

        // always fetch once, even if the setting is turned off
        if !fetch_automatically && !is_first {
            debug!("automatic fetch cancelled");
            break;
        };

        if let Err(err) = loop_iter(game, &mut is_first, &app, thunderstore.clone()).await {
            logger::log_js_err(
                "error while fetching packages from Thunderstore",
                &err,
                &app,
            );
        }

        tokio::time::sleep(FETCH_INTERVAL).await;
    }

    async fn loop_iter(
        game: Game,
        is_first: &mut bool,
        app: &AppHandle,
        thunderstore: StateMutex<'_, Thunderstore>,
    ) -> Result<()> {
        if thunderstore.lock().unwrap().is_fetching {
            return Ok(());
        }

        let result = fetch_packages(app, game, *is_first).await;

        let mut lock = thunderstore.lock().unwrap();
        lock.is_fetching = false;
        lock.packages_fetched |= result.is_ok();
        *is_first &= result.is_err();

        result
    }
}

fn read_and_insert_cache(manager: StateMutex<ModManager>, state: StateMutex<Thunderstore>) {
    let manager = manager.lock().unwrap();

    match super::read_cache(&manager) {
        Ok(Some(mods)) => {
            let mut thunderstore = state.lock().unwrap();

            for package in mods {
                thunderstore.packages.insert(package.uuid, package);
            }
        }
        Ok(None) => (),
        Err(err) => warn!("failed to read cache: {}", err),
    }
}

const EXCLUDED_PACKAGES_STR: &str = include_str!("../../excluded_packages.txt");

lazy_static! {
    static ref EXCLUDED_PACKAGES: Vec<&'static str> = EXCLUDED_PACKAGES_STR
        .split('\n')
        .map(|line| line.trim())
        .collect();
}

pub(super) async fn fetch_packages(
    app: &AppHandle,
    game: Game,
    write_directly: bool,
) -> Result<()> {
    const UPDATE_INTERVAL: Duration = Duration::from_millis(250);
    const INSERT_EVERY: usize = 1000;

    let state = app.state::<Mutex<Thunderstore>>();
    let client = &app.state::<NetworkClient>().0;

    let url = format!("https://thunderstore.io/c/{}/api/v1/package/", game.slug);
    let mut response = client.get(url).send().await?.error_for_status()?;

    let mut i = 0;
    let mut package_count = 0;

    let mut byte_buffer = Vec::new();
    let mut str_buffer = String::new();
    let mut package_buffer = IndexMap::new();

    let start_time = Instant::now();
    let mut last_update = Instant::now();

    // response is just one long JSON array
    while let Some(chunk) = response.chunk().await? {
        byte_buffer.extend_from_slice(&chunk);
        let Ok(chunk) = str::from_utf8(&byte_buffer) else {
            continue;
        };

        if i == 0 {
            str_buffer.extend(chunk.chars().skip(1)); // remove leading [
        } else {
            str_buffer.push_str(chunk);
        }

        byte_buffer.clear();

        // hacky solution to find the end of every package but what can you do
        while let Some(index) = str_buffer.find("}]},") {
            let (json, _) = str_buffer.split_at(index + 3);

            match serde_json::from_str::<PackageListing>(json) {
                Ok(package) => {
                    if !EXCLUDED_PACKAGES.contains(&package.full_name()) {
                        package_buffer.insert(package.uuid, package);
                        package_count += 1;
                    }
                }
                Err(err) => warn!("failed to deserialize package: {}", err),
            }

            str_buffer.replace_range(..index + 4, "");
        }

        // do this in bigger chunks to not have to lock the state too often
        if write_directly && package_buffer.len() >= INSERT_EVERY {
            let mut state = state.lock().unwrap();
            state.packages.extend(package_buffer.drain(..));
        }

        if last_update.elapsed() >= UPDATE_INTERVAL {
            emit_update(package_count, app);
            last_update = Instant::now();
        }

        i += 1;
    }

    let mut state = state.lock().unwrap();
    if write_directly {
        // add any remaining packages
        state.packages.extend(package_buffer.into_iter());
    } else {
        // remove all packages and replace them with the new ones
        state.packages = package_buffer;
    }

    state.packages_fetched = true;
    state.is_fetching = false;

    debug!(
        "loaded {} packages in {:?}",
        state.packages.len(),
        start_time.elapsed()
    );

    app.emit("status_update", None::<String>).ok();

    return Ok(());

    fn emit_update(mods: usize, app: &AppHandle) {
        app.emit(
            "status_update",
            Some(format!("Fetching mods from Thunderstore... {}", mods)),
        )
        .ok();
    }
}
