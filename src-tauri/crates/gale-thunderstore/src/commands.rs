use crate::query;
use gale_core::prelude::*;
use gale_core::ResultExt;
use uuid::Uuid;

#[tauri::command]
pub async fn query_packages(
    args: query::QueryArgs,
    state: tauri::State<'_, AppState>,
) -> CmdResult<Vec<query::ListedPackageInfo>> {
    query::query_packages(args, &state).await.map_into()
}

#[tauri::command]
pub async fn query_package(
    id: Uuid,
    state: tauri::State<'_, AppState>,
) -> CmdResult<query::PackageInfo> {
    query::query_package(id, &state).await.map_into()
}
