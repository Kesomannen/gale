use crate::{cache, Progress};
use anyhow::{anyhow, Context};
use futures_util::StreamExt;
use gale_core::prelude::*;
use gale_thunderstore::api::VersionId;
use sqlx::types::Uuid;
use std::{io::Cursor, path::Path, time::Instant};

pub async fn install<F>(
    version_uuid: Uuid,
    profile_path: &Path,
    mut handler: F,
    state: &AppState,
) -> Result<()>
where
    F: FnMut(Progress),
{
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
        let data = download(&id, version.file_size as u64, &mut handler, state)
            .await
            .context("failed to download package")?;

        handler(Progress::Extract);

        crate::common::extract(Cursor::new(data), id.full_name(), cache_path.clone())
            .context("failed to extract package")?;
    }

    handler(Progress::Install);

    crate::common::install(&cache_path, profile_path).context("failed to install package")?;

    log::info!(
        "installed {} in {}s (cache {})",
        id,
        start.elapsed().as_secs_f32(),
        if cache_hit { "hit" } else { "miss" }
    );

    Ok(())
}

async fn download<F>(
    id: &VersionId,
    total_size: u64,
    mut handler: F,
    state: &AppState,
) -> Result<Vec<u8>>
where
    F: FnMut(Progress),
{
    let mut stream = gale_thunderstore::api::download(&state.reqwest, id).await?;

    let mut vec = Vec::new();
    while let Some(chunk) = stream.next().await {
        vec.extend_from_slice(&chunk?);
        handler(Progress::Download {
            done: vec.len() as u64,
            total: total_size,
        });
    }

    Ok(vec)
}
