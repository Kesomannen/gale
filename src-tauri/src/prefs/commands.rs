use eyre::anyhow;
use serde::Deserialize;
use tauri::{command, AppHandle, Manager, Window};

use super::Prefs;
use crate::{
    state::ManagerExt,
    util::{cmd::Result, window::WindowExt},
};

#[command]
pub fn get_prefs(app: AppHandle) -> Prefs {
    app.lock_prefs().clone()
}

#[command]
pub fn set_prefs(value: Prefs, app: AppHandle) -> Result<()> {
    let mut prefs = app.lock_prefs();
    prefs.set(value, &app)?;
    Ok(())
}

#[command]
pub fn is_first_run(app: AppHandle) -> Result<bool> {
    let mut prefs = app.lock_prefs();
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

#[command]
pub fn zoom_window(value: Zoom, window: Window, app: AppHandle) -> Result<()> {
    let mut prefs = app.lock_prefs();
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
