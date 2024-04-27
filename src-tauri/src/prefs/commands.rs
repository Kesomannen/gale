use std::fs;

use anyhow::{anyhow, ensure};

use crate::{command_util::{Result, StateMutex}, util::IoResultExt};

use super::{PrefValue, Prefs};

#[tauri::command]
pub fn get_pref(key: &str, prefs: StateMutex<Prefs>) -> Result<PrefValue> {
    let prefs = prefs.lock().unwrap();
    let result = prefs.get_or_err(key)?;
    Ok(result.clone())
}

#[tauri::command]
pub fn set_pref(key: &str, value: PrefValue, prefs: StateMutex<Prefs>) -> Result<()> {
    let mut prefs = prefs.lock().unwrap();

    match key {
        "data_dir" | "cache_dir" | "temp_dir" => move_dir(key, value, &mut prefs)?,
        _ => prefs.set(key, value)?
    };

    prefs.save()?;

    Ok(())
}

fn move_dir(key: &str, value: PrefValue, prefs: &mut Prefs) -> anyhow::Result<()> {
    let new_path = match value {
        PrefValue::Path(path) => path,
        _ => return Err(anyhow!("value is not a path"))
    };

    ensure!(new_path.exists(), "{} does not exist", new_path.display());
    ensure!(new_path.is_dir(), "{} is not a directory", new_path.display());
    ensure!(new_path.read_dir()?.count() == 0, "{} is not empty", new_path.display());

    fs::remove_dir_all(&new_path)
        .fs_context(&format!("removing {} dir", key), &new_path)?;

    if let Some(PrefValue::Path(old_path)) = prefs.get(key) {
        fs::rename(old_path, &new_path)
            .fs_context(&format!("moving {} dir", key), &new_path)?;
    }

    prefs.set(key, PrefValue::Path(new_path))?;

    Ok(())
}
