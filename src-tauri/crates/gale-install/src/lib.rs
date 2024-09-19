use gale_core::prelude::*;

use anyhow::Context;
use gale_thunderstore::api::VersionId;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    path::{Path, PathBuf},
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

    pub fn package_id(&self) -> Cow<'_, str> {
        match self {
            InstallSource::Thunderstore { identifier, .. } => Cow::Borrowed(identifier.full_name()),
            InstallSource::Local { name, .. } => Cow::Borrowed(name),
            InstallSource::Github { owner, repo, .. } => Cow::Owned(format!("{}-{}", owner, repo)),
        }
    }

    pub fn name(&self) -> Cow<'_, str> {
        match self {
            InstallSource::Thunderstore { identifier, .. } => Cow::Borrowed(identifier.name()),
            InstallSource::Local { name, .. } => Cow::Borrowed(name),
            InstallSource::Github { repo, .. } => Cow::Borrowed(repo),
        }
    }
}

pub async fn install<F>(
    source: &InstallSource,
    profile_path: &Path,
    mut on_progress: F,
    state: &AppState,
) -> Result<()>
where
    F: FnMut(Progress),
{
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
                cache::insert_local(&path, cache_path.clone(), &name).await?;
            }
            InstallSource::Github { owner, repo, tag } => {
                cache::insert_github(&owner, &repo, &tag, cache_path.clone(), &state).await?;
            }
        }
    }

    on_progress(Progress::Install);

    common::install(&cache_path, profile_path)?;

    Ok(())
}
