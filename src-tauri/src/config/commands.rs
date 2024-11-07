use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::Serialize;

use super::{File, LoadFileResultExt};
use crate::{
    profile::ModManager,
    util::cmd::{Result, StateMutex},
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum FrontendLoadFileResult {
    Ok(File),
    Err {
        #[serde(rename = "displayName")]
        display_name: String,
        #[serde(rename = "relativePath")]
        relative_path: PathBuf,
        error: String,
    },
    Unsupported {
        #[serde(rename = "relativePath")]
        relative_path: PathBuf,
    },
}

#[tauri::command]
pub fn get_config_files(manager: StateMutex<ModManager>) -> Result<Vec<FrontendLoadFileResult>> {
    let mut manager = manager.lock().unwrap();
    let profile = manager.active_profile_mut();

    let other_files = profile.refresh_config();

    Ok(profile
        .config
        .iter()
        .map(|res| match res {
            Ok(file) => FrontendLoadFileResult::Ok(file.clone()),
            Err(err) => FrontendLoadFileResult::Err {
                display_name: err.display_name.to_string(),
                relative_path: err.relative_path.clone(),
                error: format!("{:#}", err.error),
            },
        })
        .chain(
            other_files
                .into_iter()
                .map(|relative_path| FrontendLoadFileResult::Unsupported { relative_path }),
        )
        .collect())
}

#[tauri::command]
pub fn set_config_entry(
    file: &Path,
    section: &str,
    entry: &str,
    value: super::Value,
    manager: StateMutex<ModManager>,
) -> Result<()> {
    let mut manager = manager.lock().unwrap();

    manager
        .active_profile_mut()
        .modify_config(file, section, entry, move |entry| {
            entry.as_normal_mut()?.value = value;
            Ok(())
        })?;

    Ok(())
}

#[tauri::command]
pub fn reset_config_entry(
    file: &Path,
    section: &str,
    entry: &str,
    manager: StateMutex<ModManager>,
) -> Result<super::Value> {
    let mut manager = manager.lock().unwrap();

    let new_value = manager
        .active_profile_mut()
        .modify_config(file, section, entry, |entry| {
            let tagged = entry.as_normal_mut()?;

            tagged.reset()?;
            Ok(tagged.value.clone())
        })?;

    Ok(new_value)
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

    let is_cfg = file
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext == "cfg");

    if is_cfg {
        let Some(index) = profile
            .config
            .iter()
            .position(|f| f.relative_path() == file)
        else {
            return Ok(()); // ignore if the file is not in the list
        };

        profile.config.remove(index).ok();
    }

    let path = profile.path.join(file);
    trash::delete(path).context("failed to move file to recycle bin")?;

    Ok(())
}
