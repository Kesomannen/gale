use std::{
    fs,
    io::{self},
    iter,
    sync::Mutex,
    time::{Duration, Instant},
};

use anyhow::{anyhow, bail, Context, Result};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use thiserror::Error;
use typeshare::typeshare;

use crate::{
    logger,
    prefs::Prefs,
    thunderstore::{BorrowedMod, Thunderstore},
    util::{cmd::StateMutex, error::IoResultExt},
    NetworkClient,
};

use super::{commands::save, installer, ModManager, ModRef, Profile};
use itertools::Itertools;
use uuid::Uuid;

pub mod commands;
pub mod updater;

pub fn setup(handle: &AppHandle) -> Result<()> {
    handle.manage(Mutex::new(InstallState::default()));

    Ok(())
}

#[derive(Default)]
pub struct InstallState {
    cancelled: bool,
}

fn missing_deps<'a>(
    borrowed_mod: BorrowedMod<'a>,
    profile: &'a Profile,
    thunderstore: &'a Thunderstore,
) -> impl Iterator<Item = BorrowedMod<'a>> {
    thunderstore
        .dependencies(borrowed_mod.version)
        .0
        .into_iter()
        .filter(|dep| !profile.has_mod(&dep.package.uuid4))
}

fn total_download_size(
    borrowed_mod: BorrowedMod<'_>,
    profile: &Profile,
    prefs: &Prefs,
    thunderstore: &Thunderstore,
) -> Result<u64> {
    Ok(missing_deps(borrowed_mod, profile, thunderstore)
        .chain(iter::once(borrowed_mod))
        .filter(|borrowed| match installer::cache_path(*borrowed, prefs) {
            Ok(cache_path) => !cache_path.exists(),
            Err(_) => true,
        })
        .map(|borrowed_mod| borrowed_mod.version.file_size)
        .sum())
}

const DOWNLOAD_UPDATE_INTERVAL: Duration = Duration::from_millis(100);

type ProgressHandler = Box<dyn Fn(&InstallProgress, &AppHandle) + 'static + Send>;
type EventHandler = Box<dyn Fn(&ModInstall, &mut ModManager, &Thunderstore) + 'static + Send>;

pub struct InstallOptions {
    can_cancel: bool,
    send_progress: bool,
    on_progress: Option<ProgressHandler>,
    before_install: Option<EventHandler>,
}

impl Default for InstallOptions {
    fn default() -> Self {
        Self {
            can_cancel: true,
            send_progress: true,
            on_progress: None,
            before_install: None,
        }
    }
}

impl InstallOptions {
    pub fn can_cancel(mut self, can_cancel: bool) -> Self {
        self.can_cancel = can_cancel;
        self
    }

    pub fn send_progress(mut self, send_progress: bool) -> Self {
        self.send_progress = send_progress;
        self
    }

    pub fn on_progress<F>(mut self, on_progress: F) -> Self
    where
        F: Fn(&InstallProgress, &AppHandle) + 'static + Send,
    {
        self.on_progress = Some(Box::new(on_progress));
        self
    }

    pub fn before_install<F>(mut self, before_install: F) -> Self
    where
        F: Fn(&ModInstall, &mut ModManager, &Thunderstore) + 'static + Send,
    {
        self.before_install = Some(Box::new(before_install));
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModInstall {
    pub mod_ref: ModRef,
    pub enabled: bool,
    pub index: Option<usize>,
}

impl ModInstall {
    pub fn new(mod_ref: ModRef) -> Self {
        Self {
            mod_ref,
            enabled: true,
            index: None,
        }
    }

    pub fn with_state(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn at(mut self, index: usize) -> Self {
        self.index = Some(index);
        self
    }

    pub fn uuid(&self) -> &Uuid {
        &self.mod_ref.package_uuid
    }
}

impl From<BorrowedMod<'_>> for ModInstall {
    fn from(borrowed_mod: BorrowedMod<'_>) -> Self {
        Self::new(borrowed_mod.into())
    }
}

#[typeshare]
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InstallProgress<'a> {
    pub total_progress: f32,
    pub installed_mods: usize,
    pub total_mods: usize,
    pub current_name: &'a str,
    pub can_cancel: bool,
    pub task: InstallTask,
}

#[typeshare]
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase", tag = "kind", content = "payload")]
pub enum InstallTask {
    Done,
    Error,
    Downloading { total: u64, downloaded: u64 },
    Extracting,
    Installing,
}

struct Installer<'a> {
    options: InstallOptions,
    index: usize,
    current_name: String,

    total_mods: usize,
    total_bytes: u64,
    completed_bytes: u64,

    app: &'a AppHandle,
    client: &'a reqwest::Client,

    thunderstore: StateMutex<'a, Thunderstore>,
    manager: StateMutex<'a, ModManager>,
    prefs: StateMutex<'a, Prefs>,
    install_state: StateMutex<'a, InstallState>,
}

enum InstallMethod {
    Cached,
    Download { url: String, size: u64 },
}

#[derive(Debug, Error)]
enum InstallError {
    #[error("cancelled")]
    Cancelled,

