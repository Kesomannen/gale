use std::sync::{Mutex, MutexGuard};

use eyre::{eyre, Context, Result};
use rusqlite::params;
use serde::de::DeserializeOwned;

use crate::{
    profile::{ModManager, ProfileSaveData},
    util,
};

pub struct Db(Mutex<rusqlite::Connection>);

pub fn init() -> Result<Db> {
    let path = util::path::default_app_data_dir().join("data.sqlite3");

    let conn = rusqlite::Connection::open(path)?;
    create_tables(&conn).context("failed to create schema")?;

    Ok(Db(Mutex::new(conn)))
}

fn create_tables(conn: &rusqlite::Connection) -> Result<()> {
    create_json_table("manager", conn)?;
    create_json_table("managed_games", conn)?;
    create_json_table("profiles", conn)?;

    Ok(())
}

fn create_json_table(name: &str, conn: &rusqlite::Connection) -> Result<()> {
    conn.execute(
        &format!(
            "CREATE TABLE IF NOT EXISTS {} (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            data JSON NOT NULL
        )",
            name
        ),
        (),
    )?;

    Ok(())
}

pub struct QueryResult<T> {
    pub id: i64,
    pub data: T,
}

trait StatementExt {
    fn map_json<T>(&mut self) -> Result<impl Iterator<Item = Result<QueryResult<T>>>>
    where
        T: DeserializeOwned;
}

impl StatementExt for rusqlite::Statement<'_> {
    fn map_json<T>(&mut self) -> Result<impl Iterator<Item = Result<QueryResult<T>>>>
    where
        T: DeserializeOwned,
    {
        Ok(self
            .query_map((), |row| {
                Ok((row.get::<_, i64>("id")?, row.get::<_, String>("data")?))
            })?
            .map(|res| {
                res.map_err(|err| eyre!(err)).and_then(|(id, str)| {
                    let data: T =
                        serde_json::from_str(&str).context("failed to deserialize row")?;

                    Ok(QueryResult { id, data })
                })
            }))
    }
}

impl Db {
    fn conn(&self) -> MutexGuard<'_, rusqlite::Connection> {
        self.0.lock().unwrap()
    }

    pub fn get_all_profiles(&self) -> Result<Vec<QueryResult<ProfileSaveData>>> {
        let conn = self.conn();
        let profiles = conn
            .prepare("SELECT id, data FROM profiles")?
            .map_json()?
            .collect::<Result<Vec<_>>>()?;

        Ok(profiles)
    }

    pub fn save_manager(&self, manager: &ModManager) -> Result<()> {
        let mut conn = self.conn();

        let tx = conn.transaction()?;

        let string = serde_json::to_string(&manager.save_data())?;
        tx.execute(
            "INSERT OR REPLACE INTO manager (id, data)
            VALUES (1, ?)",
            [string],
        )?;

        {
            let mut stmt =
                tx.prepare("INSERT OR REPLACE INTO managed_games (id, data) VALUES (?, ?)")?;
            for (i, game) in manager.games.values().enumerate() {
                let string = serde_json::to_string(&game.save_data())?;
                stmt.execute(params![i + 1, string])?;
            }
        }

        {
            let mut stmt =
                tx.prepare("INSERT OR REPLACE INTO profiles (id, data) VALUES (?, ?)")?;
            for (i, profile) in manager
                .games
                .values()
                .flat_map(|game| &game.profiles)
                .enumerate()
            {
                let string = serde_json::to_string(&profile.save_data())?;
                stmt.execute(params![i + 1, string])?;
            }
        }

        tx.commit()?;

        Ok(())
    }
}
