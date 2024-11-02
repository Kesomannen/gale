use std::{
    fs,
    io::Cursor,
    sync::Mutex,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use core::str;
use futures_util::StreamExt;
use tauri::{AppHandle, Emitter, Manager};
use thiserror::Error;

use crate::{
    prefs::Prefs,
    profile::{commands::save, ModManager},
    thunderstore::Thunderstore,
    util::{cmd::StateMutex, error::IoResultExt},
};

use super::{cache, InstallOptions, InstallProgress, InstallTask, ModInstall};

#[derive(Default)]
pub struct InstallState {
    pub cancelled: bool,
}

const DOWNLOAD_UPDATE_INTERVAL: Duration = Duration::from_millis(100);

pub struct Installer<'a> {
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
    pub fn create(
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

        let borrowed = data.id.borrow(&thunderstore)?;
        let path = cache::path(&borrowed.version.ident, &prefs);

        self.current_name = borrowed.package.name().to_owned();
        self.update(InstallTask::Installing);

        if path.exists() {
            if let Some(callback) = &self.options.before_install {
                callback(data, &mut manager, &thunderstore);
            }
        }

        if super::fs::try_cache_install(data, &path, &mut manager, &thunderstore)? {
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

        let borrowed = install.id.borrow(&thunderstore)?;
        let cache_path = cache::path(&borrowed.version.ident, &prefs);

        fs::create_dir_all(&cache_path).fs_context("create mod cache dir", &cache_path)?;

        self.check_cancelled()?;
        self.update(InstallTask::Extracting);

        super::fs::extract(
            Cursor::new(data),
            borrowed.package.ident.as_str(),
            cache_path.clone(),
        )
        .context("failed to extract mod")?;

        self.check_cancelled()?;
        self.update(InstallTask::Installing);

        if let Some(callback) = &self.options.before_install {
            callback(install, &mut manager, &thunderstore);
        }

        super::fs::try_cache_install(install, &cache_path, &mut manager, &thunderstore)
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

    pub async fn install_all(&mut self, to_install: Vec<ModInstall>) -> Result<()> {
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

                    let borrowed = data.id.borrow(&thunderstore)?;
                    let name = &borrowed.package.ident;

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
            let borrowed_mod = install.id.borrow(&thunderstore)?;
            self.total_bytes += borrowed_mod.version.file_size;
        }

        Ok(())
    }
}
