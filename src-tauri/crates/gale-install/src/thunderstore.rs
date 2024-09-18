use crate::cache;
use anyhow::{anyhow, Context};
use futures_util::StreamExt;
use gale_core::prelude::*;
use gale_thunderstore::api::VersionId;
use sqlx::types::Uuid;
use std::{io::Cursor, path::Path, time::Instant};

pub async fn install(version_uuid: Uuid, profile_path: &Path, state: &AppState) -> Result<()> {
    let start = Instant::now();

    let version = sqlx::query!(
        "SELECT
            v.major,
            v.minor,
            v.patch,
            v.file_size,
            p.name,
            p.owner
        FROM
            versions v
            JOIN packages p ON
                p.id = v.package_id
        WHERE
            v.id = ?
        ",
        version_uuid
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| anyhow!("version not found: {}", version_uuid))?;

    let id: VersionId = (
        &version.owner,
        &version.name,
        version.major as u32,
        version.minor as u32,
        version.patch as u32,
    )
        .into();

    let (cache_path, cache_hit) = cache::check(id.as_str(), "thunderstore", state)
        .await
        .context("failed to check cache")?;

    if !cache_hit {
        let data = download(&id, state)
            .await
            .context("failed to download package")?;

        crate::common::extract(Cursor::new(data), id.full_name(), cache_path.clone())
            .context("failed to extract package")?;
    }

    crate::common::install(&cache_path, profile_path).context("failed to install package")?;

    log::info!(
        "installed {} in {}s (cache {})",
        id,
        start.elapsed().as_secs_f32(),
        if cache_hit { "hit" } else { "miss" }
    );

    Ok(())
}

async fn download(id: &VersionId, state: &AppState) -> Result<Vec<u8>> {
    let mut stream = gale_thunderstore::api::download(&state.reqwest, id).await?;

    let mut vec = Vec::new();
    while let Some(chunk) = stream.next().await {
        vec.extend_from_slice(&chunk?);
    }

    Ok(vec)
}
