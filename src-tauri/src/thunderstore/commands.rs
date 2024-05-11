use crate::command_util::StateMutex;

use super::{
    models::FrontendMod, query::{self, QueryModsArgs, QueryState}, Thunderstore
};

#[tauri::command]
pub fn query_all_mods(
    args: QueryModsArgs,
    thunderstore: StateMutex<Thunderstore>,
    query_state: StateMutex<QueryState>,
) -> Option<Vec<FrontendMod>> {
    let thunderstore = thunderstore.lock().unwrap();

    match thunderstore.finished_loading {
        true => {
            Some(query::query_frontend_mods(&args, thunderstore.latest()))
        },
        false => {
            let mut query_state = query_state.lock().unwrap();
            query_state.current_query = Some(args);
            None
        }
    }
}
