use crate::util::cmd::{Result, StateMutex};

use super::{PrefValue, Prefs};
use log::debug;

#[tauri::command]
pub fn get_pref(key: &str, prefs: StateMutex<Prefs>) -> Result<Option<PrefValue>> {
    let prefs = prefs.lock().unwrap();
    let result = prefs.get(key);
    Ok(result.cloned())
}

#[tauri::command]
pub fn set_pref(key: &str, value: PrefValue, prefs: StateMutex<Prefs>, window: tauri::Window) -> Result<()> {
    let mut prefs = prefs.lock().unwrap();

    debug!("setting pref {} to {:?}", key, value);

    prefs.set(key, value, Some(&window))?;

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
