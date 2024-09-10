use gale_core::prelude::*;
use gale_thunderstore::api::VersionId;
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use tauri::{
    generate_handler,
    plugin::{Builder, TauriPlugin},
    AppHandle, Emitter,
};

mod actions;
mod commands;
mod query;

pub fn init() -> TauriPlugin<tauri::Wry> {
    Builder::new("gale-profile")
        .setup(|app, _| Ok(()))
        .invoke_handler(generate_handler![
            commands::create,
            commands::delete,
            commands::rename,
            commands::query,
        ])
        .build()
}

#[derive(Serialize, Deserialize, Debug)]
enum ProfileModSource {
    Thunderstore {
        identifier: VersionId,
        version_uuid: Uuid,
    },
    Local {
        id: i64,
    },
}

async fn emit_update(id: i64, state: &AppState, app: &AppHandle) -> Result<()> {
    let info = query::single(id, state).await?;
    app.emit("profile-update", info)?;
    Ok(())
}

fn emit_update_spawn(id: i64, app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let state = app.app_state();
        if let Err(err) = emit_update(id, &state, &app).await {
            log::error!("failed to emit profile update: {:#}", err);
        }
    });
}
