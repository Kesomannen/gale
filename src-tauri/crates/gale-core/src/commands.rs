use crate::{
    community::{self, CommunityInfo},
    error::CmdResult,
    state::AppState,
    ResultExt,
};
use tauri::State;

#[tauri::command]
pub async fn get_communities(state: State<'_, AppState>) -> CmdResult<Vec<CommunityInfo>> {
    community::get_all(&state).await.map_into()
}
