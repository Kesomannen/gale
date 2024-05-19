use uuid::Uuid;
use crate::{command_util::Result, manager::ModManager};
use std::sync::Mutex;
use tauri::Manager;
use itertools::Itertools;

#[tauri::command]
pub async fn update_mod(
    uuid: Uuid,
    app: tauri::AppHandle,
) -> Result<()> {
    super::update_mods(&[uuid], &app).await?;

    Ok(())
}

#[tauri::command]
pub async fn update_all(app: tauri::AppHandle) -> Result<()> {
    let uuids = {
        let manager = app.state::<Mutex<ModManager>>();
        let manager = manager.lock().unwrap();

        manager.active_profile().remote_mods().map(|(m, _)| m.package_uuid).collect_vec()
    };

    super::update_mods(&uuids, &app).await?;

    Ok(())
}