use std::{
    fs,
    io::Cursor,
    path::Path,
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

use chrono::Utc;
use core::str;
use eyre::{Context, Result};
use futures_util::StreamExt;
use tracing::warn;
use tauri::{AppHandle, Emitter};
use thiserror::Error;
use zip::ZipArchive;

use super::{cache, InstallOptions, InstallProgress, InstallTask, ModInstall};
use crate::{
    profile::{ModManager, ProfileMod, ProfileModKind, ThunderstoreMod},
    state::ManagerExt,
    thunderstore::Thunderstore,
    util::error::IoResultExt,
};

const DOWNLOAD_UPDATE_INTERVAL: Duration = Duration::from_millis(100);

pub struct Installer<'a> {
    options: InstallOptions,
    index: usize,
    current_name: String,

    start_time: Instant,
    total_mods: usize,
    total_bytes: u64,
    completed_bytes: u64,

    app: &'a AppHandle,
}

enum InstallMethod {
    Cached,
    Download { url: String, file_size: u64 },
}

#[derive(Debug, Error)]
enum InstallError {
    #[error("cancelled")]
    Cancelled,

    #[error(transparent)]
    Error(#[from] eyre::Error),
}

type InstallResult<T> = std::result::Result<T, InstallError>;

impl<'a> Installer<'a> {
    pub fn create(options: InstallOptions, app: &'a AppHandle) -> Result<Self> {
        Ok(Self {
            options,
            index: 0,
            app,
            total_mods: 0,
            total_bytes: 0,
            completed_bytes: 0,
            current_name: String::new(),
            start_time: Instant::now(),
        })
    }

    fn is_cancelled(&self) -> bool {
        self.options.can_cancel
            && self
                .app
                .app_state()
                .cancel_install_flag
                .load(Ordering::Relaxed)
    }

    fn check_cancel(&self) -> InstallResult<()> {
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
            duration_secs: self.start_time.elapsed().as_secs_f32(),
        };

        if let Some(callback) = &self.options.on_progress {
            callback(&progress, self.app);
        }

        if self.options.send_progress {
            self.app.emit("install_progress", &progress).ok();
        }
    }

    fn try_cache_install(&mut self, data: &ModInstall) -> Result<InstallMethod> {
        let prefs = self.app.lock_prefs();
        let mut manager = self.app.lock_manager();
        let thunderstore = self.app.lock_thunderstore();

        let version = data.id.borrow(&thunderstore)?.version;
        let cache_path = cache::path(&version.ident, &prefs);

        self.current_name = version.name().to_owned();

        if cache_path.exists() {
            self.update(InstallTask::Installing);

            if let Some(callback) = &self.options.before_install {
                callback(data, &mut manager, &thunderstore)?;
            }

            cache_install(data, &cache_path, &mut manager, &thunderstore)?;

            self.completed_bytes += version.file_size;
            manager.active_profile().save(self.app.db())?;

            Ok(InstallMethod::Cached)
        } else {
            Ok(InstallMethod::Download {
                url: version.download_url(),
                file_size: version.file_size,
            })
        }
    }

