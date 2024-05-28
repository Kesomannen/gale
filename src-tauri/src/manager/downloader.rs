use std::{
    fs,
    io::Cursor,
    iter,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Duration, Instant},
};

use anyhow::{anyhow, Context, Result};
use futures_util::StreamExt;
use serde::Serialize;
use tauri::{AppHandle, Manager};
use thiserror::Error;
use typeshare::typeshare;

use crate::{
    command_util::StateMutex,
    fs_util,
    prefs::Prefs,
    thunderstore::{BorrowedMod, Thunderstore},
    util::{print_err, IoResultExt},
    NetworkClient,
};

use super::{commands::save, ModManager, ModRef, Profile, ProfileMod};

pub mod commands;
pub mod updater;

pub fn setup(app: &AppHandle) -> Result<()> {
    app.manage(Mutex::new(InstallState::default()));

    if !cfg!(target_os = "linux") {
        tauri_plugin_deep_link::register("ror2mm", deep_link_handler(app.clone()))?;
    }

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
) -> Result<impl Iterator<Item = BorrowedMod<'a>>> {
    Ok(thunderstore
        .dependencies(borrowed_mod.version)
        .0
        .into_iter()
        .chain(iter::once(borrowed_mod))
        .filter(|dep| !profile.has_mod(&dep.package.uuid4)))
}

fn total_download_size(
    borrowed_mod: BorrowedMod<'_>,
    profile: &Profile,
    prefs: &Prefs,
    thunderstore: &Thunderstore,
) -> Result<u64> {
    Ok(missing_deps(borrowed_mod, profile, thunderstore)?
        .filter(|borrowed_mod| match cache_path(borrowed_mod, prefs) {
            Ok(cache_path) => !cache_path.exists(),
            Err(_) => true,
        })
        .map(|borrowed_mod| borrowed_mod.version.file_size)
        .sum())
}

fn cache_path(borrowed_mod: &BorrowedMod<'_>, prefs: &Prefs) -> Result<PathBuf> {
    let mut path = prefs.get_path_or_err("cache_dir")?.clone();
    path.push(&borrowed_mod.package.full_name);
    path.push(&borrowed_mod.version.version_number.to_string());

    Ok(path)
}

fn try_cache_install(
    borrowed_mod: BorrowedMod<'_>,
    profile: &mut Profile,
    path: &Path,
) -> Result<bool> {
    match path.try_exists().fs_context("checking cache", path)? {
        true => {
            let name = &borrowed_mod.package.full_name;
            install_from_disk(path, &profile.path, name)?;
            profile.mods.push(ProfileMod::remote_now(borrowed_mod.reference()));
            Ok(true)
        }
        false => Ok(false),
    }
}

const DOWNLOAD_UPDATE_INTERVAL: Duration = Duration::from_millis(100);

#[typeshare]
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct InstallProgress<'a> {
    total_progress: f32,
    installed_mods: usize,
    total_mods: usize,
    current_name: &'a str,
    can_cancel: bool,
    task: InstallTask,
}

#[typeshare]
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase", tag = "kind", content = "payload")]
enum InstallTask {
    Done,
    Error,
    Downloading { total: u64, downloaded: u64 },
    Extracting,
    Installing,
}

