use crate::{emit_update, ProfileModSource};
use anyhow::Context;
use gale_core::prelude::*;
use gale_thunderstore::api::VersionId;
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    fmt::{Debug, Display},
    path::PathBuf,
    sync::Mutex,
};
use tauri::{AppHandle, Manager};
use tokio::sync::Notify;
use uuid::Uuid;

#[derive(Debug)]
pub struct InstallMetadata {
    enabled: bool,
    index: Option<usize>,
    profile_id: i64,
}

impl InstallMetadata {
    pub fn new(profile_id: i64) -> Self {
        Self {
            enabled: true,
            index: None,
            profile_id,
        }
    }

    pub fn with_state(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn with_index(mut self, index: usize) -> Self {
        self.index = Some(index);
        self
    }
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
        full_name: String,
        version: String,
    },
    #[serde(rename_all = "camelCase")]
    Github {
        owner: String,
        repo: String,
        tag: String,
    },
}

impl From<InstallSource> for ProfileModSource {
    fn from(source: InstallSource) -> Self {
        match source {
            InstallSource::Thunderstore {
                identifier,
                version_uuid,
            } => ProfileModSource::Thunderstore {
                identifier,
                version_uuid,
            },
            InstallSource::Local {
                full_name, version, ..
            } => ProfileModSource::Local { full_name, version },
            InstallSource::Github { owner, repo, tag } => {
                ProfileModSource::Github { owner, repo, tag }
            }
        }
    }
}

impl Display for InstallSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstallSource::Thunderstore { identifier, .. } => {
                write!(f, "{} (thunderstore)", identifier)
            }
            InstallSource::Local {
                full_name,
                version,
                path,
            } => {
                write!(
                    f,
                    "{}-{} (local from {})",
                    full_name,
                    version,
                    path.display()
                )
            }
            InstallSource::Github { owner, repo, tag } => {
                write!(f, "{}-{}-{} (github)", owner, repo, tag)
            }
        }
    }
}

pub struct InstallQueue {
    notify: Notify,
    queue: Mutex<VecDeque<(InstallSource, InstallMetadata)>>,
}

impl InstallQueue {
    pub(crate) fn new() -> Self {
        Self {
            notify: Notify::new(),
            queue: Mutex::new(VecDeque::new()),
        }
    }

    pub fn enqueue(&self, source: InstallSource, metadata: InstallMetadata) -> usize {
        self.enqueue_many(std::iter::once((source, metadata)))
    }

    pub fn enqueue_many<I>(&self, items: I) -> usize
    where
        I: IntoIterator<Item = (InstallSource, InstallMetadata)>,
    {
        let mut queue = self.queue.lock().unwrap();
        queue.extend(items);
        self.notify.notify_one();

        queue.len()
    }

    pub fn with_lock<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&mut VecDeque<(InstallSource, InstallMetadata)>) -> T,
    {
        f(&mut self.queue.lock().unwrap())
    }
}

pub(crate) async fn handler(app: AppHandle) {
    let state = app.state::<AppState>();
    let queue = app.state::<InstallQueue>();

    loop {
        let next = queue.queue.lock().unwrap().pop_front();

        let (source, metadata) = match next {
            Some(item) => item,
            None => {
                queue.notify.notified().await;
                continue;
            }
        };

        if let Err(err) = handle_request(source, &metadata, &state, &app).await {
            log::error!("failed to install mod: {:#}", err);
        }
    }
}

async fn handle_request(
    source: InstallSource,
    metadata: &InstallMetadata,
    state: &AppState,
    app: &AppHandle,
) -> Result<()> {
    let profile_path: PathBuf = sqlx::query!(
        "SELECT path FROM profiles WHERE id = ?",
        metadata.profile_id
    )
    .fetch_optional(&state.db)
    .await?
    .context("profile not found")?
    .path
    .into();

    log::debug!("installing {} at {}", source, profile_path.display());

    match &source {
        InstallSource::Thunderstore { version_uuid, .. } => {
            gale_install::from_thunderstore(*version_uuid, &profile_path, state).await?;
        }
        InstallSource::Local {
            path,
            full_name,
            version,
        } => {
            gale_install::from_local(path, full_name, version, &profile_path, state).await?;
        }
        InstallSource::Github { owner, repo, tag } => {
            gale_install::from_github(owner, repo, tag, &profile_path, state).await?;
        }
    };

    let source = ProfileModSource::from(source);
    let json = sqlx::types::Json(source);

    let index = match metadata.index {
        Some(index) => index as i64,
        None => {
            let max_index = sqlx::query!(
                r#"SELECT MAX(order_index) AS "max: i64"
                FROM profile_mods
                WHERE profile_id = ?"#,
                metadata.profile_id
            )
            .fetch_optional(&state.db)
            .await?
            .and_then(|row| row.max)
            .unwrap_or(0);

            max_index + 1
        }
    };

    sqlx::query!(
        "INSERT INTO profile_mods
        (profile_id, enabled, order_index, source)
        VALUES (?, ?, ?, ?)",
        metadata.profile_id,
        metadata.enabled,
        index,
        json
    )
    .execute(&state.db)
    .await?;

    emit_update(metadata.profile_id, state, &app).await?;

    Ok(())
}
