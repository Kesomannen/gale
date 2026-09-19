use std::{path::PathBuf, sync::Arc, time::Duration};

use eyre::Result;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::{game::Game, state::ManagerExt};

pub type SharedChild = Arc<Mutex<tokio::process::Child>>;

const PROCESS_POLL_INTERVAL: Duration = Duration::from_millis(500);

#[derive(Default)]
pub struct ServerRuntime {
    running: Option<RunningServer>,
}

struct RunningServer {
    profile_id: i64,
    game: Game,
    server_dir: PathBuf,
    pid: u32,
    child: SharedChild,
}

#[derive(Debug, Clone, Serialize)]
#[serde(
    tag = "state",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ServerStatus {
    Stopped,

    Running {
        profile_id: i64,
        game_slug: String,
        pid: u32,
        server_dir: PathBuf,
    },
}

impl ServerRuntime {
    pub fn is_running(&self) -> bool {
        self.running.is_some()
    }

    pub fn is_profile_locked(&self, profile_id: i64) -> bool {
        self.running
            .as_ref()
            .is_some_and(|running| running.profile_id == profile_id)
    }

    pub fn status(&self) -> ServerStatus {
        match &self.running {
            Some(running) => ServerStatus::Running {
                profile_id: running.profile_id,
                game_slug: running.game.slug.to_string(),
                pid: running.pid,
                server_dir: running.server_dir.clone(),
            },

            None => ServerStatus::Stopped,
        }
    }

    pub fn server_dir(&self) -> Option<PathBuf> {
        self.running
            .as_ref()
            .map(|running| running.server_dir.clone())
    }

    pub fn register(
        &mut self,
        profile_id: i64,
        game: Game,
        server_dir: PathBuf,
        pid: u32,
        child: SharedChild,
    ) -> Result<ServerStatus> {
        eyre::ensure!(
            self.running.is_none(),
            "a Gale-managed dedicated server is already running"
        );

        self.running = Some(RunningServer {
            profile_id,
            game,
            server_dir,
            pid,
            child,
        });

        Ok(self.status())
    }

    /// An old watcher must not clear a newer server.
    pub fn clear_if_pid(&mut self, pid: u32) -> bool {
        let matches = self
            .running
            .as_ref()
            .is_some_and(|running| running.pid == pid);

        matches && self.running.take().is_some()
    }

    /// Takes ownership of the running child so the caller can kill it without
    /// holding the runtime lock across an await.
    pub fn take(&mut self) -> Option<SharedChild> {
        self.running.take().map(|running| running.child)
    }
}

/// Watches the server process and updates the runtime when it exits.
///
/// Runs as a tokio task instead of a std thread: the lock is only held for
/// the non-blocking `try_wait` call, so `take`/`kill` can still proceed.
pub fn watch(app: AppHandle, child: SharedChild, pid: u32) {
    tauri::async_runtime::spawn(async move {
        let result = loop {
            let status = {
                let mut child = child.lock().await;
                child.try_wait()
            };

            match status {
                Ok(Some(status)) => break Ok(status),
                Ok(None) => tokio::time::sleep(PROCESS_POLL_INTERVAL).await,
                Err(err) => break Err(err),
            }
        };

        match result {
            Ok(status) => info!(pid, ?status, "dedicated server exited"),
            Err(err) => warn!(pid, ?err, "failed to query dedicated server process"),
        }

        let mut runtime = app.lock_server_runtime();
        if runtime.clear_if_pid(pid) {
            emit_status(&app, &runtime.status());
        }
    });
}

/// Kills the process behind `child` and reports the stopped status.
///
/// Expects the runtime entry to have been removed already via [`ServerRuntime::take`].
pub async fn kill(app: AppHandle, child: SharedChild) -> Result<()> {
    let status = {
        let mut child = child.lock().await;
        child.kill().await
    };

    if let Err(err) = status {
        warn!(?err, "failed to kill dedicated server process");
    }

    let status = app.lock_server_runtime().status();
    emit_status(&app, &status);

    Ok(())
}

pub fn emit_status(app: &AppHandle, status: &ServerStatus) {
    if let Err(err) = app.emit("server_status_changed", status) {
        warn!(?err, "failed to emit dedicated server status");
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::ServerStatus;

    #[test]
    fn serializes_frontend_field_names() {
        let value = serde_json::to_value(ServerStatus::Running {
            profile_id: 7,
            game_slug: "valheim".to_owned(),
            pid: 123,
            server_dir: PathBuf::from("server"),
        })
        .unwrap();

        assert_eq!(value["profileId"], 7);
        assert_eq!(value["gameSlug"], "valheim");
        assert_eq!(value["serverDir"], "server");
    }
}