    #[error(transparent)]
    Error(#[from] anyhow::Error),
}

type InstallResult<T> = std::result::Result<T, InstallError>;

impl<'a> Installer<'a> {
    fn create(
        options: InstallOptions,
        client: &'a reqwest::Client,
        app: &'a AppHandle,
    ) -> Result<Self> {
        let manager = app.state::<Mutex<ModManager>>();
        let thunderstore = app.state::<Mutex<Thunderstore>>();
        let prefs = app.state::<Mutex<Prefs>>();
        let install_state = app.state::<Mutex<InstallState>>();

        Ok(Self {
            options,
            index: 0,
            app,
            client,
            total_mods: 0,
            total_bytes: 0,
            completed_bytes: 0,
            current_name: String::new(),
            manager,
            thunderstore,
            prefs,
            install_state,
        })
    }

    fn is_cancelled(&self) -> bool {
        self.options.can_cancel && self.install_state.lock().unwrap().cancelled
    }

    fn check_cancelled(&self) -> InstallResult<()> {
        match self.is_cancelled() {
            true => Err(InstallError::Cancelled),
            false => Ok(()),
        }
    }

    fn update(&self, task: InstallTask) {
        let total_progress = self.completed_bytes as f32 / self.total_bytes as f32;

        let progress = InstallProgress {
            task,
            total_progress,
            installed_mods: self.index,
            total_mods: self.total_mods,
            can_cancel: self.options.can_cancel,
            current_name: &self.current_name,
        };

        if let Some(callback) = &self.options.on_progress {
            callback(&progress, self.app);
        }

        if self.options.send_progress {
            self.app.emit("install_progress", &progress).ok();
        }
    }

    fn prepare_install(&mut self, data: &ModInstall) -> Result<InstallMethod> {
        let mut manager = self.manager.lock().unwrap();
        let thunderstore = self.thunderstore.lock().unwrap();
        let prefs = self.prefs.lock().unwrap();

        let borrowed = data.mod_ref.borrow(&thunderstore)?;
        let path = installer::cache_path(borrowed, &prefs)?;

        self.current_name.clone_from(&borrowed.package.name);
        self.update(InstallTask::Installing);

        if path.exists() {
            if let Some(callback) = &self.options.before_install {
                callback(data, &mut manager, &thunderstore);
            }
        }

        if installer::try_cache_install(data, &path, &mut manager, &thunderstore, &prefs)? {
            self.completed_bytes += borrowed.version.file_size;
            save(&manager, &prefs)?;
            return Ok(InstallMethod::Cached);
        }

        Ok(InstallMethod::Download {
            url: borrowed.version.download_url.clone(),
            size: borrowed.version.file_size,
        })
    }

    async fn download(&mut self, url: &str, file_size: u64) -> InstallResult<Vec<u8>> {
        self.update(InstallTask::Downloading {
            total: file_size,
            downloaded: 0,
        });

        let mut stream = self
            .client
            .get(url)
            .send()
            .await
            .and_then(|response| response.error_for_status())
            .map_err(|err| InstallError::Error(err.into()))?
            .bytes_stream();

        let mut last_update = Instant::now();
        let mut response = Vec::new();

        while let Some(item) = stream.next().await {
            let item = item.map_err(|err| InstallError::Error(err.into()))?;

            self.completed_bytes += item.len() as u64;
            response.extend_from_slice(&item);

            if last_update.elapsed() >= DOWNLOAD_UPDATE_INTERVAL {
                self.update(InstallTask::Downloading {
                    total: file_size,
                    downloaded: response.len() as u64,
                });

                last_update = Instant::now();

                self.check_cancelled()?;
            };
        }

        Ok(response)
    }

    fn install_from_download(&mut self, data: Vec<u8>, install: &ModInstall) -> InstallResult<()> {
        let mut manager = self.manager.lock().unwrap();
        let thunderstore = self.thunderstore.lock().unwrap();
        let prefs = self.prefs.lock().unwrap();

        let borrowed = install.mod_ref.borrow(&thunderstore)?;
        let cache_path = installer::cache_path(borrowed, &prefs)?;

        fs::create_dir_all(&cache_path).fs_context("create mod cache dir", &cache_path)?;

        self.check_cancelled()?;
        self.update(InstallTask::Extracting);

        installer::extract(
            io::Cursor::new(data),
            &borrowed.package.full_name,
            cache_path.clone(),
        )
        .context("failed to extract mod")?;

        self.check_cancelled()?;
        self.update(InstallTask::Installing);

        if let Some(callback) = &self.options.before_install {
            callback(install, &mut manager, &thunderstore);
        }

        installer::try_cache_install(install, &cache_path, &mut manager, &thunderstore, &prefs)
            .context("failed to install after download")?;

        manager
            .save(&prefs)
            .context("failed to save manager state")?;

        Ok(())
    }

    async fn install(&mut self, data: &ModInstall) -> InstallResult<()> {
        if let InstallMethod::Download { url, size } = self.prepare_install(data)? {
            // this means we didn't install from cache
            let response = self.download(&url, size).await?;
            self.install_from_download(response, data)?;
        }

        Ok(())
    }

