use crate::command_util::{Result, StateMutex};

use super::{
    models::FrontendMod,
    query::{self, ModSource, QueryModsArgs, QueryState},
    ModRef, Thunderstore,
};

#[tauri::command]
pub fn query_thunderstore(
    args: QueryModsArgs,
    thunderstore: StateMutex<Thunderstore>,
    state: StateMutex<QueryState>,
) -> Option<Vec<FrontendMod>> {
    let thunderstore = thunderstore.lock().unwrap();

    let mut state = state.lock().unwrap();

    if !thunderstore.finished_loading {
        state.query_passively = true;   
    }

    let result = query::query_frontend_mods(&args, thunderstore.latest());
    state.thunderstore_args = args;

    Some(result)
}

#[tauri::command]
pub fn stop_querying_thunderstore(state: StateMutex<QueryState>) {
    let mut state = state.lock().unwrap();
    state.query_passively = false;
}

#[tauri::command]
pub fn get_query_args(source: ModSource, query_state: StateMutex<QueryState>) -> QueryModsArgs {
    let query_state = query_state.lock().unwrap();
    match source {
        ModSource::Thunderstore => &query_state.thunderstore_args,
        ModSource::Profile => &query_state.profile_args,
    }.clone()
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
