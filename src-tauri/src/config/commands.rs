use std::path::Path;

use eyre::{eyre, Context};

use super::{frontend, AnyFileKind};
use crate::{
    profile::ModManager,
    util::cmd::{Result, StateMutex},
};

#[tauri::command]
pub fn get_config_files(manager: StateMutex<ModManager>) -> Result<Vec<frontend::File>> {
    let mut manager = manager.lock().unwrap();
    let profile = manager.active_profile_mut();

    profile.refresh_config();

    Ok(profile.config_cache.to_frontend())
}

#[tauri::command]
pub fn set_config_entry(
    file: &Path,
    section: &str,
    entry: &str,
    value: frontend::Value,
    manager: StateMutex<ModManager>,
) -> Result<()> {
    let mut manager = manager.lock().unwrap();

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

#[tauri::command]
pub fn reset_config_entry(
    file: &Path,
    section: &str,
    entry: &str,
    manager: StateMutex<ModManager>,
) -> Result<frontend::Value> {
    let mut manager = manager.lock().unwrap();

    let profile = manager.active_profile_mut();
    let file = profile.config_cache.find_file(file)?;

    let value = match &mut file.kind {
        AnyFileKind::BepInEx(file) => file.find_entry(section, entry)?.reset(),
        _ => return Err(eyre!("unsupported for this format").into()),
    }?;

    file.write(&profile.path).context("failed to write file")?;
    Ok(value)
}

#[tauri::command]
pub fn open_config_file(file: &Path, manager: StateMutex<ModManager>) -> Result<()> {
    let manager = manager.lock().unwrap();

    let profile = manager.active_profile();
    let path = profile.path.join(file);
    open::that(&path)
        .with_context(|| format!("failed to open config file at {}", path.display()))?;

    Ok(())
}

#[tauri::command]
pub fn delete_config_file(file: &Path, manager: StateMutex<ModManager>) -> Result<()> {
    let mut manager = manager.lock().unwrap();

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
