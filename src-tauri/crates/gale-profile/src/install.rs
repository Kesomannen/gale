use crate::{emit_update, ProfileModSource};
use anyhow::Context;
use gale_core::prelude::*;
use gale_install::{InstallSource, Progress};
use std::{collections::VecDeque, fmt::Debug, path::PathBuf, sync::Mutex, time::Instant};
use tauri::{AppHandle, Manager};
use tokio::sync::Notify;

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
            InstallSource::Local { name, version, .. } => ProfileModSource::Local { name, version },
            InstallSource::Github { owner, repo, tag } => {
                ProfileModSource::Github { owner, repo, tag }
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

    let mut loading_bar: Option<LoadingBar> = None;

    loop {
        let (count, next) = {
            let mut queue = queue.queue.lock().unwrap();

            (queue.len(), queue.pop_front())
        };

        let (source, metadata) = match next {
            Some(item) => item,
            None => {
                loading_bar = None;
                queue.notify.notified().await;
                continue;
            }
        };

        let name = source.package_id().into_owned();

        let loading_bar =
            loading_bar.get_or_insert_with(|| LoadingBar::new("Installing mods", &app));

        loading_bar
            .update()
            .set_progress(1.0 - count as f32)
            .send()
            .ok();

        let start = Instant::now();

        match handle_request(source, &metadata, &loading_bar, &state, &app).await {
            Ok(_) => {
                log::info!("installed {} in {:?}", name, start.elapsed());
            }
            Err(err) => {
                log::error!("failed to install {}: {:#}", name, err);
            }
        };
    }
}

async fn handle_request(
    source: InstallSource,
    metadata: &InstallMetadata,
    loading_bar: &LoadingBar<'_>,
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

    let name = source.name().into_owned();
    let on_progress = move |progress: Progress| {
        let text = match progress {
            Progress::Download { current, total } => {
                format!(
                    "Downloading {} ({}/{})",
                    name,
                    format_file_size(current),
                    format_file_size(total)
                )
            }
            Progress::Extract => format!("Extracting {}", name),
            Progress::Install => format!("Installing {}", name),
        };

        loading_bar.update().set_text(text).send().ok();
    };

    gale_install::install(&source, &profile_path, on_progress, &state).await?;

    let source = ProfileModSource::from(source);

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

    let json = sqlx::types::Json(source);

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

    emit_update(metadata.profile_id, state, app).await?;

    Ok(())
}

fn format_file_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    if bytes < KB as u64 {
        format!("{} B", bytes)
    } else if bytes < MB as u64 {
        format!("{:.1} KB", bytes as f64 / KB)
    } else if bytes < GB as u64 {
        format!("{:.1} MB", bytes as f64 / MB)
    } else {
        format!("{:.1} GB", bytes as f64 / GB)
    }
}
