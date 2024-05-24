use std::fs;

use anyhow::{anyhow, bail, ensure};

use crate::{command_util::{Result, StateMutex}, fs_util, zoom_window};

use super::{PrefValue, Prefs};

#[tauri::command]
pub fn get_pref(key: &str, prefs: StateMutex<Prefs>) -> Result<PrefValue> {
    let prefs = prefs.lock().unwrap();
    let result = prefs.get_or_err(key)?;
    Ok(result.clone())
}

#[tauri::command]
pub fn set_pref(key: &str, value: PrefValue, prefs: StateMutex<Prefs>, window: tauri::Window) -> Result<()> {
    let mut prefs = prefs.lock().unwrap();

    match key {
        "data_dir" | "cache_dir" | "temp_dir" => move_dir(key, value, &mut prefs)?,
        "zoom_factor" => match value {
            PrefValue::Float(factor) => {
                zoom_window(&window, factor as f64).map_err(|e| anyhow!(e))?;
                prefs.set(key, value)?
            },
            _ => return Err(anyhow!("value is not a float").into())
        },
        _ => prefs.set(key, value)?
    };

    prefs.save()?;

    Ok(())
}

fn move_dir(key: &str, value: PrefValue, prefs: &mut Prefs) -> anyhow::Result<()> {
    let new_path = match value {
        PrefValue::Path(path) => path,
        _ => bail!("value is not a path")
    };

    let old_path = match prefs.get(key) {
        Some(PrefValue::Path(path)) => Some(path),
        _ => None,
    };

    ensure!(old_path != Some(&new_path), "{} is already set to {}", key, new_path.display());

    ensure!(new_path.exists(), "{} does not exist", new_path.display());
    ensure!(new_path.is_dir(), "{} is not a directory", new_path.display());
    ensure!(new_path.read_dir()?.count() == 0, "{} is not empty", new_path.display());
    
    if let Some(old_path) = old_path {
        fs_util::copy_dir(old_path, &new_path)?;
        fs::remove_dir_all(old_path)?;
    } else {
        fs::create_dir_all(&new_path)?;
    }

    prefs.set(key, PrefValue::Path(new_path))?;

    Ok(())
}
