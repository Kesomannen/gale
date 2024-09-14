use crate::scan_mod;
use anyhow::{anyhow, ensure};
use gale_core::prelude::*;
use std::path::Path;
use walkdir::WalkDir;

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

pub async fn uninstall_mod(id: i64, state: &AppState) -> Result<()> {
    for path in scan_mod(id, state).await? {
        std::fs::remove_dir_all(path)?;
    }

    Ok(())
}

pub async fn toggle_mod(id: i64, state: &AppState) -> Result<()> {
    let old_state = sqlx::query!("SELECT enabled FROM profile_mods WHERE id = ?", id)
        .fetch_one(&state.db)
        .await?
        .enabled;

    let new_state = !old_state;

    for path in scan_mod(id, state).await? {
        let files = WalkDir::new(path)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_file());

        for file in files {
            let path = file.path();
            if new_state {
                if let Some(ext) = path.extension() {
                    if ext == "old" {
                        std::fs::rename(path, path.with_extension(""))?;
                    }
                }
            } else {
                let mut new = path.to_path_buf();
                new.add_extension("old");
                std::fs::rename(path, &new)?;
            }
        }
    }

    sqlx::query!(
        "UPDATE profile_mods SET enabled = ? WHERE id = ?",
        new_state,
        id
    )
    .execute(&state.db)
    .await?;

    Ok(())
}
