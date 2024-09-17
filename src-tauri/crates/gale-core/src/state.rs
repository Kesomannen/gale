use crate::error::Result;
use anyhow::{anyhow, Context};
use log::debug;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqliteSynchronous},
    ConnectOptions,
};
use std::{env::current_exe, path::PathBuf, str::FromStr};
use tauri::{AppHandle, Wry};

pub trait ManagerExt {
    fn app_state(&self) -> &AppState;

    fn db(&self) -> &SqlitePool {
        &self.app_state().db
    }

    fn reqwest(&self) -> &reqwest::Client {
        &self.app_state().reqwest
    }
}

impl<M> ManagerExt for M
where
    M: tauri::Manager<Wry>,
{
    fn app_state(&self) -> &AppState {
        self.state::<AppState>().inner()
    }
}

pub struct AppState {
    pub db: SqlitePool,
    pub reqwest: reqwest::Client,
}

impl AppState {
    pub(crate) async fn setup(_app: &AppHandle) -> Result<AppState> {
        let path = current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("data.sqlite3");

        let url = format!(
            "sqlite:///{}",
            path.to_str().context("path contains invalid UTF-8")?
        );

        debug!("connecting to database at {url}");

        let options = SqliteConnectOptions::from_str(&url)?
            .synchronous(SqliteSynchronous::Normal)
            .journal_mode(SqliteJournalMode::Wal)
            .disable_statement_logging()
            .create_if_missing(true);

        let db = SqlitePool::connect_with(options)
            .await
            .context("failed to connect to database")?;

        sqlx::migrate!("../../../migrations")
            .run(&db)
            .await
            .context("failed to run database migrations")?;

        let reqwest = reqwest::Client::builder()
            .user_agent(format!("gale/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .context("failed to create reqwest client")?;

        Ok(AppState { db, reqwest })
    }

    pub async fn profile_path(&self, id: i64) -> Result<PathBuf> {
        let path = sqlx::query!("SELECT path FROM profiles WHERE id = ?", id)
            .fetch_optional(&self.db)
            .await?
            .ok_or_else(|| anyhow!("profile with {} not found", id))?
            .path;

        Ok(path.into())
    }
}
