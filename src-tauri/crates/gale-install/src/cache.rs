use gale_core::prelude::*;
use std::path::PathBuf;

pub async fn path(state: &AppState) -> Result<PathBuf> {
    let cache_path = sqlx::query!("SELECT cache_path FROM settings")
        .fetch_one(&state.db)
        .await?
        .cache_path;

    Ok(cache_path.into())
}
