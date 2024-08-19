use anyhow::Context;

use super::{File, LoadFileResultExt};
use crate::{
    manager::ModManager,
    util::cmd::{Result, StateMutex},
};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "content")]
pub enum FrontendLoadFileResult {
    Ok(File),
    Err { name: String, error: String },
}

#[tauri::command]
pub fn get_config_files(manager: StateMutex<ModManager>) -> Result<Vec<FrontendLoadFileResult>> {
    let mut manager = manager.lock().unwrap();
    let profile = manager.active_profile_mut();

    profile.refresh_config();

    Ok(profile
        .config
        .iter()
        .map(|res| match res {
            Ok(file) => FrontendLoadFileResult::Ok(file.clone()),
            Err(err) => FrontendLoadFileResult::Err {
                name: err.name.clone(),
                error: format!("{:#}", err.error),
            },
        })
        .collect())
}

#[tauri::command]
pub fn set_untagged_config_entry(
    file: &str,
    section: &str,
    entry: &str,
    value: String,
    manager: StateMutex<ModManager>,
) -> Result<()> {
    let mut manager = manager.lock().unwrap();

    manager
        .active_profile_mut()
        .modify_config(file, section, entry, move |entry| {
            *entry.as_untagged_mut()? = value;
            Ok(())
        })?;

    Ok(())
}

#[tauri::command]
pub fn set_tagged_config_entry(
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
            entry.as_tagged_mut()?.value = value;
            Ok(())
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

    let new_value = manager
        .active_profile_mut()
        .modify_config(file, section, entry, |entry| {
            let tagged = entry.as_tagged_mut()?;

            tagged.reset()?;
            Ok(tagged.value.clone())
        })?;

    Ok(new_value)
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
    let mut manager = manager.lock().unwrap();

    let profile = manager.active_profile_mut();
    let index = match profile.config.iter().position(|f| f.name() == file) {
        Some(index) => index,
        None => return Ok(()), // just ignore if the file doesn't exist
    };

    let file = profile.config.remove(index);
    let path = file.path_from(&profile.path);
    trash::delete(&path).context("failed to move file to trashcan")?;

    Ok(())
}
