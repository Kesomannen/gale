use gale_core::prelude::*;
use std::path::{Path, PathBuf};

pub async fn root(state: &AppState) -> Result<PathBuf> {
    let cache_path = sqlx::query!("SELECT cache_path FROM settings")
        .fetch_one(&state.db)
        .await?
        .cache_path;

    Ok(cache_path.into())
}

pub async fn check(
    package_id: impl AsRef<Path>,
    subdir: impl AsRef<Path>,
    state: &AppState,
) -> Result<(PathBuf, bool)> {
    let mut path = root(state).await?;
    path.push(subdir);
    path.push(package_id);

    let exists = path.exists();
    Ok((path, exists))
}
