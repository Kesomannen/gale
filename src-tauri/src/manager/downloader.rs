use std::{
    fs,
    io::Cursor,
    iter,
    path::{Path, PathBuf},
    sync::Mutex,
};

use anyhow::{anyhow, ensure, Context, Result};
use itertools::Itertools;
use serde::Serialize;
use tauri::{AppHandle, Manager};
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
use futures_util::StreamExt;

pub mod commands;

pub fn setup(app: &AppHandle) -> Result<()> {
    tauri_plugin_deep_link::register("ror2mm", deep_link_handler(app.clone()))?;

    Ok(())
}

fn missing_deps<'a>(
    borrowed_mod: BorrowedMod<'a>,
    profile: &'a Profile,
    thunderstore: &'a Thunderstore,
) -> impl Iterator<Item = Result<BorrowedMod<'a>>> {
    thunderstore
        .dependencies(borrowed_mod.version)
        .filter_ok(|dep| !profile.has_mod(&dep.package.uuid4))
        .chain(iter::once(Ok(borrowed_mod)))
}

fn total_download_size(
    borrowed_mod: BorrowedMod<'_>,
    profile: &Profile,
    prefs: &Prefs,
    thunderstore: &Thunderstore,
) -> u64 {
    missing_deps(borrowed_mod, profile, thunderstore)
        .filter_map(Result::ok)
        .filter(|borrowed_mod| {
            let cache_path = cache_path(borrowed_mod, prefs);

            !cache_path.try_exists().unwrap_or(false)
        })
        .map(|borrowed_mod| borrowed_mod.version.file_size as u64)
        .sum()
}

fn cache_path(borrowed_mod: &BorrowedMod<'_>, prefs: &Prefs) -> PathBuf {
    let mut path = prefs.cache_path.clone();
    path.push(&borrowed_mod.package.full_name);
    path.push(&borrowed_mod.version.version_number.to_string());

    path
}

fn try_cache_install(
    borrowed_mod: &BorrowedMod<'_>,
    profile: &mut Profile,
    path: &Path,
) -> Result<bool> {
    match path.try_exists().fs_context("check cache", path)? {
        true => {
            let name = &borrowed_mod.package.full_name;
            install_from_disk(path, &profile.path, name)?;
            profile.mods.push(ProfileMod::Remote(borrowed_mod.into()));
            Ok(true)
        }
        false => Ok(false),
    }
}

async fn install_mod(
    mod_ref: &ModRef,
    mut on_task_update: impl FnMut(InstallTask, u32),
    client: &reqwest::Client,
    manager: StateMutex<'_, ModManager>,
    thunderstore: StateMutex<'_, Thunderstore>,
    prefs: StateMutex<'_, Prefs>,
) -> Result<()> {
    let (url, total) = {
        let mut manager = manager.lock().unwrap();
        let thunderstore = thunderstore.lock().unwrap();
        let prefs = prefs.lock().unwrap();

        let borrowed_mod = mod_ref.borrow(&thunderstore)?;
        let profile = manager.active_profile_mut();
        let cache_path = cache_path(&borrowed_mod, &prefs);

        let result = try_cache_install(&borrowed_mod, profile, &cache_path)
            .context("failed to install from cache")?;

        if result {
            let diff = borrowed_mod.version.file_size;
            on_task_update(InstallTask::Installing, diff);

            save(&manager, &prefs)?;
            return Ok(());
        }

        (
            borrowed_mod.version.download_url.clone(),
            borrowed_mod.version.file_size,
        )
    }; // we can't carry the locks across await points

    let mut stream = client.get(&url)
        .send().await?
        .error_for_status()?
        .bytes_stream();

    let mut i = 0;
    let mut prev_update = 0;
    let mut response = Vec::new();

    while let Some(item) = stream.next().await {
        let item = item?;
        response.extend_from_slice(&item);
        
        if i % 50 == 0 {
            let downloaded = response.len() as u32;
            let diff = downloaded - prev_update;
            prev_update = downloaded;

            on_task_update(InstallTask::Downloading { downloaded, total }, diff);
        }

        i += 1;
    }

    let mut manager = manager.lock().unwrap();
    let thunderstore = thunderstore.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    let borrowed_mod = mod_ref.borrow(&thunderstore)?;
    let mut path = cache_path(&borrowed_mod, &prefs);

    fs::create_dir_all(&path)
        .fs_context("create mod cache dir", &path)?;

    on_task_update(InstallTask::Extracting, 0);

    zip_extract::extract(Cursor::new(response), &path, true)
        .fs_context("extracting mod", &path)?;

    normalize_mod_structure(&mut path)?;

    on_task_update(InstallTask::Installing, 0);

    let profile = manager.active_profile_mut();

    let result = try_cache_install(&borrowed_mod, profile, &path)
        .context("failed to install from cache after download")?;

    ensure!(result, "mod not found in cache after download");

    save(&manager, &prefs)?;

    Ok(())
}

