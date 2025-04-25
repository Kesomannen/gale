use core::str;
use std::{
    sync::LazyLock,
    time::{Duration, Instant},
};

use eyre::Result;
use indexmap::IndexMap;
use tauri::{AppHandle, Emitter};
use tracing::{debug, info, warn};

use crate::{game::Game, logger, state::ManagerExt, thunderstore::PackageListing};

pub(super) async fn fetch_package_loop(game: Game, app: AppHandle) {
    const FETCH_INTERVAL: Duration = Duration::from_secs(60 * 15);

    read_and_insert_cache(&app);

    let mut is_first = true;

    loop {
        let fetch_automatically = app.lock_prefs().fetch_mods_automatically;

        // always fetch once, even if the setting is turned off
        if !fetch_automatically && !is_first {
            info!("automatic fetch cancelled by user setting");
            break;
        };

        if let Err(err) = loop_iter(game, &mut is_first, &app).await {
            logger::log_webview_err("Error while fetching packages from Thunderstore", err, &app);
        }

        tokio::time::sleep(FETCH_INTERVAL).await;
    }

    async fn loop_iter(game: Game, is_first: &mut bool, app: &AppHandle) -> Result<()> {
        if app.lock_thunderstore().is_fetching {
            warn!("automatic fetch cancelled due to ongoing fetch");
            return Ok(());
        }

        let result = fetch_packages(game, *is_first, app).await;

        let mut state = app.lock_thunderstore();

        state.is_fetching = false;
        state.packages_fetched |= result.is_ok();

        *is_first &= result.is_err();

        result
    }
}

fn read_and_insert_cache(app: &AppHandle) {
    match super::read_cache(&app.lock_manager()) {
        Ok(Some(mods)) => {
            let mut thunderstore = app.lock_thunderstore();

            for package in mods {
                thunderstore.packages.insert(package.uuid, package);
            }
        }
        Ok(None) => (),
        Err(err) => warn!("failed to read cache: {}", err),
    }
}

const EXCLUDED_PACKAGES_STR: &str = include_str!("../../excluded_packages.txt");

static EXCLUDED_PACKAGES: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    EXCLUDED_PACKAGES_STR
        .split('\n')
        .map(|line| line.trim())
        .collect()
});

pub(super) async fn fetch_packages(
    game: Game,
    write_directly: bool,
    app: &AppHandle,
) -> Result<()> {
    const UPDATE_INTERVAL: Duration = Duration::from_millis(250);
    const INSERT_EVERY: usize = 1000;

    debug!(
        write_directly,
        game = game.slug.to_string(),
        "fetching packages"
    );

    let url = format!("https://thunderstore.io/c/{}/api/v1/package/", game.slug);
    let mut response = app.http().get(url).send().await?.error_for_status()?;

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
            let mut state = app.lock_thunderstore();
            state.packages.extend(package_buffer.drain(..));
        }

        if last_update.elapsed() >= UPDATE_INTERVAL {
            emit_update(package_count, app);
            last_update = Instant::now();
        }

        i += 1;
    }

    let mut state = app.lock_thunderstore();
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
        "fetched {} packages for {} in {:?}",
        state.packages.len(),
        game.slug,
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

pub async fn wait_for_fetch(app: &AppHandle) {
    loop {
        if app.lock_thunderstore().packages_fetched() {
            return;
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
