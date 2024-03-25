use std::time::Duration;

use uuid::Uuid;
use anyhow::Context;

use crate::util;

use super::{models::PackageListing, query::{self, QueryModsArgs}, BorrowedMod, OwnedMod, ThunderstoreState};

type Result<T> = util::CommandResult<T>;

#[tauri::command]
pub async fn query_all_mods(
    args: QueryModsArgs,
    state: tauri::State<'_, ThunderstoreState>,
) -> Result<Vec<OwnedMod>> {
    wait_for_load(&state).await;

    let mod_list = state.all_mods.lock().unwrap();
    Ok(query::query_mods(
        args,
        mod_list.values().map(|package| BorrowedMod {
            package,
            version: &package.versions[0],
        }),
    ))
}

#[tauri::command]
pub async fn get_mod(
    full_name: String,
    state: tauri::State<'_, ThunderstoreState>,
) -> Result<PackageListing> {
    wait_for_load(&state).await;

    let all_mods = state.all_mods.lock().unwrap();
    super::find_package(&full_name, &all_mods).cloned().context("mod not found")
        .map_err(|e| e.into())
}

#[tauri::command]
pub async fn get_mod_by_id(
    id: Uuid,
    state: tauri::State<'_, ThunderstoreState>,
) -> Result<PackageListing> {
    wait_for_load(&state).await;

    let all_mods = state.all_mods.lock().unwrap();
    all_mods.get(&id).cloned().context("mod not found")
        .map_err(|e| e.into())
}

async fn wait_for_load(state: &tauri::State<'_, ThunderstoreState>) {
    while !*state.finished_loading.lock().unwrap() {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
