use eyre::eyre;
use serde::Deserialize;
use tauri::{AppHandle, Manager, Window, command, window::Color};

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
        .zoom(f64::from(prefs.zoom_factor))
        .map_err(|err| eyre!(err))?;

    prefs.save(app.db())?;

    Ok(())
}

#[command]
pub fn get_system_fonts() -> Result<Vec<String>> {
    font_kit::source::SystemSource::new()
        .all_families()
        .map_err(|err| eyre!(err).into())
}

#[command]
pub async fn get_system_accent_color() -> Result<Option<Color>> {
    crate::util::color::system_accent()
        .await
        .map_err(|err| eyre!(err).into())
}
