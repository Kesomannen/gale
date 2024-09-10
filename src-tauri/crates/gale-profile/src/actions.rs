use anyhow::{anyhow, ensure};
use gale_core::prelude::*;
use std::path::Path;

pub async fn create(name: &str, path: &Path, community_id: i64, state: &AppState) -> Result<i64> {
    ensure!(!path.exists(), "profile path already exists");

    let path_str = path
        .to_str()
        .ok_or(anyhow!("profile path must be valid utf-8"))?;

    let id = sqlx::query!(
        "INSERT INTO profiles (name, path, community_id) VALUES (?, ?, ?) RETURNING id",
        name,
        path_str,
        community_id
    )
    .fetch_one(&state.db)
    .await?
    .id;

    std::fs::create_dir_all(path)?;

    Ok(id)
}

pub async fn delete(id: i64, state: &AppState) -> Result<()> {
    let path = sqlx::query!("SELECT path FROM profiles WHERE id = ?", id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(anyhow!("profile not found"))?
        .path;

    std::fs::remove_dir_all(path)?;

    sqlx::query!("DELETE FROM profiles WHERE id = ?", id)
        .execute(&state.db)
        .await?;

    Ok(())
}

pub async fn rename(id: i64, name: &str, state: &AppState) -> Result<()> {
    sqlx::query!("UPDATE profiles SET name = ? WHERE id = ?", name, id)
        .execute(&state.db)
        .await?;

    Ok(())
}
