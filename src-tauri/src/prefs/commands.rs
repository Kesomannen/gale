use anyhow::anyhow;

use crate::{command_util::{Result, StateMutex}, zoom_window};

use super::{PrefValue, Prefs};

#[tauri::command]
pub fn get_pref(key: &str, prefs: StateMutex<Prefs>) -> Result<Option<PrefValue>> {
    let prefs = prefs.lock().unwrap();
    let result = prefs.get(key);
    Ok(result.cloned())
}

#[tauri::command]
pub fn set_pref(key: &str, value: PrefValue, prefs: StateMutex<Prefs>, window: tauri::Window) -> Result<()> {
    let mut prefs = prefs.lock().unwrap();

    match key {
        "data_dir" | "cache_dir" | "temp_dir" => prefs.move_dir(key, value)?,
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

#[tauri::command]
pub fn is_first_run(prefs: StateMutex<Prefs>) -> Result<bool> {
    let mut prefs = prefs.lock().unwrap();
    match prefs.is_first_run {
        true => {
            prefs.is_first_run = false;
            Ok(true)
        },
        false => Ok(false)
    }
}
