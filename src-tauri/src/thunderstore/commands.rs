use crate::command_util::{Result, StateMutex};

use super::{
    models::FrontendMod,
    query::{self, QueryModsArgs, QueryState},
    ModRef, Thunderstore,
};

#[tauri::command]
pub fn query_all_mods(
    args: QueryModsArgs,
    thunderstore: StateMutex<Thunderstore>,
    query_state: StateMutex<QueryState>,
) -> Option<Vec<FrontendMod>> {
    let thunderstore = thunderstore.lock().unwrap();

    match thunderstore.finished_loading {
        true => Some(query::query_frontend_mods(&args, thunderstore.latest())),
        false => {
            let mut query_state = query_state.lock().unwrap();
            query_state.current_query = Some(args);
            None
        }
    }
}

#[tauri::command]
pub fn missing_deps(
    mod_ref: ModRef,
    thunderstore: StateMutex<Thunderstore>,
) -> Result<Vec<String>> {
    let thunderstore = thunderstore.lock().unwrap();

    let borrowed = mod_ref.borrow(&thunderstore)?;
    Ok(thunderstore
        .dependencies(borrowed.version)
        .1
        .into_iter()
        .map(String::from)
        .collect())
}
