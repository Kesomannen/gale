use gale_core::prelude::*;

use anyhow::Context;
use bytes::Bytes;
use futures_util::{Stream, TryStreamExt};
use gale_thunderstore::api::VersionId;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use uuid::Uuid;

mod cache;
mod common;

pub enum Progress {
    Install,
    Extract,
    Download { current: u64, total: u64 },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum InstallSource {
    #[serde(rename_all = "camelCase")]
    Thunderstore {
        identifier: VersionId,
        version_uuid: Uuid,
    },
    #[serde(rename_all = "camelCase")]
    Local {
        path: PathBuf,
        name: String,
        version: String,
    },
    #[serde(rename_all = "camelCase")]
    Github {
        owner: String,
        repo: String,
        tag: String,
    },
}

impl InstallSource {
    /// Unique identifier for this specific version of the package
    pub fn version_id(&self) -> Cow<'_, str> {
        match self {
            InstallSource::Thunderstore { identifier, .. } => Cow::Borrowed(identifier.as_str()),
            InstallSource::Local { name, version, .. } => {
                Cow::Owned(format!("{}-{}", name, version))
            }
            InstallSource::Github { owner, repo, tag } => {
                Cow::Owned(format!("{}-{}-{}", owner, repo, tag))
            }
        }
    }

    /// Unique identifier for the package itself
    pub fn package_id(&self) -> Cow<'_, str> {
        match self {
            InstallSource::Thunderstore { identifier, .. } => Cow::Borrowed(identifier.full_name()),
            InstallSource::Local { name, .. } => Cow::Borrowed(name),
            InstallSource::Github { owner, repo, .. } => Cow::Owned(format!("{}-{}", owner, repo)),
        }
    }

    /// Display name for the package. Not guaranteed to be unique!
    pub fn name(&self) -> Cow<'_, str> {
        match self {
            InstallSource::Thunderstore { identifier, .. } => Cow::Borrowed(identifier.name()),
            InstallSource::Local { name, .. } => Cow::Borrowed(name),
            InstallSource::Github { repo, .. } => Cow::Borrowed(repo),
        }
    }
}

pub async fn install(
    source: &InstallSource,
    profile_path: &Path,
    mut on_progress: impl FnMut(Progress),
    state: &AppState,
) -> Result<()> {
    let version_id = source.version_id();

    let subdir = match source {
        InstallSource::Thunderstore { .. } => "thunderstore",
        InstallSource::Local { .. } => "local",
        InstallSource::Github { .. } => "github",
    };

    let (cache_path, cache_hit) = cache::check(version_id.as_ref(), subdir, &state)
        .await
        .context("failed to check cache")?;

    if !cache_hit {
        match source {
            InstallSource::Thunderstore { identifier, .. } => {
                cache::insert_thunderstore(
                    &identifier,
                    cache_path.clone(),
                    &mut on_progress,
                    &state,
                )
                .await?;
            }
            InstallSource::Local { path, name, .. } => {
                cache::insert_local(&path, cache_path.clone(), &name, &mut on_progress).await?;
            }
            InstallSource::Github { owner, repo, tag } => {
                cache::insert_github(
                    &owner,
                    &repo,
                    &tag,
                    cache_path.clone(),
                    &mut on_progress,
                    &state,
                )
                .await?;
            }
        }
    }

    on_progress(Progress::Install);

    common::install(&cache_path, profile_path)?;

    Ok(())
}

async fn stream_download_res(
    response: reqwest::Response,
    on_progress: impl FnMut(Progress),
) -> Result<Vec<u8>> {
    stream_download(
        response.content_length().unwrap_or_default(),
        on_progress,
        response.bytes_stream(),
    )
    .await
}

async fn stream_download<E>(
    len: u64,
    mut on_progress: impl FnMut(Progress),
    mut stream: impl Stream<Item = std::result::Result<Bytes, E>> + Unpin,
) -> Result<Vec<u8>>
where
    E: std::error::Error + Send + Sync + 'static,
{
    const UPDATE_INTERVAL: Duration = Duration::from_millis(500);

    let mut last_update = Instant::now();
    let mut vec = Vec::with_capacity(len as usize);

    while let Some(chunk) = stream.try_next().await? {
        vec.extend_from_slice(&chunk);

        if last_update.elapsed() >= UPDATE_INTERVAL {
            last_update = Instant::now();
            on_progress(Progress::Download {
                current: vec.len() as u64,
                total: len,
            });
        }
    }

    Ok(vec)
}
