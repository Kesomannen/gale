use crate::{
    manager::ModManager,
    prefs::Prefs,
    util::cmd::{Result, StateMutex},
};
use anyhow::{anyhow, Context};
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;

#[tauri::command]
pub fn launch_game(
    vanilla: bool,
    manager: StateMutex<ModManager>,
    prefs: StateMutex<Prefs>,
) -> Result<()> {
    let manager = manager.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    manager.active_game().launch(vanilla, &prefs)?;
    Ok(())
}

#[tauri::command]
pub fn copy_launch_args(app: AppHandle, manager: StateMutex<ModManager>) -> Result<()> {
    let manager = manager.lock().unwrap();

    let path = &manager.active_profile().path;

    let (enable_prefix, target_prefix) = super::get_doorstop_args(path)?;

    let preloader_path = super::find_preloader(path)?;
    let preloader_path = preloader_path
        .to_str()
        .ok_or(anyhow!("profile path contains invalid UTF-8"))?;

    let text = format!(
        r#"{} {} {} "{}""#,
        enable_prefix, "true", target_prefix, preloader_path
    );

    app.clipboard()
        .write_text(text)
        .context("failed to copy launch args")?;

    Ok(())
}
