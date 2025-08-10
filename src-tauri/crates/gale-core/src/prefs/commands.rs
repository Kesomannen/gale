use eyre::anyhow;
use font_kit::source::SystemSource;
use gale_util::{cmd::Result, window::WindowExt};
use serde::Deserialize;
use tauri::{AppHandle, Manager, Window, command};

use super::Prefs;

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
        .get_webview_window("main")
        .unwrap()
        .zoom(prefs.zoom_factor as f64)
        .map_err(|err| anyhow!(err))?;

    prefs.save(&app.state())?;

    Ok(())
}

#[command]
pub fn get_system_fonts() -> Result<Vec<String>> {
    let fonts = SystemSource::new().all_families().unwrap();

    Ok(fonts)
}
