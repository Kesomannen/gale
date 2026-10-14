use std::{path::PathBuf, sync::Arc, time::Duration};

use eyre::{OptionExt, Result, ensure};
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::{process::Child, sync::Mutex, time::sleep};
use tracing::{info, warn};

use super::local::LocalServerProcess;
use crate::{game::Game, state::ManagerExt};

const PROCESS_POLL_INTERVAL: Duration = Duration::from_millis(500);

type SharedChild = Arc<Mutex<Child>>;

#[derive(Default)]
pub struct ServerRuntime {
    running: Option<RunningServer>,
}

pub fn start(app: AppHandle, process: LocalServerProcess) -> Result<ServerStatus> {
    let pid = process
        .child
        .id()
        .ok_or_eyre("dedicated server process did not provide an id")?;
    let child = Arc::new(Mutex::new(process.child));
    let running = RunningServer {
        profile_id: process.profile_id,
        game: process.game,
        server_dir: process.server_dir,
        pid,
        child: child.clone(),
    };
    let status = match app.lock_server_runtime().register(running) {
        Ok(status) => status,
        Err(error) => {
            if let Ok(mut child) = child.try_lock() {
                let _ = child.start_kill();
            }
            return Err(error);
        }
    };

    emit_status(&app, &status);
    tauri::async_runtime::spawn(watch(app, child, pid));

    Ok(status)
}

pub async fn stop(app: &AppHandle) -> Result<()> {
    let Some((child, pid)) = app
        .lock_server_runtime()
        .running
        .as_ref()
        .map(|running| (running.child.clone(), running.pid))
    else {
        return Ok(());
    };

    let mut child = child.lock().await;

    if child.try_wait()?.is_none() {
        child.kill().await?;
        let _ = child.wait().await;
    }
    drop(child);

    if app.lock_server_runtime().clear_if_pid(pid) {
        emit_status(app, &ServerStatus::Stopped);
    }

    Ok(())
}

async fn watch(app: AppHandle, child: SharedChild, pid: u32) {
    loop {
        let result = child.lock().await.try_wait();

        match result {
            Ok(Some(exit_status)) => {
                info!(pid, ?exit_status, "dedicated server exited");

                let cleared = app.lock_server_runtime().clear_if_pid(pid);
                if cleared {
                    emit_status(&app, &ServerStatus::Stopped);
                }

                return;
            }
            Ok(None) => sleep(PROCESS_POLL_INTERVAL).await,
            Err(err) => {
                warn!(pid, ?err, "failed to query dedicated server process");

                let cleared = app.lock_server_runtime().clear_if_pid(pid);
                if cleared {
                    emit_status(&app, &ServerStatus::Stopped);
                }

                return;
            }
        }
    }
}

fn emit_status(app: &AppHandle, status: &ServerStatus) {
    if let Err(err) = app.emit("server_status_changed", status) {
        warn!(?err, "failed to emit dedicated server status");
    }
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

    fn register(&mut self, running: RunningServer) -> Result<ServerStatus> {
        ensure!(
            self.running.is_none(),
            "a Gale-managed dedicated server is already running"
        );

        self.running = Some(running);

        Ok(self.status())
    }

    fn clear_if_pid(&mut self, pid: u32) -> bool {
        let matches = self
            .running
            .as_ref()
            .is_some_and(|running| running.pid == pid);

        if matches {
            self.running.take();
        }

        matches
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
