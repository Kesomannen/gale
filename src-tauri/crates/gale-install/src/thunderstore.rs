use crate::{cache, common};
use anyhow::Context;
use futures_util::StreamExt;
use gale_core::prelude::*;
use gale_thunderstore::api::VersionId;
use sqlx::types::Uuid;
use std::{io::Cursor, path::Path};

pub async fn install(
    version_uuid: Uuid,
    profile_path: &Path,
    state: &AppState,
) -> Result<VersionId> {
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
    .context("version not found")?;

    let id: VersionId = (
        &version.owner,
        &version.name,
        version.major,
        version.minor,
        version.patch,
    )
        .into();

    let mut cache_path = cache::path(&state).await?;
    cache_path.push(id.full_name());
    cache_path.push(id.version());

    if !cache_path.exists() {
        let data = download(&id, state)
            .await
            .context("failed to download package")?;

        common::extract(Cursor::new(data), id.full_name(), cache_path.clone())
            .context("failed to extract package")?;
    }

    common::install(&cache_path, profile_path).context("failed to install package")?;

    Ok(id)
}

async fn download(id: &VersionId, state: &AppState) -> Result<Vec<u8>> {
    let mut stream = gale_thunderstore::api::download(&state.reqwest, id).await?;

    let mut vec = Vec::new();
    while let Some(chunk) = stream.next().await {
        vec.extend_from_slice(&chunk?);
    }

    Ok(vec)
}
