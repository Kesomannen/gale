use std::{
    path::PathBuf,
    process::Child,
    sync::{Arc, Mutex},
};

use eyre::{Result, ensure};
use serde::Serialize;

pub type SharedChild = Arc<Mutex<Child>>;

#[derive(Default)]
pub struct ServerRuntime {
    running: Option<RunningServer>,
}

struct RunningServer {
    profile_id: i64,
    game_slug: String,
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
                game_slug: running.game_slug.clone(),
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
        game_slug: String,
        server_dir: PathBuf,
        child: SharedChild,
    ) -> Result<ServerStatus> {
        ensure!(
            self.running.is_none(),
            "a Gale-managed dedicated server is already running"
        );

        let pid = child
            .lock()
            .expect("server child process mutex poisoned")
            .id();

        self.running = Some(RunningServer {
            profile_id,
            game_slug,
            server_dir,
            pid,
            child,
        });

        Ok(self.status())
    }

    // An old watcher must not clear a newer server.
    pub fn clear_if_pid(&mut self, pid: u32) -> Option<i64> {
        let matches = self
            .running
            .as_ref()
            .is_some_and(|running| running.pid == pid);

        if !matches {
            return None;
        }

        self.running.take().map(|running| running.profile_id)
    }

    pub fn force_stop(&mut self) -> Result<Option<i64>> {
        let Some(running) = self.running.take() else {
            return Ok(None);
        };

        let profile_id = running.profile_id;

        let mut child = running
            .child
            .lock()
            .expect("server child process mutex poisoned");

        if child.try_wait()?.is_none() {
            child.kill()?;
            let _ = child.wait();
        }

        Ok(Some(profile_id))
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
