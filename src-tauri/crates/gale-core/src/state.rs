use crate::Result;
use anyhow::Context;
use log::debug;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqliteSynchronous},
    ConnectOptions,
};
use std::{env::current_exe, str::FromStr};
use tauri::AppHandle;

pub struct AppState {
    pub db: SqlitePool,
    pub thunderstore: thunderstore::Client,
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
            "sqlite://{}",
            path.to_str().context("path contains invalid UTF-8")?
        );

        debug!("connecting to database at {url}");

        let options = SqliteConnectOptions::from_str(&url)?
            .synchronous(SqliteSynchronous::Off)
            .journal_mode(SqliteJournalMode::Wal)
            .disable_statement_logging()
            .create_if_missing(true);
        let db = SqlitePool::connect_with(options).await?;

        sqlx::migrate!("../../../migrations").run(&db).await?;

        let thunderstore = thunderstore::Client::new();

        Ok(AppState { db, thunderstore })
    }
}
