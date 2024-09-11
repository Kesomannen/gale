use gale_core::prelude::*;
use gale_thunderstore::api::VersionId;
use sqlx::types::Uuid;
use std::path::Path;
use tauri::{
    generate_handler,
    plugin::{Builder, TauriPlugin},
};

mod cache;
mod commands;
mod common;
mod local;
mod thunderstore;

pub fn init() -> TauriPlugin<tauri::Wry> {
    Builder::new("gale-install")
        .invoke_handler(generate_handler![])
        .build()
}

pub async fn from_thunderstore(
    version_uuid: Uuid,
    profile_path: &Path,
    state: &AppState,
) -> Result<VersionId> {
    thunderstore::install(version_uuid, profile_path, state).await
}
