use core::str;
use std::{
    io::Read,
    sync::LazyLock,
    time::{Duration, Instant},
};

use bytes::Bytes;
use eyre::{Context, Report, Result};
use flate2::read::GzDecoder;
use futures_util::{TryFutureExt, future};
use indexmap::IndexMap;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::{
    game::Game,
    logger,
    state::ManagerExt,
    thunderstore::{Backend, PackageListing},
};

pub async fn fetch_package_loop(game: Game, app: AppHandle) {
    let backends = app.lock_prefs().backends(game);
    future::join_all(
        backends
            .into_backend_slice()
            .iter()
            .map(|b| fetch_single_package_loop(game, app.clone(), *b)),
    )
    .await;
}

pub(super) async fn fetch_single_package_loop(game: Game, app: AppHandle, backend: Backend) {
    const FETCH_INTERVAL: Duration = Duration::from_secs(60 * 15);

    let mut is_first = true;

    loop {
        let fetch_automatically = app.lock_prefs().fetch_mods_automatically;

        // always fetch once, even if the setting is turned off
        if !fetch_automatically && !is_first {
            info!("automatic fetch cancelled by user setting");
            break;
        };

        if let Err(err) = loop_iter(game, &mut is_first, &app, backend).await {
            logger::log_webview_err(
                format!("Error while fetching packages from {backend:?}"),
                err,
                &app,
            );
        }

        tokio::time::sleep(FETCH_INTERVAL).await;
    }

    async fn loop_iter(
        game: Game,
        is_first: &mut bool,
        app: &AppHandle,
        backend: Backend,
    ) -> Result<()> {
        if app.lock_thunderstore().is_fetching {
            warn!("automatic fetch cancelled due to ongoing fetch");
            return Ok(());
        }

        let result = fetch_single_packages(game, *is_first, app, backend).await;

        let mut state = app.lock_thunderstore();
        state.is_fetching = false;

        let backend_state = state.backend_mut(backend);
        backend_state.packages_fetched |= result.is_ok();

        *is_first &= result.is_err();

        // notify frontend to show any mod updates that were just fetched
        app.lock_manager().active_profile().notify_frontend(app)?;

        result
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
) -> Vec<(Backend, Report)> {
    let backends = app.lock_prefs().backends(game);
    let result =
        future::join_all(backends.into_backend_slice().iter().map(|b| {
            fetch_single_packages(game, write_directly, app, *b).map_err(move |e| (*b, e))
        }))
        .await
        .into_iter()
        .filter_map(Result::err)
        .collect();

    let mut state = app.lock_thunderstore();
    state.is_fetching = false;

    result
}

pub(super) async fn fetch_single_packages(
    game: Game,
    write_directly: bool,
    app: &AppHandle,
    backend: Backend,
) -> Result<()> {
    let start_time = Instant::now();

    let Some(index_url) = backend.index_url(game) else {
        app.lock_thunderstore()
            .backend_mut(backend)
            .packages_fetched = true;
        return Ok(());
    };

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
            .map(|package| (package.uuid, PackageListing { backend, ..package }));

        if write_directly {
            let mut state = app.lock_thunderstore();
            let backend_state = state.backend_mut(backend);
            let prev_count = backend_state.packages.len();
            backend_state.packages.extend(packages);

            package_count += backend_state.packages.len() - prev_count;
        } else {
            package_buffer.extend(packages);

            package_count = package_buffer.len();
        };

        emit_update(package_count, app);
    }

    let mut state = app.lock_thunderstore();
    let backend_state = state.backend_mut(backend);
    backend_state.packages_fetched = true;

    if !write_directly {
        backend_state.packages = package_buffer;
    }

    debug!(
        "fetched {} {:?} packages for {} in {:?}",
        backend_state.packages.len(),
        backend,
        game.slug,
        start_time.elapsed()
    );

    app.emit("status_update", None::<String>).ok();

    return Ok(());

    fn emit_update(mods: usize, app: &AppHandle) {
        app.emit(
            "status_update",
            Some(format!("Fetching mods from Thunderstore... {mods}")),
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
        let game = app.lock_manager().active_game;
        if app.lock_thunderstore().packages_fetched(&app, game) {
            return;
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
