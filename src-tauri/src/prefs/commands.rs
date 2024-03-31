use std::{fs, path::PathBuf};

use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::util;

use super::PrefsState;

type Result<T> = util::CommandResult<T>;

#[typeshare]
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "content")]
pub enum PrefValue {
    Path(PathBuf),
    OptionPath(Option<PathBuf>),
    Bool(bool)
}

impl PrefValue {
    fn as_path(&self) -> Result<PathBuf> {
        match self {
            PrefValue::Path(path) => Ok(path.clone()),
            _ => Err(anyhow!("value is not a path").into()),
        }
    }

    fn as_option_path(&self) -> Result<Option<PathBuf>> {
        match self {
            PrefValue::OptionPath(path) => Ok(path.clone()),
            PrefValue::Path(path) => Ok(Some(path.clone())),
            _ => Err(anyhow!("value is not an option path").into()),
        }
    }
    
    fn as_bool(&self) -> Result<bool> {
        match self {
            PrefValue::Bool(b) => Ok(*b),
            _ => Err(anyhow!("value is not a bool").into()),
        }
    }
}

#[typeshare]
#[derive(Serialize, Deserialize, Debug)]
pub struct PrefEntry {
    name: String,
    value: PrefValue,
}

impl PrefEntry {
    fn new(name: &str, value: PrefValue) -> Self {
        Self {
            name: name.to_string(),
            value,
        }
    }

    fn path(name: &str, value: &PathBuf) -> Self {
        Self::new(name, PrefValue::Path(value.clone()))
    }

    fn option_path(name: &str, value: Option<&PathBuf>) -> Self {
        Self::new(name, PrefValue::OptionPath(value.cloned()))
    }

    fn bool(name: &str, value: bool) -> Self {
        Self::new(name, PrefValue::Bool(value))
    }
}

#[tauri::command]
pub fn get_pref(name: String, prefs: tauri::State<PrefsState>) -> Result<PrefEntry> {
    let config = prefs.lock();

    match name.as_str() {
        "steam_exe_path" => Ok(PrefEntry::option_path("steam_exe_path", config.steam_exe_path.as_ref())),
        "cache_path" => Ok(PrefEntry::path("cache_path", &config.cache_path)),
        "data_path" => Ok(PrefEntry::path("data_path", &config.data_path)),
        "auto_start" => Ok(PrefEntry::bool("auto_start", config.auto_start)),
        _ => Err(anyhow!("config {} not found", name).into())
    }
}

#[tauri::command]
pub fn set_pref(entry: PrefEntry, prefs: tauri::State<PrefsState>) -> Result<()> {
    inner(entry, &prefs)?;
    prefs.save()?;
    return Ok(());

    fn inner(entry: PrefEntry, prefs: &PrefsState) -> Result<()> {
        let mut config = prefs.lock();
        let value = entry.value;
        match entry.name.as_str() {
            "steam_exe_path" => config.steam_exe_path = value.as_option_path()?,
            "cache_path" => {
                let path = value.as_path()?;
                fs::rename(&config.cache_path, &path)
                    .context("failed to move cache")?;
                config.cache_path = path;
            },
            "data_path" => {
                let path = value.as_path()?;
                fs::rename(&config.data_path, &path)
                    .context("failed to move data")?;
                config.data_path = path;
            },
            "auto_start" => {
                config.auto_start = value.as_bool()?
            },
            _ => Err(anyhow!("config {} not found", entry.name))?
        };

        Ok(())
    }
}
