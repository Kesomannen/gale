use std::{
    collections::VecDeque,
    fs,
    future::Future,
    io::Cursor,
    iter,
    sync::{Mutex, MutexGuard},
    time::{Duration, Instant},
};

use eyre::{bail, eyre, Context, Result};
use futures_util::StreamExt;
use itertools::Itertools;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::sync::oneshot;
use tracing::warn;
use uuid::Uuid;
use zip::ZipArchive;

use crate::{
    logger,
    profile::install::{InstallOptions, ModInstall},
    state::ManagerExt,
    util::error::IoResultExt,
};

#[derive(Default)]
pub struct InstallQueue {
    pending: Mutex<VecDeque<InstallRequest>>,
    notify: tokio::sync::Notify,
}

impl InstallQueue {
    pub fn new(app: AppHandle) -> Self {
        tauri::async_runtime::spawn(handle_queue(app));

        Self::default()
    }

    pub fn handle(&self) -> Handle {
        Handle {
            pending: self.pending.lock().unwrap(),
            queue: self,
        }
    }

    pub fn push_mods(
        &self,
        mods: impl IntoIterator<Item = ModInstall>,
        profile_id: i64,
        options: InstallOptions,
        app: &AppHandle,
    ) -> impl Future<Output = Result<()>> {
        self.handle().push_mods(mods, profile_id, options, app)
    }

    pub fn push_with_deps(
        &self,
        mods: Vec<ModInstall>,
        profile_id: i64,
        options: InstallOptions,
        allow_multiple: bool,
        app: &AppHandle,
    ) -> Result<impl Future<Output = Result<()>>> {
        self.handle()
            .push_with_deps(mods, profile_id, options, allow_multiple, app)
    }
}

pub struct Handle<'a> {
    pending: MutexGuard<'a, VecDeque<InstallRequest>>,
    queue: &'a InstallQueue,
}

impl<'a> Handle<'a> {
    pub fn has_mod(&self, uuid: Uuid, profile_id: i64) -> bool {
        self.pending
            .iter()
            .any(|request| request.install.uuid() == uuid && request.profile_id == profile_id)
    }

    fn has_request(&self, request: &InstallRequest) -> bool {
        self.has_mod(request.install.uuid(), request.profile_id)
    }

    fn push_mods(
        &mut self,
        mods: impl IntoIterator<Item = ModInstall>,
        profile_id: i64,
        options: InstallOptions,
        app: &AppHandle,
    ) -> impl Future<Output = Result<()>> {
        let requests = mods.into_iter().map(|install| InstallRequest {
            install,
            options: options.clone(),
            profile_id,
            on_complete: None,
        });

        let mut pushed_count = 0usize;
        let mut pushed_bytes = 0;

        for request in requests {
            // skip mod if it's already queued
            if self.has_request(&request) {
                continue;
            }

            pushed_count += 1;
            pushed_bytes += request.install.file_size;

            self.pending.push_back(request);
        }

        let (tx, rx) = oneshot::channel();

        if pushed_count > 0 {
            self.pending.iter_mut().last().unwrap().on_complete = Some(tx);
            self.queue.notify.notify_waiters();

            emit(
                InstallEvent::AddCount {
                    mods: pushed_count,
                    bytes: pushed_bytes,
                },
                app,
            );
        } else {
            // complete the task immediately since there are no mods to install
            tx.send(Ok(())).unwrap();
        }

        async move {
            match rx.await {
                Ok(Ok(())) => Ok(()),
                Ok(Err(err)) => Err(err),
                Err(err) => Err(eyre!(err)),
            }
        }
    }

    fn push_with_deps(
        &mut self,
        mods: Vec<ModInstall>,
        profile_id: i64,
        options: InstallOptions,
        allow_multiple: bool,
        app: &AppHandle,
    ) -> Result<impl Future<Output = Result<()>>> {
        let mods = {
            let manager = app.lock_manager();
            let thunderstore = app.lock_thunderstore();
            let (_, profile) = manager.profile_by_id(profile_id)?;

            if !allow_multiple && mods.len() == 1 && profile.has_mod(mods[0].uuid()) {
                bail!("mod is already installed");
            }

            // find the missing dependencies of each mod and flatten them into one vec
            let mods = mods
                .into_iter()
                .map(|install| {
                    let borrowed = install.id.borrow(&thunderstore)?;

                    Ok(iter::once(install).chain(
                        profile
                            .missing_deps(borrowed.dependencies(), &thunderstore)
                            .map(ModInstall::from),
                    ))
                })
                .flatten_ok()
                .collect::<Result<Vec<_>>>()
                .context("failed to resolve dependencies")?;

            mods.into_iter()
                .unique_by(|install| install.uuid()) // remove duplicate dependencies
                .rev() // install dependencies first
                .collect_vec()
        };

        Ok(self.push_mods(mods, profile_id, options, app))
    }
}

pub struct InstallRequest {
    install: ModInstall,
    options: InstallOptions,
    profile_id: i64,
    on_complete: Option<oneshot::Sender<Result<()>>>,
}

impl InstallRequest {
    fn complete(self, result: Result<()>, app: &AppHandle) {
        match result {
            Ok(()) => {
                if let Some(tx) = self.on_complete {
                    tx.send(Ok(())).ok();
                }
            }
            Err(err) => {
                let err = if let Some(tx) = self.on_complete {
                    match tx.send(Err(err)) {
                        Ok(()) => return,
                        Err(err) => err.unwrap_err(),
                    }
                } else {
                    err
                };

                logger::log_webview_err(
                    format!("Failed to install {}", self.install.ident),
                    err,
                    &app,
                );
            }
        }
    }
}

