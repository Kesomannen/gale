use std::{
    collections::HashSet,
    iter,
    sync::{Mutex, MutexGuard},
};

use eyre::{Context, Result};
use rusqlite::{params, types::Type as SqliteType};
use serde::de::DeserializeOwned;
use uuid::Uuid;

use crate::{
    profile::{self, ManagedGame, ModManager, Profile},
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
    conn.execute(
        "CREATE TABLE IF NOT EXISTS manager (
            id INTEGER PRIMARY KEY NOT NULL,
            active_game_slug TEXT
        )",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS managed_games (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            slug TEXT NOT NULL,
            favorite BOOLEAN NOT NULL DEFAULT FALSE,
            active_profile_id INT NOT NULL
        )",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS profiles (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            name TEXT NOT NULL,
            path TEXT NOT NULL,
            game_slug TEXT NOT NULL,
            mods JSON NOT NULL,
            modpack JSON,
            ignored_updates JSON
        )",
        (),
    )?;

    Ok(())
}

fn map_json_row<I, T>(row: &rusqlite::Row, idx: I) -> rusqlite::Result<T>
where
    I: rusqlite::RowIndex,
    T: DeserializeOwned,
{
    let string = row.get::<_, String>(idx)?;
    serde_json::from_str(&string).map_err(|err| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(err))
    })
}

fn map_json_option_row<I, T>(row: &rusqlite::Row, idx: I) -> rusqlite::Result<Option<T>>
where
    I: rusqlite::RowIndex,
    T: DeserializeOwned,
{
    match map_json_row(row, idx) {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::InvalidColumnType(_, _, SqliteType::Null)) => Ok(None),
        Err(err) => Err(err),
    }
}

pub struct ManagerData {
    pub id: i64,
    pub active_game_slug: Option<String>,
}

pub struct ManagedGameData {
    pub id: i64,
    pub slug: String,
    pub favorite: bool,
    pub active_profile_id: i64,
}

pub struct ProfileData {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub game_slug: String,
    pub mods: Vec<profile::ProfileMod>,
    pub modpack: Option<profile::export::modpack::ModpackArgs>,
    pub ignored_updates: Option<HashSet<Uuid>>,
}

pub struct SaveData {
    pub manager: ManagerData,
    pub games: Vec<ManagedGameData>,
    pub profiles: Vec<ProfileData>,
}

impl Db {
    fn conn(&self) -> MutexGuard<'_, rusqlite::Connection> {
        self.0.lock().unwrap()
    }

    fn with_transaction<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce(&rusqlite::Transaction) -> Result<()>,
    {
        let mut conn = self.conn();
        let tx = conn.transaction()?;

        f(&tx)?;

        tx.commit()?;
        Ok(())
    }

    pub fn next_profile_id(&self) -> Result<i64> {
        let conn = self.conn();

        let res = conn
            .prepare("SELECT MAX(id) + 1 FROM profiles")?
            .query_row((), |row| match row.get::<_, i64>(0) {
                Ok(value) => Ok(value),
                // if there are no profiles, return 1
                Err(rusqlite::Error::InvalidColumnType(_, _, SqliteType::Null)) => Ok(1),
                err => err,
            })?;

        Ok(res)
    }

    pub fn read(&self) -> Result<SaveData> {
        let conn = self.conn();

        let manager = conn
            .prepare("SELECT id, active_game_slug FROM manager")?
            .query_map((), |row| {
                Ok(ManagerData {
                    id: row.get(0)?,
                    active_game_slug: row.get(1)?,
                })
            })?
            .next()
            .transpose()?
            .unwrap_or_else(|| ManagerData {
                id: 1,
                active_game_slug: None,
            });

        let games = conn
            .prepare("SELECT id, slug, favorite, active_profile_id FROM managed_games")?
            .query_map((), |row| {
                Ok(ManagedGameData {
                    id: row.get(0)?,
                    slug: row.get(1)?,
                    favorite: row.get(2)?,
                    active_profile_id: row.get(3)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        let profiles = conn
            .prepare(
                "SELECT id, name, path, game_slug, mods, modpack, ignored_updates FROM profiles",
            )?
            .query_map((), |row| {
                Ok(ProfileData {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    path: row.get(2)?,
                    game_slug: row.get(3)?,
                    mods: map_json_row(row, 4)?,
                    modpack: map_json_option_row(row, 5)?,
                    ignored_updates: map_json_row(row, 6)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(SaveData {
            manager,
            games,
            profiles,
        })
    }

    pub fn save_all(&self, manager: &ModManager) -> Result<()> {
        self.with_transaction(|tx| {
            self._save_manager(tx, manager)?;
            self.save_games(tx, manager.games.values())?;
            self.save_profiles(tx, manager.games.values().flat_map(|game| &game.profiles))?;

            Ok(())
        })
    }

    pub fn save_manager(&self, manager: &ModManager) -> Result<()> {
        self.with_transaction(|tx| self._save_manager(&tx, manager))
    }

    fn _save_manager(&self, tx: &rusqlite::Transaction, manager: &ModManager) -> Result<()> {
        tx.execute(
            "INSERT OR REPLACE INTO manager (id, active_game_slug)
            VALUES (?, ?)",
            params![1, manager.active_game.slug],
        )?;

        Ok(())
    }

    pub fn save_game(&self, game: &ManagedGame) -> Result<()> {
        self.with_transaction(|tx| self.save_games(&tx, iter::once(game)))
    }

    fn save_games<'a>(
        &self,
        tx: &rusqlite::Transaction,
        games: impl Iterator<Item = &'a ManagedGame>,
    ) -> Result<()> {
        let mut stmt = tx.prepare(
            "INSERT OR REPLACE INTO managed_games (id, slug, favorite, active_profile_id)
                VALUES (?, ?, ?, ?)",
        )?;

        for game in games {
            stmt.execute(params![
                game.id,
                game.game.slug,
                game.favorite,
                game.active_profile_id
            ])?;
        }

        Ok(())
    }

    pub fn save_profile(&self, profile: &Profile) -> Result<()> {
        self.with_transaction(|tx| self.save_profiles(&tx, iter::once(profile)))
    }

    fn save_profiles<'a>(
        &self,
        tx: &rusqlite::Transaction,
        profiles: impl Iterator<Item = &'a Profile>,
    ) -> Result<()> {
        let mut stmt = tx.prepare(
            "INSERT OR REPLACE INTO profiles 
                (id, name, path, game_slug, mods, modpack, ignored_updates) 
                VALUES (?, ?, ?, ?, ?, ?, ?)",
        )?;

        for profile in profiles {
            let mods = serde_json::to_string(&profile.mods)?;
            let modpack = profile
                .modpack
                .as_ref()
                .map(|modpack| serde_json::to_string(modpack))
                .transpose()?;
            let ignored_updates = serde_json::to_string(&profile.ignored_updates)?;

            stmt.execute(params![
                profile.id,
                profile.name,
                profile.path.to_string_lossy(),
                profile.game.slug,
                mods,
                modpack,
                ignored_updates
            ])?;
        }

        Ok(())
    }
}
