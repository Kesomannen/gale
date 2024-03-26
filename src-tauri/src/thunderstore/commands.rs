use crate::util;

use super::{query::{self, QueryModsArgs}, OwnedMod, ThunderstoreState};

type Result<T> = util::CommandResult<T>;

#[tauri::command]
pub fn query_all_mods(
    args: QueryModsArgs,
    thunderstore: tauri::State<'_, ThunderstoreState>,
    query_state: tauri::State<'_, query::QueryState>,
) -> Result<Option<Vec<OwnedMod>>> {
    let finished_loading = thunderstore.finished_loading.lock().unwrap();

    match *finished_loading {
        true => {
            let packages = thunderstore.packages.lock().unwrap();
            Ok(Some(query::query_mods(&args, super::latest_versions(&packages))))
        }
        false => {
            let mut current_query = query_state.current_query.lock().unwrap();
            *current_query = Some(args);
            Ok(None)
        }
    }
}