pub fn normalize_mod_structure(path: &mut PathBuf) -> Result<()> {
    for dir in ["BepInExPack", "BepInEx", "plugins"].iter() {
        path.push(dir);
        fs_util::flatten_if_exists(&*path)?;
        path.pop();
    }

    Ok(())
}

#[typeshare]
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase", tag = "type", content = "content")]
enum InstallProgressPayload<'a> {
    #[serde(rename_all = "camelCase")]
    InProgress(&'a InstallProgress<'a>),
    Done,
    Error,
}

#[typeshare]
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct InstallProgress<'a> {
    installed_mods: usize,
    total_mods: usize,

    downloaded_bytes: u32,
    total_bytes: u32,

    current_mod_name: &'a str,
    current_task: InstallTask,
}

#[typeshare]
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase", tag = "type", content = "content")]
enum InstallTask {
    Installing,
    Extracting,
    Downloading {
        total: u32,
        downloaded: u32,
    }
}

pub async fn install_mods(mod_refs: &[ModRef], app: &tauri::AppHandle) -> Result<()> {
    let client = app.state::<NetworkClient>();
    let client = &client.0;

    let manager = app.state::<Mutex<ModManager>>();
    let thunderstore = app.state::<Mutex<Thunderstore>>();
    let prefs = app.state::<Mutex<Prefs>>();

    let mod_info = {
        let thunderstore = thunderstore.lock().unwrap();

        mod_refs.iter()
            .map(|mod_ref| { 
                let borrowed_mod = mod_ref.borrow(&thunderstore)?;

                Ok((
                    borrowed_mod.package.name.clone(),
                    borrowed_mod.version.file_size,
                ))
            })
            .collect::<Result<Vec<_>>>()?
    };

    let total_mods = mod_info.len();
    let total_bytes = mod_info.iter().map(|(_, size)| size).sum();
    let mut downloaded_bytes = 0;

    for (i, mod_ref) in mod_refs.iter().enumerate() {
        let info = &mod_info[i];
        let mut progress = InstallProgress {
            installed_mods: i,
            total_mods,
            downloaded_bytes,
            total_bytes,
            current_mod_name: &info.0,
            current_task: InstallTask::Installing,
        };

        update(app, InstallProgressPayload::InProgress(&progress));

        let result = install_mod(
            mod_ref,
            |task, diff| {
                downloaded_bytes += diff;
                progress.downloaded_bytes = downloaded_bytes;
                
                progress.current_task = task;
                update(app, InstallProgressPayload::InProgress(&progress));
            },
            client,
            manager.clone(),
            thunderstore.clone(),
            prefs.clone(),
        )
        .await;

        if let Err(err) = result {
            update(app, InstallProgressPayload::Error);
            let err = err.context(format!("failed to install mod {}", info.0));
            return Err(err);
        }
    }

    update(app, InstallProgressPayload::Done);

    return Ok(());

    fn update(app: &AppHandle, payload: InstallProgressPayload) {
        let _ = app.emit_all("install_progress", payload);
    }
}

