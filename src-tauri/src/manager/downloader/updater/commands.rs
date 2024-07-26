use crate::{manager::ModManager, thunderstore::ModRef, util::cmd::Result};
use itertools::Itertools;
use serde::Deserialize;
use std::sync::Mutex;
use tauri::Manager;
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UpdateVersion {
    Latest,
    Specific(Uuid),
}

#[tauri::command]
pub async fn update_mod(uuid: Uuid, version: UpdateVersion, app: tauri::AppHandle) -> Result<()> {
    match version {
        UpdateVersion::Latest => {
            super::update_mods(&[uuid], &app).await?;
        }
        UpdateVersion::Specific(version_uuid) => {
            super::change_version(
                ModRef {
                    package_uuid: uuid,
                    version_uuid,
                },
                &app,
            )
            .await?;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn update_all(app: tauri::AppHandle) -> Result<()> {
    let uuids = {
        let manager = app.state::<Mutex<ModManager>>();
        let manager = manager.lock().unwrap();

        manager
            .active_profile()
            .remote_mods()
            .map(|(mod_ref, _, _)| mod_ref.package_uuid)
            .collect_vec()
    };

    super::update_mods(&uuids, &app).await?;

    Ok(())
}
