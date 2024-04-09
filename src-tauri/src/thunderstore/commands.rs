use crate::command_util::{Result, StateMutex};

use super::{
    models::FrontendMod, query::{self, QueryModsArgs, QueryState}, Thunderstore
};

#[tauri::command]
pub fn query_all_mods(
    args: QueryModsArgs,
    thunderstore: StateMutex<Thunderstore>,
    query_state: StateMutex<QueryState>,
) -> Result<Option<Vec<FrontendMod>>> {
    let mut query_state = query_state.lock().unwrap();
    query_state.current_query = Some(args);
    Ok(None)
}
