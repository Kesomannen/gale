use anyhow::anyhow;
use gale_core::prelude::*;
use std::path::PathBuf;

pub async fn launch(profile_id: i64, state: &AppState) -> Result<()> {
    let (profile_path, community_id) = sqlx::query!(
        "SELECT path, community_id
        FROM profiles
        WHERE id = ?",
        profile_id
    )
    .map(|record| (PathBuf::from(record.path), record.community_id))
    .fetch_one(&state.db)
    .await?;

    let (game_path, steam_id) = game_info(community_id, state).await?;

    Ok(())
}

async fn game_info(community_id: i64, state: &AppState) -> Result<(PathBuf, i64)> {
    let (path_override, steam_dir_name, steam_id) = sqlx::query!(
        "SELECT override_path, steam_dir_name, steam_id
        FROM communities
        WHERE id = ?",
        community_id
    )
    .map(|record| (record.override_path, record.steam_dir_name, record.steam_id))
    .fetch_one(&state.db)
    .await?;

    if let Some(path) = path_override {
        Ok((PathBuf::from(path), steam_id))
    } else {
        let mut path: PathBuf = sqlx::query!("SELECT steam_library_path FROM settings")
            .fetch_one(&state.db)
            .await?
            .steam_library_path
            .ok_or(anyhow!("steam library path not set"))?
            .into();

        path.push("steamapps");
        path.push("common");
        path.push(steam_dir_name);

        Ok((path, steam_id))
    }
}
