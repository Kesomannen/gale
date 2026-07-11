use std::{
    collections::VecDeque,
    fs,
    future::Future,
    io::Cursor,
    iter,
    sync::{
        Mutex, MutexGuard,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use eyre::{Context, Result, bail, eyre};
use futures_util::StreamExt;
use itertools::Itertools;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::sync::{Notify, futures::Notified, oneshot};
use tracing::{debug, info, warn};
use uuid::Uuid;
use zip::ZipArchive;

use crate::{logger, state::ManagerExt, thunderstore::VersionIdent, util::error::IoResultExt};

use super::{CancelBehavior, InstallError, InstallOptions, InstallResult, ModInstall};

pub struct InstallQueue {
    state: Mutex<State>,
    /// Notified when a batch is pushed to the queue.
    notify_push: Notify,
    /// Notified when all batches have been completed.
    notify_empty: Notify,
    cancel: AtomicBool,
}

#[derive(Default)]
struct State {
    pending: VecDeque<InstallBatch>,
    /// The profile id and package uuids of the currently installing batch.
    processing: Option<(i64, Vec<Uuid>)>,
}

impl InstallQueue {
    pub fn new(app: AppHandle) -> Self {
        let cancel = AtomicBool::new(false);

        tauri::async_runtime::spawn(handle_queue(app));

        Self {
            state: Mutex::new(State::default()),
            notify_push: Notify::new(),
            notify_empty: Notify::new(),
            cancel,
        }
    }

    pub fn wait_for_empty(&'_ self) -> Notified<'_> {
        self.notify_empty.notified()
    }

    pub fn cancel_all(&self) {
        self.cancel.store(true, Ordering::SeqCst);
    }

    pub fn lock(&'_ self) -> InstallQueueLock<'_> {
        InstallQueueLock {
            state: self.state.lock().unwrap(),
            queue: self,
        }
    }

    /// Queue mods to be installed. Returns a future that resolves when the batch is completed,
    /// errors out, or is cancelled.
    ///
    /// This checks for mods already in the queue and skips them.
    pub fn install(
        &self,
        mods: impl IntoIterator<Item = ModInstall>,
        profile_id: i64,
        options: InstallOptions,
        app: &AppHandle,
    ) -> impl Future<Output = InstallResult<()>> {
        self.lock().push_batch(mods, profile_id, options, app)
    }

    /// Queue mods to be installed, along with their dependencies. Returns a future that resolves
    /// when the batch is completed, errors out, or is cancelled.
    ///
    /// This checks for mods already in the queue *and* already installed ones and skips them.
    ///
    /// If `allow_multiple` is `false`, it also checks if "root" mod(s) are already installed
    /// and errors out of that's the case.
    pub fn install_with_deps(
        &self,
        mods: Vec<ModInstall>,
        profile_id: i64,
        options: InstallOptions,
        allow_multiple: bool,
        app: &AppHandle,
    ) -> Result<impl Future<Output = InstallResult<()>>> {
        self.lock()
            .push_with_deps(mods, profile_id, options, allow_multiple, app)
    }
}

/// A RAII guard for the inner state of the [`InstallQueue`].
pub struct InstallQueueLock<'a> {
    state: MutexGuard<'a, State>,
    queue: &'a InstallQueue,
}

impl<'a> InstallQueueLock<'a> {
    /// Whether any mods are currently being installed.
    pub fn is_processing(&self) -> bool {
        self.state.processing.is_some()
    }

    /// Whether any mods are queued for this installation on this profile.
    pub fn has_any_for_profile(&self, profile_id: i64) -> bool {
        self.state
            .processing
            .as_ref()
            .is_some_and(|(other_profile_id, _)| *other_profile_id == profile_id)
            || self
                .state
                .pending
                .iter()
                .any(|batch| batch.profile_id == profile_id)
    }

    pub fn has_mod(&self, uuid: Uuid, profile_id: i64) -> bool {
        self.state
            .processing
            .as_ref()
            .is_some_and(|(other_profile_id, other_uuids)| {
                *other_profile_id == profile_id && other_uuids.contains(&uuid)
            })
            || self.state.pending.iter().any(|batch| {
                batch.profile_id == profile_id
                    && batch.mods.iter().any(|install| install.uuid() == uuid)
            })
    }

    fn push_batch<I>(
        &mut self,
        mods: I,
        profile_id: i64,
        options: InstallOptions,
        app: &AppHandle,
    ) -> impl Future<Output = InstallResult<()>> + use<I>
    where
        I: IntoIterator<Item = ModInstall>,
    {
        let (tx, rx) = oneshot::channel();

        let mods = mods
            .into_iter()
            .filter(|install| !self.has_mod(install.uuid(), profile_id))
            .collect_vec();

        let mod_count = mods.len();
        let bytes = mods.iter().map(|install| install.file_size as i64).sum();

        let batch = InstallBatch {
            mods,
            options,
            profile_id,
            on_complete: tx,
        };

        if mod_count > 0 {
            info!(len = mod_count, options = ?batch.options, "pushing batch to install queue");
            self.state.pending.push_back(batch);
            self.queue.notify_push.notify_waiters();

            emit(
                InstallEvent::AddCount {
                    mods: mod_count,
                    bytes,
                },
                app,
            );
        } else {
            info!(options = ?batch.options, "no mods to install, completing batch immediately");
            // complete the task immediately since there are no mods to install
            batch.complete(Ok(()), app);
        }

        async move {
            match rx.await {
                Ok(Ok(())) => Ok(()),
                Ok(Err(err)) => Err(err),
                Err(err) => Err(InstallError::Error(eyre!(err))),
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
    ) -> Result<impl Future<Output = InstallResult<()>> + use<>> {
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

        Ok(self.push_batch(mods, profile_id, options, app))
    }

    fn pop_next(&mut self) -> Option<InstallBatch> {
        let next = self.state.pending.pop_front();
        self.state.processing = next.as_ref().map(|batch| {
            (
                batch.profile_id,
                batch.mods.iter().map(|install| install.uuid()).collect(),
            )
        });
        next
    }
}

pub struct InstallBatch {
    mods: Vec<ModInstall>,
    options: InstallOptions,
    profile_id: i64,
    on_complete: oneshot::Sender<InstallResult<()>>,
}

impl InstallBatch {
    fn complete(self, result: InstallResult<()>, app: &AppHandle) {
        match self.on_complete.send(result) {
            Ok(_) => (),
            // The receiver was dropped (meaning the original caller doesn't care anymore), however
            // if we succeeded or was cancelled we don't care either ...
            Err(Ok(_)) => (),
            Err(Err(InstallError::Cancelled)) => (),
            Err(Err(InstallError::Error(err))) => {
                // ... but on errors we want to log the error and notify the frontend.
                logger::log_webview_err("Failed to install batch", err, app);
            }
        }
    }
}

async fn handle_queue(app: AppHandle) {
    let queue = app.install_queue();

    // continously wait for new batches to be pushed and process them
    loop {
        queue.notify_push.notified().await;
        queue.cancel.store(false, Ordering::SeqCst);

        debug!("started processing batches");

        let mut reason = HideReason::Done;

        emit(InstallEvent::Show, &app);

        loop {
            let batch = queue.lock().pop_next();

            match batch {
                Some(batch) => {
                    reason = handle_batch(batch, &queue.cancel, &app).await;
                }
                None => break,
            }
        }

        debug!(?reason, "finished processing batches");

        emit(InstallEvent::Hide { reason }, &app);

        app.lock_manager()
            .cache_mods(&app.lock_thunderstore(), &app.lock_prefs())
            .ok();

        queue.notify_empty.notify_waiters();
    }
}

async fn handle_batch(batch: InstallBatch, cancel: &AtomicBool, app: &AppHandle) -> HideReason {
    let mut result = Ok(());
    let mut reason = HideReason::Done;

    for (i, install) in batch.mods.iter().enumerate() {
        result = handle_install(&batch, i, cancel, app).await;

        match result {
            Ok(()) => (),
            Err(InstallError::Cancelled) => {
                rollback_batch(&batch, app, i).unwrap_or_else(|err| {
                    warn!("failed to rollback cancelled installation: {}", err)
                });

                // cancel all pending bathes
                let mut handle = app.install_queue().lock();
                for batch in handle.state.pending.drain(..) {
                    batch.complete(Err(InstallError::Cancelled), app);
                }

                reason = HideReason::Cancelled;
                break;
            }
            Err(InstallError::Error(err)) => {
                rollback_batch(&batch, app, i)
                    .unwrap_or_else(|err| warn!(?err, "failed to rollback failed installation",));

                result = Err(InstallError::Error(
                    err.wrap_err(format!("failed to install {}", install.ident)),
                ));

                reason = HideReason::Error;
                break;
            }
        }
    }

    info!(len = batch.mods.len(), "completed batch");

    batch.complete(result, app);
    reason
}

fn rollback_batch(batch: &InstallBatch, app: &AppHandle, count: usize) -> Result<()> {
    match batch.options.cancel_behavior {
        CancelBehavior::Individual => Ok(()),
        CancelBehavior::Prevent => {
            warn!("tried to roll back batch with CancelBehavior::Prevent!");

            Ok(())
        }
        CancelBehavior::Batch => {
            info!(
                batch_len = batch.mods.len(),
                rollback_count = count,
                "rolling back batch"
            );

            let mut manager = app.lock_manager();

            let (_, profile) = manager.profile_by_id_mut(batch.profile_id)?;

            for installed in batch.mods.iter().take(count) {
                profile
                    .force_remove_mod(installed.uuid())
                    .unwrap_or_else(|err| {
                        warn!(
                            ident = %installed.ident,
                            ?err, "failed to rollback installed mod");
                    });
            }

            profile.save(app, true)?;
            Ok(())
        }
    }
}

async fn handle_install(
    batch: &InstallBatch,
    index: usize,
    cancel: &AtomicBool,
    app: &AppHandle,
) -> InstallResult<()> {
    match try_cache_install(batch, index, app)? {
        CacheStatus::Hit => Ok(()),
        CacheStatus::Miss => {
            let bytes = download(&batch.mods[index], cancel, &batch.options, app)
                .await
                .context("error while downloading")?;
            install_from_download(bytes, batch, index, cancel, app)
                .context("error while installing")?;

            Ok(())
        }
    }
}

enum CacheStatus {
    Hit,
    Miss,
}

fn try_cache_install(batch: &InstallBatch, index: usize, app: &AppHandle) -> Result<CacheStatus> {
    let install = &batch.mods[index];

    let cache_path = super::cache::path(&install.ident, &app.lock_prefs());

    if !cache_path.exists() {
        return Ok(CacheStatus::Miss);
    }

    emit(
        InstallEvent::set_task(&install.ident, InstallTask::Install),
        app,
    );

    let mut manager = app.lock_manager();

    let (game, profile) = manager.profile_by_id_mut(batch.profile_id)?;
    if let Some(callback) = &batch.options.before_install {
        callback(install, profile)?;
    }

    debug!(
        ident = %install.ident,
        "cache hit, installing from cache"
    );

    let package_name = install.ident.full_name();
    let mut installer = game.mod_loader.installer_for(package_name);
    installer.install(&cache_path, package_name, profile)?;

    install.clone().insert_into(profile)?;

    profile.save(app, true)?;

    emit(
        InstallEvent::AddProgress {
            mods: 1,
            bytes: install.file_size as i64,
        },
        app,
    );

    Ok(CacheStatus::Hit)
}

async fn download(
    install: &ModInstall,
    cancel: &AtomicBool,
    options: &InstallOptions,
    app: &AppHandle,
) -> InstallResult<Vec<u8>> {
    emit(
        InstallEvent::set_task(&install.ident, InstallTask::Download),
        app,
    );

    let url = install.id.backend.download_url(&install.ident);

    debug!(
        ident = %install.ident,
        size = install.file_size,
        url = %url,
        "downloading mod"
    );

    const MAX_RETRIES: usize = 3;
    const INITIAL_BACKOFF: Duration = Duration::from_secs(2);

    let mut response = Vec::with_capacity(install.file_size as usize);
    let mut retries = 0;
    let mut backoff = INITIAL_BACKOFF;

    loop {
        match try_download(&mut response, &url, cancel, options, app).await {
            Ok(()) => break Ok(response),
            Err(InstallError::Cancelled) => return Err(InstallError::Cancelled),
            Err(InstallError::Error(err)) => {
                if retries >= MAX_RETRIES {
                    break Err(InstallError::Error(err.wrap_err("max retries exceeded")));
                }

                emit(
                    InstallEvent::AddProgress {
                        mods: 0,
                        bytes: -(response.len() as i64),
                    },
                    app,
                );

                response.clear();

                retries += 1;

                warn!(attempt = retries, err = ?err, url = %url, backoff = ?backoff, "download failed, retrying");

                tokio::time::sleep(backoff).await;
                backoff *= 2;
            }
        }
    }
}

async fn try_download(
    buf: &mut Vec<u8>,
    url: &str,
    cancel: &AtomicBool,
    options: &InstallOptions,
    app: &AppHandle,
) -> InstallResult<()> {
    const UPDATE_DELAY: Duration = Duration::from_millis(100);

    let mut stream = app
        .http()
        .get(url)
        .send()
        .await
        .and_then(|response| response.error_for_status())
        .context("failed to send request")?
        .bytes_stream();

    let mut last_update = Instant::now();
    let mut last_size_update = 0i64;

    while let Some(item) = stream.next().await {
        let item = item.context("failed to read chunk from stream")?;
        buf.extend_from_slice(&item);

        if last_update.elapsed() >= UPDATE_DELAY {
            last_update = Instant::now();
            emit(
                InstallEvent::AddProgress {
                    mods: 0,
                    bytes: buf.len() as i64 - last_size_update,
                },
                app,
            );
            last_size_update = buf.len() as i64;

            check_cancel(cancel, options)?;
        }
    }

    emit(
        InstallEvent::AddProgress {
            mods: 0,
            bytes: buf.len() as i64 - last_size_update,
        },
        app,
    );

    Ok(())
}

fn install_from_download(
    data: Vec<u8>,
    batch: &InstallBatch,
    index: usize,
    cancel: &AtomicBool,
    app: &AppHandle,
) -> InstallResult<()> {
    let manager = app.lock_manager();

    let install = &batch.mods[index];

    let cache_path = super::cache::path(&install.ident, &app.lock_prefs());
    let package_name = install.ident.full_name();

    let (game, _) = manager.profile_by_id(batch.profile_id)?;
    drop(manager);

    fs::create_dir_all(&cache_path).fs_context("creating mod cache dir", &cache_path)?;

    let mut installer = game.mod_loader.installer_for(package_name);

    emit(
        InstallEvent::set_task(&install.ident, InstallTask::Extract),
        app,
    );

    let archive = ZipArchive::new(Cursor::new(data)).context("failed to open archive")?;

    installer
        .extract(archive, package_name, cache_path.clone())
        .inspect_err(|_| {
            // the cached mod is probably in an invalid state, so remove it
            fs::remove_dir_all(&cache_path).unwrap_or_else(|err| {
                warn!(
                    "failed to clean up after failed extraction of {}: {:#}",
                    install.ident, err
                );
            });
        })
        .context("error while extracting")?;

    check_cancel(cancel, &batch.options)?;

    emit(
        InstallEvent::set_task(&install.ident, InstallTask::Install),
        app,
    );

    let mut manager = app.lock_manager();
    let (_, profile) = manager.profile_by_id_mut(batch.profile_id)?;

    if let Some(callback) = &batch.options.before_install {
        callback(install, profile)?;
    }

    debug!(
        ident = %install.ident,
        "installing mod"
    );

    installer.install(&cache_path, package_name, profile)?;
    install.clone().insert_into(profile)?;

    profile.save(app, true)?;

    emit(InstallEvent::AddProgress { mods: 1, bytes: 0 }, app);

    Ok(())
}

/// Events sent to the frontend to keep track of installation progress.
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase", tag = "type")]
enum InstallEvent<'a> {
    Show,
    #[serde(rename_all = "camelCase")]
    Hide {
        reason: HideReason,
    },
    #[serde(rename_all = "camelCase")]
    AddCount {
        mods: usize,
        bytes: i64,
    },
    #[serde(rename_all = "camelCase")]
    AddProgress {
        mods: usize,
        bytes: i64,
    },
    #[serde(rename_all = "camelCase")]
    SetTask {
        name: &'a str,
        task: InstallTask,
    },
}

#[derive(Debug, Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
enum HideReason {
    Done,
    Error,
    Cancelled,
}

#[derive(Debug, Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
enum InstallTask {
    Download,
    Extract,
    Install,
}

impl<'a> InstallEvent<'a> {
    fn set_task(ident: &'a VersionIdent, task: InstallTask) -> Self {
        Self::SetTask {
            name: ident.name(),
            task,
        }
    }
}

fn emit(event: InstallEvent, app: &AppHandle) {
    app.emit("install_event", event).ok();
}

fn check_cancel(cancel: &AtomicBool, options: &InstallOptions) -> InstallResult<()> {
    if cancel.load(Ordering::SeqCst) {
        if options.cancel_behavior == CancelBehavior::Prevent {
            warn!("attempted to cancel uncancellable batch");
            cancel.store(false, Ordering::SeqCst);

            Ok(())
        } else {
            Err(InstallError::Cancelled)
        }
    } else {
        Ok(())
    }
}