async fn handle_queue(app: AppHandle) {
    let queue = app.install_queue();

    loop {
        queue.notify.notified().await;

        loop {
            emit(InstallEvent::Show, &app);

            let request = queue.handle().pending.pop_front();

            match request {
                Some(request) => handle_request(queue, request, &app).await,
                None => break,
            }
        }

        emit(InstallEvent::Hide, &app);

        app.lock_manager()
            .cache_mods(&app.lock_thunderstore(), &app.lock_prefs())
            .ok();
    }
}

async fn handle_request(queue: &InstallQueue, mut request: InstallRequest, app: &AppHandle) {
    let result = handle_request_inner(&request, app).await;

    if result.is_err() {
        // keep popping request until we find one with an on_complete sender
        let mut handle = queue.handle();
        loop {
            if request.on_complete.is_some() {
                break;
            }

            match handle.pending.pop_front() {
                Some(next) => request = next,
                None => break,
            };
        }
    }

    request.complete(result, app);
}

async fn handle_request_inner(request: &InstallRequest, app: &AppHandle) -> Result<()> {
    match try_cache_install(request, app)? {
        InstallMethod::Cached => Ok(()),
        InstallMethod::Download => {
            let bytes = download(request, app).await?;
            install_from_download(bytes, request, app)?;

            Ok(())
        }
    }
}

enum InstallMethod {
    Cached,
    Download,
}

fn try_cache_install(request: &InstallRequest, app: &AppHandle) -> Result<InstallMethod> {
    let prefs = app.lock_prefs();
    let mut manager = app.lock_manager();
    let thunderstore = app.lock_thunderstore();

    let borrow = request.install.id.borrow(&thunderstore)?;
    let cache_path = super::cache::path(&borrow.version.ident, &prefs);

    if cache_path.exists() {
        if let Some(callback) = &request.options.before_install {
            callback(&request.install, &mut manager, &thunderstore)?;
        }

        let (game, profile) = manager.profile_by_id_mut(request.profile_id)?;
        let package_name = borrow.ident().full_name();

        let mut installer = game.mod_loader.installer_for(package_name);
        installer.install(&cache_path, package_name, profile)?;

        request.install.clone().insert_into(profile)?;

        profile.save(app.db())?;

        emit(
            InstallEvent::AddProgress {
                mods: 1,
                bytes: request.install.file_size,
            },
            app,
        );

        Ok(InstallMethod::Cached)
    } else {
        Ok(InstallMethod::Download)
    }
}

async fn download(request: &InstallRequest, app: &AppHandle) -> Result<Vec<u8>> {
    let url = format!(
        "https://thunderstore.io/package/download/{}/",
        request.install.ident.path()
    );

    let mut stream = app
        .http()
        .get(url)
        .send()
        .await
        .and_then(|response| response.error_for_status())?
        .bytes_stream();

    let mut response = Vec::with_capacity(request.install.file_size as usize);

    const UPDATE_DELAY: Duration = Duration::from_millis(100);
    let mut last_update = Instant::now();
    let mut last_size_update = 0u64;

    while let Some(item) = stream.next().await {
        let item = item?;
        response.extend_from_slice(&item);

        if last_update.elapsed() >= UPDATE_DELAY {
            last_update = Instant::now();
            emit(
                InstallEvent::AddProgress {
                    mods: 0,
                    bytes: response.len() as u64 - last_size_update,
                },
                &app,
            );
            last_size_update = response.len() as u64;
        }
    }

    emit(
        InstallEvent::AddProgress {
            mods: 0,
            bytes: response.len() as u64 - last_size_update,
        },
        app,
    );

    Ok(response)
}

fn install_from_download(data: Vec<u8>, request: &InstallRequest, app: &AppHandle) -> Result<()> {
    let prefs = app.lock_prefs();
    let mut manager = app.lock_manager();
    let thunderstore = app.lock_thunderstore();

    let borrow = request.install.id.borrow(&thunderstore)?;
    let cache_path = super::cache::path(&borrow.version.ident, &prefs);
    let package_name = borrow.ident().full_name();

    let (game, _) = manager.profile_by_id_mut(request.profile_id)?;

    fs::create_dir_all(&cache_path).fs_context("creating mod cache dir", &cache_path)?;

    let mut installer = game.mod_loader.installer_for(package_name);

    let archive = ZipArchive::new(Cursor::new(data)).context("failed to open archive")?;

    installer
        .extract(archive, package_name, cache_path.clone())
        .inspect_err(|_| {
            // the cached mod is probably in an invalid state, so remove it
            fs::remove_dir_all(&cache_path).unwrap_or_else(|err| {
                warn!(
                    "failed to clean up after failed extraction of {}: {:#}",
                    request.install.ident, err
                );
            });
        })
        .context("error while extracting")?;

    if let Some(callback) = &request.options.before_install {
        callback(&request.install, &mut manager, &thunderstore)?;
    }

    let (_, profile) = manager.profile_by_id_mut(request.profile_id)?;

    installer.install(&cache_path, package_name, profile)?;
    request.install.clone().insert_into(profile)?;

    profile.save(app.db())?;

    emit(InstallEvent::AddProgress { mods: 1, bytes: 0 }, app);

    Ok(())
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase", tag = "type")]
enum InstallEvent {
    Show,
    Hide,
    #[serde(rename_all = "camelCase")]
    AddCount {
        mods: usize,
        bytes: u64,
    },
    #[serde(rename_all = "camelCase")]
    AddProgress {
        mods: usize,
        bytes: u64,
    },
}

fn emit(event: InstallEvent, app: &AppHandle) {
    app.emit("install_event", event).ok();
}
