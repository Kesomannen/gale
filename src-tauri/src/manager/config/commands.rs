use serde::Serialize;
use typeshare::typeshare;

use crate::{
    manager::{self, ModManager},
    util::{self},
};

type Result<T> = util::CommandResult<T>;

#[typeshare]
#[derive(Serialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "content")]
pub enum GetConfigResult {
    Ok(super::File),
    Err { file: String, error: String },
}

#[tauri::command]
pub fn get_config_files(manager: tauri::State<ModManager>) -> Result<Vec<GetConfigResult>> {
    let mut profiles = manager.profiles.lock().unwrap();
    let active_profile = manager::get_active_profile(&mut profiles, &manager)?;

    active_profile.refresh_config();

    Ok(active_profile
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
    manager: tauri::State<ModManager>,
) -> Result<()> {
    let mut profiles = manager.profiles.lock().unwrap();
    let active_profile = manager::get_active_profile(&mut profiles, &manager)?;

    active_profile.modify_config(file, section, entry, move |entry| entry.value = value)?;
    Ok(())
}

#[tauri::command]
pub fn reset_config_entry(
    file: &str,
    section: &str,
    entry: &str,
    manager: tauri::State<ModManager>,
) -> Result<super::Value> {
    let mut profiles = manager.profiles.lock().unwrap();
    let active_profile = manager::get_active_profile(&mut profiles, &manager)?;

    active_profile.modify_config(file, section, entry, |entry| {
        entry.reset()?;
        Ok(entry.value.clone())
    })?
}
