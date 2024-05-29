use crate::command_util::{Result, StateMutex};

use super::{
    models::FrontendMod,
    query::{self, QueryModsArgs, QueryState},
    ModRef, Thunderstore,
};

#[tauri::command]
pub fn query_all_mods(
    args: Option<QueryModsArgs>,
    thunderstore: StateMutex<Thunderstore>,
    query_state: StateMutex<QueryState>,
) -> Option<Vec<FrontendMod>> {
    let thunderstore = thunderstore.lock().unwrap();

    match (args, thunderstore.finished_loading) {
        (Some(args), true) => Some(query::query_frontend_mods(&args, thunderstore.latest())),
        (args, _) => {
            let mut query_state = query_state.lock().unwrap();
            query_state.current_query = args;
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
