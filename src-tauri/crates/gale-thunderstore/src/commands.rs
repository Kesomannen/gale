use crate::query;
use gale_core::prelude::*;
use uuid::Uuid;

#[tauri::command]
pub async fn query_packages(
    args: query::QueryArgs,
    state: tauri::State<'_, AppState>,
) -> std::result::Result<Vec<query::Package>, String> {
    query::query_packages(args, &state)
        .await
        .map_err(|err| format!("{err:#}"))
}

#[tauri::command]
pub async fn query_package(
    id: Uuid,
    state: tauri::State<'_, AppState>,
) -> std::result::Result<query::PackageInfo, String> {
    query::query_package(id, &state)
        .await
        .map_err(|err| format!("{err:#}"))
}
