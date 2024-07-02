use crate::util::cmd::{Result, StateMutex};

use super::Prefs;
use tauri::AppHandle;

#[tauri::command]
pub fn get_prefs(prefs: StateMutex<Prefs>) -> Prefs {
    prefs.lock().unwrap().clone()
}

#[tauri::command]
pub fn set_prefs(value: Prefs, prefs: StateMutex<Prefs>, app: AppHandle) -> Result<()> {
    let mut prefs = prefs.lock().unwrap();
    prefs.set(value, &app)?;
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
