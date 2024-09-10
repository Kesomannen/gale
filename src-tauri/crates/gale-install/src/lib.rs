use gale_core::prelude::*;
use sqlx::types::Uuid;
use std::{collections::VecDeque, path::PathBuf, sync::Mutex};
use tauri::{
    generate_handler,
    plugin::{Builder, TauriPlugin},
    AppHandle, Manager,
};

mod cache;
mod commands;
mod common;
mod local;
mod thunderstore;

pub struct InstallMetadata {
    name: String,
    enabled: bool,
    index: Option<usize>,
    profile_id: i64,
}

impl InstallMetadata {
    pub fn new(name: String, profile_id: i64) -> Self {
        Self {
            name,
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

pub enum InstallSource {
    Thunderstore(Uuid),
    Local(PathBuf),
}

pub struct InstallQueue {
    queue: Mutex<VecDeque<(InstallSource, InstallMetadata)>>,
}

impl InstallQueue {
    pub async fn enqueue(&self, source: InstallSource, metadata: InstallMetadata) -> Result<usize> {
        let mut queue = self.queue.lock().unwrap();
        queue.push_back((source, metadata));

        Ok(queue.len())
    }
}

pub fn init() -> TauriPlugin<tauri::Wry> {
    Builder::new("gale-install")
        .setup(|app, _| {
            let queue = InstallQueue {
                queue: Mutex::new(VecDeque::new()),
            };

            app.manage(queue);

            let handle = app.to_owned();
            tauri::async_runtime::spawn(install_task(handle));

            Ok(())
        })
        .invoke_handler(generate_handler![])
        .build()
}

async fn install_task(app: AppHandle) {
    let state = app.state::<AppState>();
    let queue = app.state::<InstallQueue>();

    loop {
        let next = queue.queue.lock().unwrap().pop_front();

        let (source, metadata) = match next {
            Some(item) => item,
            None => {
                continue;
            }
        };

        if let Err(err) = handle_request(source, &metadata, &state).await {
            log::error!("failed to install {}: {:#}", metadata.name, err);
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

    match source {
        InstallSource::Thunderstore(version_uuid) => {
            thunderstore::install(version_uuid, &profile_path, state).await?;
        }
        InstallSource::Local(path) => todo!(),
    };

    /*
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

    */

    Ok(())
}
