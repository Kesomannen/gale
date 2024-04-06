use crate::command_util::{Result, StateMutex};

use super::{
    query::{self, QueryModsArgs, QueryState},
    OwnedMod, Thunderstore,
};

#[tauri::command]
pub fn query_all_mods(
    args: QueryModsArgs,
    thunderstore: StateMutex<Thunderstore>,
    query_state: StateMutex<QueryState>,
) -> Result<Option<Vec<OwnedMod>>> {
    let thunderstore = thunderstore.lock().unwrap();

    match thunderstore.finished_loading {
        true => {
            Ok(Some(query::query_mods(
                &args,
                thunderstore.latest_versions(),
            )))
        }
        false => {
            let mut query_state = query_state.lock().unwrap();
            query_state.current_query = Some(args);
            Ok(None)
        }
    }
}
