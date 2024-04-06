use std::{fs, path::{Path, PathBuf}};

use anyhow::{anyhow, ensure};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{command_util::{Result, StateMutex}, util::IoResultExt};

use super::Prefs;

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

    fn path(name: &str, value: &Path) -> Self {
        Self::new(name, PrefValue::Path(value.to_path_buf()))
    }

    fn option_path(name: &str, value: Option<&PathBuf>) -> Self {
        Self::new(name, PrefValue::OptionPath(value.cloned()))
    }

    fn bool(name: &str, value: bool) -> Self {
        Self::new(name, PrefValue::Bool(value))
    }
}

#[tauri::command]
pub fn get_pref(name: String, prefs: StateMutex<Prefs>) -> Result<PrefEntry> {
    let prefs = prefs.lock().unwrap();

    match name.as_str() {
        "steam_exe_path" => Ok(PrefEntry::option_path("steam_exe_path", prefs.steam_exe_path.as_ref())),
        "cache_path" => Ok(PrefEntry::path("cache_path", &prefs.cache_path)),
        "temp_path" => Ok(PrefEntry::path("temp_path", &prefs.temp_path)),
        "data_path" => Ok(PrefEntry::path("data_path", &prefs.data_path)),
        _ => Err(anyhow!("config {} not found", name).into())
    }
}

#[tauri::command]
pub fn set_pref(entry: PrefEntry, prefs: StateMutex<Prefs>) -> Result<()> {
    let mut prefs = prefs.lock().unwrap();

    match entry.name.as_str() {
        "steam_exe_path" => prefs.steam_exe_path = entry.value.as_option_path()?,
        "cache_path" => move_dir(&mut prefs.cache_path, &entry.value, "cache")?,
        "temp_path" => move_dir(&mut prefs.temp_path, &entry.value, "temp")?,
        "data_path" => move_dir(&mut prefs.data_path, &entry.value, "data")?,
        _ => Err(anyhow!("config {} not found", entry.name))?
    };

    prefs.save()?;

    return Ok(());

    fn move_dir(current: &mut PathBuf, new: &PrefValue, name: &str) -> anyhow::Result<()> {
        let new_path = new.as_path()?;

        ensure!(new_path.exists(), "{} does not exist", new_path.display());
        ensure!(new_path.is_dir(), "{} is not a directory", new_path.display());
        ensure!(new_path.read_dir()?.count() == 0, "{} is not empty", new_path.display());

        fs::remove_dir_all(&new_path)
            .fs_context(&format!("removing {} dir", name), &new_path)?;

        fs::rename(&current, &new_path)
            .fs_context(&format!("moving {} dir", name), &new_path)?;

        *current = new_path;

        Ok(())
    }
}
