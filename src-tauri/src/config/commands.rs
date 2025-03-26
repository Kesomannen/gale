use std::path::Path;

use eyre::{eyre, Context};
use log::trace;
use tauri::{command, AppHandle};

use super::{frontend, AnyFileKind};
use crate::{state::ManagerExt, util::cmd::Result};

#[command]
pub fn get_config_files(app: AppHandle) -> Result<Vec<frontend::File>> {
    let mut manager = app.lock_manager();
    let profile = manager.active_profile_mut();

    profile.refresh_config();

    trace!(
        "{}",
        serde_json::to_string_pretty(&profile.config_cache.to_frontend())?
    );

    Ok(profile.config_cache.to_frontend())
}

#[command]
pub fn set_config_entry(
    file: &Path,
    section: &str,
    entry: &str,
    value: frontend::Value,
    app: AppHandle,
) -> Result<()> {
    let mut manager = app.lock_manager();

    let profile = manager.active_profile_mut();
    let file = profile.config_cache.find_file(file)?;

    match &mut file.kind {
        AnyFileKind::BepInEx(file) => file.find_entry(section, entry)?.set(value),
        AnyFileKind::GDWeave(file) => file.set(entry, value),
        _ => return Err(eyre!("unsupported for this format").into()),
    }?;

    file.write(&profile.path).context("failed to write file")?;
    Ok(())
}

#[command]
pub fn reset_config_entry(
    file: &Path,
    section: &str,
    entry: &str,
    app: AppHandle,
) -> Result<frontend::Value> {
    let mut manager = app.lock_manager();

    let profile = manager.active_profile_mut();
    let file = profile.config_cache.find_file(file)?;

    let value = match &mut file.kind {
        AnyFileKind::BepInEx(file) => file.find_entry(section, entry)?.reset(),
        _ => return Err(eyre!("unsupported for this format").into()),
    }?;

    file.write(&profile.path).context("failed to write file")?;
    Ok(value)
}

#[command]
pub fn open_config_file(file: &Path, app: AppHandle) -> Result<()> {
    let manager = app.lock_manager();

    let profile = manager.active_profile();
    let path = profile.path.join(file);
    open::that(&path)
        .with_context(|| format!("failed to open config file at {}", path.display()))?;

    Ok(())
}

#[command]
pub fn delete_config_file(file: &Path, app: AppHandle) -> Result<()> {
    let mut manager = app.lock_manager();

    let profile = manager.active_profile_mut();

    let Some(index) = profile
        .config_cache
        .0
        .iter()
        .position(|f| f.relative_path == file)
    else {
        return Ok(()); // ignore if the file is not in the list
    };

    profile.config_cache.0.remove(index);

    let path = profile.path.join(file);
    trash::delete(path).context("failed to move file to recycle bin")?;

    Ok(())
}