    async fn download(&mut self, url: &str, file_size: u64) -> InstallResult<Vec<u8>> {
        self.update(InstallTask::Downloading {
            total: file_size,
            downloaded: 0,
        });

        let mut stream = self
            .app
            .http()
            .get(url)
            .send()
            .await
            .and_then(|response| response.error_for_status())
            .map_err(|err| InstallError::Error(err.into()))?
            .bytes_stream();

        let mut last_update = Instant::now();
        let mut response = Vec::with_capacity(file_size as usize);

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

                self.check_cancel()?;
            };
        }

        Ok(response)
    }

    fn install_from_download(&mut self, data: Vec<u8>, install: &ModInstall) -> InstallResult<()> {
        let prefs = self.app.lock_prefs();
        let mut manager = self.app.lock_manager();
        let thunderstore = self.app.lock_thunderstore();

        let version = install.id.borrow(&thunderstore)?.version;
        let cache_path = cache::path(&version.ident, &prefs);

        fs::create_dir_all(&cache_path).fs_context("creating mod cache dir", &cache_path)?;

        self.check_cancel()?;
        self.update(InstallTask::Extracting);

        let mut installer = manager
            .active_game
            .mod_loader
            .installer_for(version.full_name());

        let archive = ZipArchive::new(Cursor::new(data)).context("failed to open archive")?;

        installer
            .extract(archive, version.full_name(), cache_path.clone())
            .inspect_err(|_| {
                // the cached mod is probably in an invalid state
                fs::remove_dir_all(&cache_path).unwrap_or_else(|err| {
                    warn!(
                        "failed to clean up after failed extraction of {}: {:#}",
                        self.current_name, err
                    );
                });
            })
            .context("error while extracting")?;

        self.check_cancel()?;
        self.update(InstallTask::Installing);

        if let Some(callback) = &self.options.before_install {
            callback(install, &mut manager, &thunderstore)?;
        }

        cache_install(install, &cache_path, &mut manager, &thunderstore)?;

        manager.active_profile().save(self.app.db())?;

        Ok(())
    }

    async fn install(&mut self, data: &ModInstall) -> InstallResult<()> {
        if let InstallMethod::Download { url, file_size } = self.try_cache_install(data)? {
            let response = self.download(&url, file_size).await?;
            self.install_from_download(response, data)
        } else {
            Ok(())
        }
    }

    pub async fn install_all(&mut self, mods: Vec<ModInstall>) -> Result<()> {
        self.app
            .app_state()
            .cancel_install_flag
            .store(false, Ordering::Relaxed);

        self.total_mods = mods.len();
        self.count_total_bytes(&mods)?;

        for i in 0..mods.len() {
            self.index = i;
            let data = &mods[i];

            match self.install(data).await {
                Ok(()) => (),
                Err(InstallError::Cancelled) => {
                    self.update(InstallTask::Error);

                    let mut manager = self.app.lock_manager();

                    let profile = manager.active_profile_mut();

                    for install in mods.iter().take(i) {
                        profile
                            .force_remove_mod(install.uuid())
                            .context("failed to clean up after cancellation")?;
                    }

                    return Ok(());
                }
                Err(InstallError::Error(err)) => {
                    self.update(InstallTask::Error);

                    let thunderstore = self.app.lock_thunderstore();

                    let borrowed = data.id.borrow(&thunderstore)?;
                    let name = &borrowed.package.ident;

                    return Err(err.wrap_err(format!("failed to install {}", name)));
                }
            }
        }

        self.update(InstallTask::Done);

        let manager = self.app.lock_manager();
        let thunderstore = self.app.lock_thunderstore();

        manager.cache_mods(&thunderstore).ok();

        Ok(())
    }

    fn count_total_bytes(&mut self, mods: &Vec<ModInstall>) -> Result<()> {
        let thunderstore = self.app.lock_thunderstore();
        for install in mods {
            let borrowed = install.id.borrow(&thunderstore)?;
            self.total_bytes += borrowed.version.file_size;
        }

        Ok(())
    }
}

fn cache_install(
    data: &ModInstall,
    src: &Path,
    manager: &mut ModManager,
    thunderstore: &Thunderstore,
) -> Result<()> {
    let borrowed = data.id.borrow(thunderstore)?;
    let package_name = borrowed.ident().full_name();

    let mut installer = manager.active_game.mod_loader.installer_for(package_name);
    let profile = manager.active_profile_mut();

    installer.install(src, package_name, profile)?;

    let install_time = data.install_time.unwrap_or_else(Utc::now);

    let profile_mod = ProfileMod::new_at(
        install_time,
        ProfileModKind::Thunderstore(ThunderstoreMod {
            ident: borrowed.ident().clone(),
            id: borrowed.into(),
        }),
    );

    match data.index {
        Some(index) if index < profile.mods.len() => {
            profile.mods.insert(index, profile_mod);
        }
        _ => {
            profile.mods.push(profile_mod);
        }
    };

    if !data.enabled {
        profile.force_toggle_mod(borrowed.package.uuid)?;
    }

    Ok(())
}
