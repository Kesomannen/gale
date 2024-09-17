use crate::{
    error::CmdResult,
    game::{self, GameInfo},
    state::AppState,
    ResultExt,
};
use tauri::State;

#[tauri::command]
pub async fn get_games(state: State<'_, AppState>) -> CmdResult<Vec<GameInfo>> {
    game::get_all(&state).await.map_into()
}
