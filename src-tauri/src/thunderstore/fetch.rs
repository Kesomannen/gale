use core::str;
use std::{
    io::Read,
    sync::LazyLock,
    time::{Duration, Instant},
};

use bytes::Bytes;
use eyre::{Context, Result};
use flate2::read::GzDecoder;
use indexmap::IndexMap;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

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
    match super::cache::get_packages(app) {
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
    let start_time = Instant::now();

    let index_url = format!(
        "https://thunderstore.io/c/{}/api/v1/package-listing-index/",
        game.slug
    );

    let bytes = app
        .http()
        .get(index_url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    let urls: Vec<String> = serde_json::from_reader(GzDecoder::new(&bytes[..]))?;

    let mut package_count = 0;
    let mut package_buffer = IndexMap::new();

    let (tx, mut rx) = mpsc::channel(urls.len());

    let handle = app.to_owned();
    tokio::spawn(async move {
        if let Err(err) = fetch_chunks(tx, urls, handle).await {
            error!("failed to request package listing chunks: {:#}", err);
        }
    });

    while let Some(chunk) = rx.recv().await {
        let mut text = String::new();
        let mut decoder = GzDecoder::new(&chunk[..]);
        decoder.read_to_string(&mut text)?;

        let packages: Vec<PackageListing> = serde_json::from_str(&text)?;

        let packages = packages
            .into_iter()
            .filter(|package| {
                !EXCLUDED_PACKAGES
                    .iter()
                    .any(|excluded| package.full_name() == *excluded)
            })
            .map(|package| (package.uuid, package));

        if write_directly {
            let mut state = app.lock_thunderstore();
            let prev_count = state.packages.len();
            state.packages.extend(packages);

            package_count += state.packages.len() - prev_count;
        } else {
            package_buffer.extend(packages);

            package_count = package_buffer.len();
        };

        emit_update(package_count, app);
    }

    let mut state = app.lock_thunderstore();

    state.packages_fetched = true;
    state.is_fetching = false;

    if !write_directly {
        state.packages = package_buffer;
    }

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

    async fn fetch_chunks(
        tx: mpsc::Sender<Bytes>,
        urls: Vec<String>,
        app: AppHandle,
    ) -> Result<()> {
        for url in urls {
            let bytes = app.http().get(url).send().await?.bytes().await?;
            tx.send(bytes)
                .await
                .context("chunk channel closed too early")?;
        }

        Ok(())
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
