use eyre::anyhow;
use serde::Deserialize;
use tauri::{AppHandle, Manager, Window};

use super::Prefs;
use crate::util::{
    cmd::{Result, StateMutex},
    window::WindowExt,
};

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
        }
        false => Ok(false),
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Zoom {
    Set { factor: f32 },
    Modify { delta: f32 },
}

#[tauri::command]
pub fn zoom_window(value: Zoom, prefs: StateMutex<Prefs>, window: Window) -> Result<()> {
    let mut prefs = prefs.lock().unwrap();
    prefs.zoom_factor = match value {
        Zoom::Set { factor } => factor,
        Zoom::Modify { delta } => prefs.zoom_factor + delta,
    }
    .clamp(0.5, 1.5);

    window
        .webview_windows()
        .values()
        .next()
        .unwrap()
        .zoom(prefs.zoom_factor as f64)
        .map_err(|err| anyhow!(err))?;

    prefs.save()?;

    Ok(())
}
