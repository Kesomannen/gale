use crate::ProfileModSource;
use gale_core::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, path::PathBuf, sync::Mutex};
use tauri::{AppHandle, Manager};
use tokio::sync::Notify;
use uuid::Uuid;

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

#[derive(Debug, Serialize, Deserialize)]
pub enum InstallSource {
    Thunderstore(Uuid),
    Local(PathBuf),
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

    pub fn enqueue(&self, source: InstallSource, metadata: InstallMetadata) -> Result<usize> {
        let mut queue = self.queue.lock().unwrap();
        queue.push_back((source, metadata));
        self.notify.notify_one();

        Ok(queue.len())
    }

    pub fn with_lock<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&mut VecDeque<(InstallSource, InstallMetadata)>) -> T,
    {
        f(&mut self.queue.lock().unwrap())
    }
}

pub fn enqueue(source: InstallSource, metadata: InstallMetadata, app: &AppHandle) -> Result<usize> {
    app.state::<InstallQueue>().enqueue(source, metadata)
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

        if let Err(err) = handle_request(source, &metadata, &state).await {
            log::error!("failed to install mod: {:#}", err);
        }
    }
}

async fn handle_request(
    source: InstallSource,
    metadata: &InstallMetadata,
    state: &AppState,
) -> Result<()> {
    let profile_path: PathBuf = sqlx::query!(
        "SELECT path FROM profiles WHERE id = ?",
        metadata.profile_id
    )
    .fetch_one(&state.db)
    .await?
    .path
    .into();

    let source = match source {
        InstallSource::Thunderstore(version_uuid) => {
            let identifier =
                gale_install::from_thunderstore(version_uuid, &profile_path, state).await?;

            ProfileModSource::Thunderstore {
                identifier,
                version_uuid,
            }
        }
        InstallSource::Local(_path) => todo!(),
    };

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
            .fetch_one(&state.db)
            .await?
            .max
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

    Ok(())
}
