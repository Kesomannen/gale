use crate::command_util::{Result, StateMutex};

use super::{
    models::FrontendMod,
    query::{self, QueryModsArgs, QueryState},
    ModRef, Thunderstore,
};

#[tauri::command]
pub fn query_thunderstore(
    args: QueryModsArgs,
    thunderstore: StateMutex<Thunderstore>,
    state: StateMutex<QueryState>,
) -> Vec<FrontendMod> {
    let start = std::time::Instant::now();

    let thunderstore = thunderstore.lock().unwrap();
    
    let result = query::query_frontend_mods(&args, thunderstore.latest());
    
    if !thunderstore.finished_loading {
        let mut state = state.lock().unwrap();
        state.current_query = Some(args);
    }

    log::debug!(
        "query took {:?}ms, found {} mods",
        start.elapsed(),
        result.len()
    );

    result
}

#[tauri::command]
pub fn stop_querying_thunderstore(state: StateMutex<QueryState>) {
    let mut state = state.lock().unwrap();
    state.current_query = None;
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