struct Installer<'a> {
    to_install: &'a [(ModRef, bool)],
    can_cancel: bool,
    index: usize,
    current_name: String,

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
        to_install: &'a [(ModRef, bool)],
        can_cancel: bool,
        client: &'a reqwest::Client,
        app: &'a AppHandle,
    ) -> Result<Self> {
        let manager = app.state::<Mutex<ModManager>>();
        let thunderstore = app.state::<Mutex<Thunderstore>>();
        let prefs = app.state::<Mutex<Prefs>>();
        let install_state = app.state::<Mutex<InstallState>>();

        let mut total_bytes = 0u64;

        {
            let ts_lock = thunderstore.lock().unwrap();

            for (mod_ref, _) in to_install {
                let borrowed_mod = mod_ref.borrow(&ts_lock)?;
                total_bytes += borrowed_mod.version.file_size;
            }
        }

        Ok(Self {
            to_install,
            can_cancel,
            index: 0,
            app,
            client,
            total_bytes,
            completed_bytes: 0,
            current_name: String::new(),
            manager,
            thunderstore,
            prefs,
            install_state,
        })
    }

    fn is_cancelled(&self) -> bool {
        self.can_cancel && self.install_state.lock().unwrap().cancelled
    }

    fn check_cancelled(&self) -> InstallResult<()> {
        match self.is_cancelled() {
            true => Err(InstallError::Cancelled),
            false => Ok(()),
        }
    }

    fn update(&self, task: InstallTask) {
        let total_progress = self.completed_bytes as f32 / self.total_bytes as f32;

        self.app
            .emit_all(
                "install_progress",
                InstallProgress {
                    task,
                    total_progress,
                    installed_mods: self.index,
                    total_mods: self.to_install.len(),
                    can_cancel: self.can_cancel,
                    current_name: &self.current_name,
                },
            )
            .ok();
    }

    fn prepare_install(&mut self, mod_ref: &ModRef, enabled: bool) -> Result<InstallMethod> {
        let mut manager = self.manager.lock().unwrap();
        let thunderstore = self.thunderstore.lock().unwrap();
        let prefs = self.prefs.lock().unwrap();

        let borrowed = mod_ref.borrow(&thunderstore)?;
        let profile = manager.active_profile_mut();
        let path = cache_path(&borrowed, &prefs)?;

        self.current_name = borrowed.package.name.clone();
        self.update(InstallTask::Installing);

        if try_cache_install(borrowed.clone(), profile, &path)? {
            if !enabled {
                profile
                    .force_toggle_mod(&mod_ref.package_uuid, &thunderstore)
                    .context("failed to disable installed mod")?;
            }

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
            .and_then(|r| r.error_for_status())
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

    fn install_from_download(
        &mut self,
        data: Vec<u8>,
        mod_ref: &ModRef,
        enabled: bool,
    ) -> InstallResult<()> {
        let mut manager = self.manager.lock().unwrap();
        let thunderstore = self.thunderstore.lock().unwrap();
        let prefs = self.prefs.lock().unwrap();

        let borrowed_mod = mod_ref.borrow(&thunderstore)?;
        let mut path = cache_path(&borrowed_mod, &prefs)?;

        fs::create_dir_all(&path).fs_context("create mod cache dir", &path)?;

        self.check_cancelled()?;
        self.update(InstallTask::Extracting);

        zip_extract::extract(Cursor::new(data), &path, false).fs_context("extracting mod", &path)?;
        normalize_mod_structure(&mut path)?;

        self.check_cancelled()?;
        self.update(InstallTask::Installing);

        let profile = manager.active_profile_mut();

        try_cache_install(borrowed_mod, profile, &path)
            .context("failed to install after download")?;

        if !enabled {
            profile
                .force_toggle_mod(&mod_ref.package_uuid, &thunderstore)
                .context("failed to disable installed mod")?;
        }

        manager.save(&prefs).context("failed to save manager state")?;

        Ok(())
    }

    async fn install(&mut self, next: ModRef, enabled: bool) -> InstallResult<()> {
        if let InstallMethod::Download { url, size } = self.prepare_install(&next, enabled)? {
            // this means we didn't install from cache
            let response = self.download(&url, size).await?;
            self.install_from_download(response, &next, enabled)?;
        }

        Ok(())
    }

    async fn install_all(&mut self) -> Result<()> {
        self.install_state.lock().unwrap().cancelled = false;

        for i in 0..self.to_install.len() {
            self.index = i;
            let (mod_ref, enabled) = &self.to_install[i];

            match self.install(mod_ref.clone(), *enabled).await {
                Ok(_) => (),
                Err(InstallError::Cancelled) => {
                    self.update(InstallTask::Error);

                    let mut manager = self.manager.lock().unwrap();
                    let thunderstore = self.thunderstore.lock().unwrap();

                    let profile = manager.active_profile_mut();

                    for j in 0..i {
                        let (mod_ref, _) = &self.to_install[j];

                        profile
                            .force_remove_mod(&mod_ref.package_uuid, &thunderstore)
                            .context("failed to clean up after cancellation")?;
                    }

                    return Ok(());
                }
                Err(InstallError::Error(err)) => {
                    self.update(InstallTask::Error);

                    let thunderstore = self.thunderstore.lock().unwrap();

                    let borrowed = mod_ref.borrow(&thunderstore)?;
                    let name = &borrowed.package.full_name;

                    return Err(err.context(format!("failed to install {}", name)));
                }
            }
        }

        self.update(InstallTask::Done);

        Ok(())
    }
}

pub fn normalize_mod_structure(path: &mut PathBuf) -> Result<()> {
    for dir in ["BepInExPack", "BepInEx", "plugins"].iter() {
        path.push(dir);
        fs_util::flatten_if_exists(&*path)?;
        path.pop();
    }

    Ok(())
}

pub async fn install_mod_refs(mod_refs: &[(ModRef, bool)], can_cancel: bool, app: &tauri::AppHandle) -> Result<()> {
    let client = app.state::<NetworkClient>();
    let mut downloader = Installer::create(mod_refs, can_cancel, &client.0, app)?;
    downloader.install_all().await
}

pub async fn install_mods<F>(get_mods: F, can_cancel: bool, app: &tauri::AppHandle) -> Result<()>
where
    F: FnOnce(&ModManager, &Thunderstore) -> Result<Vec<(ModRef, bool)>>,
{
    let to_install = {
        let manager = app.state::<Mutex<ModManager>>();
        let thunderstore = app.state::<Mutex<Thunderstore>>();

        let manager = manager.lock().unwrap();
        let thunderstore = thunderstore.lock().unwrap();

        get_mods(&manager, &thunderstore).context("failed to resolve dependencies")?
    };

    install_mod_refs(&to_install, can_cancel, app).await
}

pub async fn install_with_deps(mod_ref: &ModRef, can_cancel: bool, app: &tauri::AppHandle) -> Result<()> {
    install_mods(
        move |manager, thunderstore| {
            let borrowed_mod = mod_ref.borrow(thunderstore)?;

            Ok(
                missing_deps(borrowed_mod, manager.active_profile(), thunderstore)?
                    .map(|borrowed_mod| (ModRef::from(borrowed_mod), true))
                    .collect(),
            )
        },
        can_cancel,
        app,
    )
    .await
}

pub fn install_from_disk(src: &Path, dest: &Path, full_name: &str) -> Result<()> {
    let mut split = full_name.split('-');
    split.next();
    let name = split.next().unwrap_or(full_name);

    match name.starts_with("BepInExPack") {
        true => install_from_disk_bepinex(src, dest),
        false => install_from_disk_default(src, dest, full_name),
    }
}

fn install_from_disk_default(src: &Path, dest: &Path, name: &str) -> Result<()> {
    let target_path = dest.join("BepInEx");
    let target_plugins_path = target_path.join("plugins").join(name);
    fs::create_dir_all(&target_plugins_path).context("failed to create plugins directory")?;

    for entry in fs::read_dir(src)? {
        let entry_path = entry?.path();
        let entry_name = entry_path.file_name().unwrap();

        if entry_path.is_dir() {
            if entry_name == "config" {
                let target_path = target_path.join("config");
                fs::create_dir_all(&target_path)?;
                fs_util::copy_contents(&entry_path, &target_path, false)
                    .fs_context("copying config", &entry_path)?;
            } else {
                let target_path = match entry_name.to_string_lossy().as_ref() {
                    "patchers" | "core" | "monomod" => target_path.join(entry_name).join(name),
                    "plugins" => target_plugins_path.clone(),
                    _ => target_plugins_path.join(entry_name),
                };

                fs::create_dir_all(target_path.parent().unwrap())?;
                fs_util::copy_dir(&entry_path, &target_path)
                    .fs_context("copying directory", &entry_path)?;
            }
        } else {
            fs::copy(&entry_path, &target_plugins_path.join(entry_name))
                .fs_context("copying file", &entry_path)?;
        }
    }

    Ok(())
}

fn install_from_disk_bepinex(src: &Path, dest: &Path) -> Result<()> {
    let target_path = dest.join("BepInEx");

    // Some BepInEx packs come with a subfolder where the actual BepInEx files are
    for entry in fs::read_dir(src)? {
        let entry_path = entry?.path();
        let entry_name = fs_util::file_name(&entry_path);

        if entry_path.is_dir() && entry_name.contains("BepInEx") {
            // ... and some have even more subfolders ...
            // do this first, since otherwise entry_path will be removed already
            fs_util::flatten_if_exists(&entry_path.join("BepInEx"))?;
            fs_util::flatten_if_exists(&entry_path)?;
        }
    }

    for entry in fs::read_dir(src)? {
        let entry_path = entry?.path();
        let entry_name = entry_path.file_name().unwrap();

        if entry_path.is_dir() {
            let target_path = target_path.join(entry_name);
            fs::create_dir_all(&target_path)?;

            fs_util::copy_contents(&entry_path, &target_path, false)
                .fs_context("copying directory", &entry_path)?;
        } else if ["winhttp.dll", ".doorstop_version"].into_iter().any(|name| entry_name == name) {
            fs::copy(&entry_path, dest.join(entry_name))
                .fs_context("copying file", &entry_path)?;
        }
    }

    Ok(())
}

fn resolve_deep_link(url: String, thunderstore: &Thunderstore) -> Result<ModRef> {
    let id = url
        .strip_prefix("ror2mm://v1/install/thunderstore.io/")
        .ok_or_else(|| anyhow!("Invalid deep link url: '{}'", url))?;

    let borrowed_mod = thunderstore.find_mod(id, '/')?;

    Ok(borrowed_mod.into())
}

pub fn deep_link_handler(app: AppHandle) -> impl FnMut(String) {
    move |url| {
        let mod_ref = {
            let thunderstore = app.state::<Mutex<Thunderstore>>();
            let thunderstore = thunderstore.lock().unwrap();

            match resolve_deep_link(url, &thunderstore) {
                Ok(mod_ref) => mod_ref,
                Err(e) => {
                    print_err("failed to resolve deep link", &e, &app);
                    return;
                }
            }
        };

        let handle = app.clone();
        tauri::async_runtime::spawn(async move {
            install_with_deps(&mod_ref, true, &handle)
                .await
                .unwrap_or_else(|e| {
                    print_err("install mod from deep link", &e, &handle);
                });
        });
    }
}