    async fn install_all(&mut self, to_install: Vec<ModInstall>) -> Result<()> {
        self.install_state.lock().unwrap().cancelled = false;

        self.total_mods = to_install.len();
        self.count_total_bytes(&to_install)?;

        for i in 0..to_install.len() {
            self.index = i;
            let data = &to_install[i];

            match self.install(data).await {
                Ok(()) => (),
                Err(InstallError::Cancelled) => {
                    self.update(InstallTask::Error);

                    let mut manager = self.manager.lock().unwrap();

                    let profile = manager.active_profile_mut();

                    for install in to_install.iter().take(i) {
                        profile
                            .force_remove_mod(install.uuid())
                            .context("failed to clean up after cancellation")?;
                    }

                    return Ok(());
                }
                Err(InstallError::Error(err)) => {
                    self.update(InstallTask::Error);

                    let thunderstore = self.thunderstore.lock().unwrap();

                    let borrowed = data.mod_ref.borrow(&thunderstore)?;
                    let name = &borrowed.package.full_name;

                    return Err(err.context(format!("failed to install {}", name)));
                }
            }
        }

        self.update(InstallTask::Done);

        let manager = self.manager.lock().unwrap();
        let thunderstore = self.thunderstore.lock().unwrap();

        manager.cache_mods(&thunderstore).ok();

        Ok(())
    }

    fn count_total_bytes(&mut self, to_install: &Vec<ModInstall>) -> Result<()> {
        let thunderstore = self.thunderstore.lock().unwrap();
        for install in to_install {
            let borrowed_mod = install.mod_ref.borrow(&thunderstore)?;
            self.total_bytes += borrowed_mod.version.file_size;
        }

        Ok(())
    }
}

pub async fn install_mods(
    mods: Vec<ModInstall>,
    options: InstallOptions,
    app: &AppHandle,
) -> Result<()> {
    let client = app.state::<NetworkClient>();
    let mut installer = Installer::create(options, &client.0, app)?;
    installer.install_all(mods).await
}

pub async fn install_with_mods<F>(
    mods: F,
    options: InstallOptions,
    app: &tauri::AppHandle,
) -> Result<()>
where
    F: FnOnce(&ModManager, &Thunderstore) -> Result<Vec<ModInstall>>,
{
    let mods = {
        let manager = app.state::<Mutex<ModManager>>();
        let thunderstore = app.state::<Mutex<Thunderstore>>();

        let manager = manager.lock().unwrap();
        let thunderstore = thunderstore.lock().unwrap();

        mods(&manager, &thunderstore).context("failed to resolve dependencies")?
    };

    install_mods(mods, options, app).await
}

pub async fn install_with_deps(
    mods: Vec<ModInstall>,
    options: InstallOptions,
    allow_multiple: bool,
    app: &tauri::AppHandle,
) -> Result<()> {
    {
        let manager = app.state::<Mutex<ModManager>>();
        let manager = manager.lock().unwrap();

        let profile = manager.active_profile();
        if !allow_multiple && mods.len() == 1 && profile.has_mod(mods[0].uuid()) {
            bail!("mod already installed");
        }
    }

    install_with_mods(
        move |manager, thunderstore| {
            let deps = mods
                .into_iter()
                .map(|install| {
                    let borrowed = install.mod_ref.borrow(thunderstore)?;

                    Ok(
                        missing_deps(borrowed, manager.active_profile(), thunderstore)
                            .map_into()
                            .chain(iter::once(install)),
                    )
                })
                .flatten_ok()
                .collect::<Result<Vec<_>>>()?;

            Ok(deps
                .into_iter()
                .unique_by(|install| *install.uuid())
                .collect())
        },
        options,
        app,
    )
    .await
}

fn resolve_deep_link(url: &str, thunderstore: &Thunderstore) -> Result<ModRef> {
    let id = url
        .strip_prefix("ror2mm://v1/install/thunderstore.io/")
        .ok_or_else(|| anyhow!("Invalid deep link url: '{}'", url))?;

    let borrowed_mod = thunderstore.find_mod(id, '/')?;

    Ok(borrowed_mod.into())
}

pub fn handle_deep_link(handle: &AppHandle, url: &str) {
    let mod_ref = {
        let thunderstore = handle.state::<Mutex<Thunderstore>>();
        let thunderstore = thunderstore.lock().unwrap();

        match resolve_deep_link(url, &thunderstore) {
            Ok(mod_ref) => mod_ref,
            Err(err) => {
                logger::log_js_err("Failed to resolve deep link", &err, handle);
                return;
            }
        }
    };

    let handle = handle.clone();
    tauri::async_runtime::spawn(async move {
        install_with_deps(
            vec![ModInstall::new(mod_ref)],
            InstallOptions::default(),
            false,
            &handle,
        )
        .await
        .unwrap_or_else(|err| {
            logger::log_js_err("Failed to install mod from deep link", &err, &handle);
        });
    });
}
