use crate::command_util::StateMutex;

use super::{
    models::FrontendMod, query::{QueryModsArgs, QueryState}, Thunderstore
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
            Some(super::query::query_mods(&args, thunderstore.queryable()))
        },
        false => {
            let mut query_state = query_state.lock().unwrap();
            query_state.current_query = Some(args);
            None
        }
    }
}