pub async fn install_deps<F>(get_deps: F, app: &tauri::AppHandle) -> Result<()>
where
    F: FnOnce(&ModManager, &Thunderstore) -> Result<Vec<ModRef>>,
{
    let to_install = {
        let manager = app.state::<Mutex<ModManager>>();
        let thunderstore = app.state::<Mutex<Thunderstore>>();

        let manager = manager.lock().unwrap();
        let thunderstore = thunderstore.lock().unwrap();

        get_deps(&manager, &thunderstore).context("failed to resolve dependencies")?
    };

    install_mods(&to_install, app).await
}

pub async fn install_with_deps(mod_ref: &ModRef, app: &tauri::AppHandle) -> Result<()> {
    install_deps(move |manager, thunderstore| {
        let borrowed_mod = mod_ref.borrow(thunderstore)?;

        missing_deps(borrowed_mod, manager.active_profile(), thunderstore)
            .map_ok(|borrowed_mod| ModRef::from(&borrowed_mod))
            .collect::<Result<Vec<_>>>()
    }, app).await
}

pub fn install_from_disk(src: &Path, dest: &Path, name: &str) -> Result<()> {
    let author = name.split('-').next().context("invalid name")?;

    match author {
        "BepInEx" => install_from_disk_bepinex(src, dest),
        _ => install_from_disk_default(src, dest, name),
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
                    .context("error while copying config")?;
            } else {
                let target_path = match entry_name.to_string_lossy().as_ref() {
                    "patchers" | "core" => target_path.join(entry_name).join(name),
                    "plugins" => target_plugins_path.clone(),
                    _ => target_plugins_path.join(entry_name),
                };

                fs::create_dir_all(target_path.parent().unwrap())?;
                fs_util::copy_dir(&entry_path, &target_path)
                    .context("error while copying directory")?;
            }
        } else {
            fs::copy(&entry_path, &target_plugins_path.join(entry_name))
                .context("error while copying file")?;
        }
    }

    Ok(())
}

fn install_from_disk_bepinex(src: &Path, dest: &Path) -> Result<()> {
    let target_path = dest.join("BepInEx");

    for entry in fs::read_dir(src)? {
        let entry_path = entry?.path();
        let entry_name = entry_path.file_name().unwrap();

        if entry_path.is_dir() {
            let target_path = target_path.join(entry_name);
            fs::create_dir_all(&target_path)?;

            fs_util::copy_contents(&entry_path, &target_path, false)
                .with_context(|| format!("error while copying directory {:?}", entry_path))?;
        } else if entry_name == "winhttp.dll" {
            fs::copy(&entry_path, dest.join(entry_name))
                .with_context(|| format!("error while copying file {:?}", entry_path))?;
        }
    }

    Ok(())
}

fn resolve_deep_link(url: String, thunderstore: &Thunderstore) -> Result<ModRef> {
    let id = url
        .strip_prefix("ror2mm://v1/install/thunderstore.io/")
        .ok_or_else(|| anyhow!("Invalid deep link url: '{}'", url))?;

    let borrowed_mod = thunderstore.find_mod(id, '/')?;

    Ok((&borrowed_mod).into())
}

pub fn deep_link_handler(app: AppHandle) -> impl FnMut(String) {
    move |url| {
        let mod_ref = {
            let thunderstore = app.state::<Mutex<Thunderstore>>();
            let thunderstore = thunderstore.lock().unwrap();

            match resolve_deep_link(url, &thunderstore) {
                Ok(mod_ref) => mod_ref,
                Err(e) => {
                    print_err("Failed to resolve deep link", &e, &app);
                    return;
                }
            }
        };

        let handle = app.clone();
        tauri::async_runtime::spawn(async move {
            install_with_deps(&mod_ref, &handle)
                .await
                .unwrap_or_else(|e| {
                    print_err("install mod from deep link", &e, &handle);
                });
        });
    }
}
