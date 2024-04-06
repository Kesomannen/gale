use std::fs;

use anyhow::Context;
use serde::Serialize;
use typeshare::typeshare;

use crate::{
    command_util::{Result, StateMutex}, manager::ModManager, util::IoResultExt
};

#[typeshare]
#[derive(Serialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "content")]
pub enum GetConfigResult {
    Ok(super::File),
    Err { file: String, error: String },
}

#[tauri::command]
pub fn get_config_files(manager: StateMutex<ModManager>) -> Result<Vec<GetConfigResult>> {
    let mut manager = manager.lock().unwrap();
    let profile = manager.active_profile_mut();

    profile.refresh_config();

    Ok(profile
        .config
        .iter()
        .map(|res| match res {
            Ok(file) => GetConfigResult::Ok(file.clone()),
            Err((name, err)) => GetConfigResult::Err {
                file: name.clone(),
                error: format!("{:#}", err),
            },
        })
        .collect())
}

#[tauri::command]
pub fn set_config_entry(
    file: &str,
    section: &str,
    entry: &str,
    value: super::Value,
    manager: StateMutex<ModManager>,
) -> Result<()> {
    let mut manager = manager.lock().unwrap();

    manager
        .active_profile_mut()
        .modify_config(file, section, entry, move |entry| { 
            entry.value = value;
        })?;

    Ok(())
}

#[tauri::command]
pub fn reset_config_entry(
    file: &str,
    section: &str,
    entry: &str,
    manager: StateMutex<ModManager>,
) -> Result<super::Value> {
    let mut manager = manager.lock().unwrap();

    manager
        .active_profile_mut()
        .modify_config(file, section, entry, |entry| {
            entry.reset()?;
            Ok(entry.value.clone())
        })?
}

#[tauri::command]
pub fn open_config_file(file: &str, manager: StateMutex<ModManager>) -> Result<()> {
    let manager = manager.lock().unwrap();

    let profile = manager.active_profile();
    let path = profile.find_config_file(file)?.path_from(&profile.path);
    open::that(&path).with_context(|| format!("failed to open config file {}", path.display()))?;

    Ok(())
}

#[tauri::command]
pub fn delete_config_file(file: &str, manager: StateMutex<ModManager>) -> Result<()> {
    let manager = manager.lock().unwrap();

    let profile = manager.active_profile();
    let path = profile.find_config_file(file)?.path_from(&profile.path);
    fs::remove_file(&path).fs_context("deleting config file", &path)?;

    Ok(())
}
