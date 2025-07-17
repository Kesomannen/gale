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
use tokio::sync::{futures::Notified, oneshot, Notify};
use tracing::warn;
use uuid::Uuid;
use zip::ZipArchive;

use crate::{
    logger,
    profile::install::{InstallOptions, ModInstall},
    state::ManagerExt,
    thunderstore::VersionIdent,
    util::error::IoResultExt,
};

#[derive(Default)]
struct InstallQueueState {
    pending: VecDeque<InstallBatch>,
    processing: Option<(i64, Vec<Uuid>)>,
}

pub struct InstallQueue {
    state: Mutex<InstallQueueState>,
    notify_push: tokio::sync::Notify,
    notify_empty: tokio::sync::Notify,
}

impl InstallQueue {
    pub fn new(app: AppHandle) -> Self {
        tauri::async_runtime::spawn(handle_queue(app));

        Self {
            state: Mutex::new(InstallQueueState::default()),
            notify_push: Notify::new(),
            notify_empty: Notify::new(),
        }
    }

    pub fn wait_for_empty(&self) -> Notified {
        self.notify_empty.notified()
    }

    pub fn handle(&self) -> InstallQueueHandle {
        InstallQueueHandle {
            state: self.state.lock().unwrap(),
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

pub struct InstallQueueHandle<'a> {
    state: MutexGuard<'a, InstallQueueState>,
    queue: &'a InstallQueue,
}

impl<'a> InstallQueueHandle<'a> {
    pub fn is_processing(&self) -> bool {
        self.state.processing.is_some()
    }

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

    fn push_mods(
        &mut self,
        mods: impl IntoIterator<Item = ModInstall>,
        profile_id: i64,
        options: InstallOptions,
        app: &AppHandle,
    ) -> impl Future<Output = Result<()>> {
        let (tx, rx) = oneshot::channel();

        let mods = mods
            .into_iter()
            .filter(|install| !self.has_mod(install.uuid(), profile_id))
            .collect_vec();

        let mod_count = mods.len();
        let bytes = mods.iter().map(|install| install.file_size).sum();

        let batch = InstallBatch {
            mods,
            options,
            profile_id,
            on_complete: tx,
        };

        if mod_count > 0 {
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
            // complete the task immediately since there are no mods to install
            batch.complete(Ok(()), app);
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
    on_complete: oneshot::Sender<Result<()>>,
}

impl InstallBatch {
    fn complete(self, result: Result<()>, app: &AppHandle) {
        match result {
            Ok(()) => {
                self.on_complete.send(Ok(())).ok();
            }
            Err(err) => {
                let err = match self.on_complete.send(Err(err)) {
                    Ok(()) => return,
                    Err(err) => err.unwrap_err(),
                };

                // in case the receiver has been dropped, show an error on the frontend
                logger::log_webview_err(format!("Failed to install batch"), err, &app);
            }
        }
    }
}

async fn handle_queue(app: AppHandle) {
    let queue = app.install_queue();

    loop {
        queue.notify_empty.notify_waiters();
        queue.notify_push.notified().await;

        emit(InstallEvent::Show, &app);

        loop {
            let batch = queue.handle().pop_next();

            match batch {
                Some(batch) => handle_batch(batch, &app).await,
                None => break,
            }
        }

        emit(InstallEvent::Hide, &app);

        app.lock_manager()
            .cache_mods(&app.lock_thunderstore(), &app.lock_prefs())
            .ok();
    }
}

async fn handle_batch(batch: InstallBatch, app: &AppHandle) {
    let mut result = Ok(());

    for (i, install) in batch.mods.iter().enumerate() {
        result = handle_install(&batch, i, app)
            .await
            .wrap_err_with(|| format!("failed to install {}", install.ident));

        if result.is_err() {
            break;
        }
    }

    batch.complete(result, app);
}

async fn handle_install(batch: &InstallBatch, index: usize, app: &AppHandle) -> Result<()> {
    match try_cache_install(batch, index, app)? {
        CacheStatus::Hit => Ok(()),
        CacheStatus::Miss => {
            let bytes = download(&batch.mods[index], app).await?;
            install_from_download(bytes, batch, index, app)?;

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

    if let Some(callback) = &batch.options.before_install {
        callback(&install, &mut manager)?;
    }

    let (game, profile) = manager.profile_by_id_mut(batch.profile_id)?;
    let package_name = install.ident.full_name();

    let mut installer = game.mod_loader.installer_for(package_name);
    installer.install(&cache_path, package_name, profile)?;

    install.clone().insert_into(profile)?;

    profile.save(app.db())?;

    emit(
        InstallEvent::AddProgress {
            mods: 1,
            bytes: install.file_size,
        },
        app,
    );

    Ok(CacheStatus::Hit)
}

async fn download(install: &ModInstall, app: &AppHandle) -> Result<Vec<u8>> {
    emit(
        InstallEvent::set_task(&install.ident, InstallTask::Download),
        app,
    );

    let url = format!(
        "https://thunderstore.io/package/download/{}/",
        install.ident.path()
    );

    let mut stream = app
        .http()
        .get(url)
        .send()
        .await
        .and_then(|response| response.error_for_status())?
        .bytes_stream();

    let mut response = Vec::with_capacity(install.file_size as usize);

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

fn install_from_download(
    data: Vec<u8>,
    batch: &InstallBatch,
    index: usize,
    app: &AppHandle,
) -> Result<()> {
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

    emit(
        InstallEvent::set_task(&install.ident, InstallTask::Install),
        app,
    );

    let mut manager = app.lock_manager();

    if let Some(callback) = &batch.options.before_install {
        callback(&install, &mut manager)?;
    }

    let (_, profile) = manager.profile_by_id_mut(batch.profile_id)?;

    installer.install(&cache_path, package_name, profile)?;
    install.clone().insert_into(profile)?;

    profile.save(app.db())?;

    emit(InstallEvent::AddProgress { mods: 1, bytes: 0 }, app);

    Ok(())
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase", tag = "type")]
enum InstallEvent<'a> {
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
    #[serde(rename_all = "camelCase")]
    SetTask {
        name: &'a str,
        task: InstallTask,
    },
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
