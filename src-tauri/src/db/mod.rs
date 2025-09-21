use std::{
    collections::HashSet,
    iter,
    sync::{Mutex, MutexGuard},
};

use eyre::{Context, Result};
use include_dir::include_dir;
use rusqlite::{params, types::Type as SqliteType, OptionalExtension};
use rusqlite_migration::Migrations;
use serde::de::DeserializeOwned;
use tracing::{info, trace};
use uuid::Uuid;

use crate::{
    prefs::Prefs,
    profile::{self, sync::auth::AuthCredentials, ManagedGame, ModManager, Profile},
    util,
};

pub mod cache;
mod migrate;

pub const FILE_NAME: &str = "data.sqlite3";
pub const SHM_FILE_NAME: &str = "data.sqlite3-shm";
pub const WAL_FILE_NAME: &str = "data.sqlite3-wal";

pub struct Db(Mutex<rusqlite::Connection>);

pub fn init() -> Result<(Db, bool)> {
    let path = util::path::default_app_data_dir().join(FILE_NAME);

    let existed = path.exists();

    info!(
        "connecting to database at {} (exists: {})",
        path.display(),
        existed
    );

    let mut conn = rusqlite::Connection::open(path).context("failed to connect")?;

    conn.pragma_update(None, "journal_mode", "WAL")
        .context("failed to set journal mode")?;

    conn.pragma_update(None, "synchronous", "normal")
        .context("failed to set synchronous mode")?;

    conn.trace(Some(trace_stmt));

    run_migrations(&mut conn).context("failed to run migrations")?;

    Ok((Db(Mutex::new(conn)), existed))
}

fn trace_stmt(stmt: &str) {
    trace!("{stmt}");
}

static MIGRATIONS_DIR: include_dir::Dir = include_dir!("$CARGO_MANIFEST_DIR/migrations");

fn run_migrations(conn: &mut rusqlite::Connection) -> Result<()> {
    let migrations = Migrations::from_directory(&MIGRATIONS_DIR)?;

    migrations.to_latest(conn)?;

    Ok(())
}

fn map_json_row<I, T>(row: &rusqlite::Row, idx: I) -> rusqlite::Result<T>
where
    I: rusqlite::RowIndex,
    T: DeserializeOwned,
{
    let string = row.get::<_, String>(idx)?;
    serde_json::from_str(&string).map_err(|err| {
        rusqlite::Error::FromSqlConversionFailure(0, SqliteType::Text, Box::new(err))
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
    pub sync_data: Option<profile::sync::SyncProfileData>,
    pub custom_args: Option<Vec<String>>,
    pub custom_args_enabled: Option<bool>,
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

    pub fn save_auth(&self, creds: Option<&AuthCredentials>) -> Result<()> {
        self.with_transaction(|tx| {
            let json = creds.map(serde_json::to_string).transpose()?;

            tx.prepare("INSERT OR REPLACE INTO auth (id, data) VALUES (?, ?)")?
                .execute(params![1, json])?;

            Ok(())
        })
    }

    pub fn read(&self) -> Result<(SaveData, Prefs, Option<AuthCredentials>, bool)> {
        if migrate::should_migrate() {
            let (data, prefs) = migrate::migrate().context("failed to migrate legacy save data")?;

            return Ok((data, prefs, None, true));
        }

        let conn = self.conn();

        let manager = conn
            .prepare("SELECT id, active_game_slug FROM manager")?
            .query_row((), |row| {
                Ok(ManagerData {
                    id: row.get(0)?,
                    active_game_slug: row.get(1)?,
                })
            })
            .optional()?
            .unwrap_or(ManagerData {
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

        let mut profiles = conn
            .prepare(
                "SELECT id, name, path, game_slug, mods, modpack, ignored_updates, sync_data, custom_args, custom_args_enabled FROM profiles",
            )?
            .query_map((), |row| {
                Ok(ProfileData {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    path: row.get(2)?,
                    game_slug: row.get(3)?,
                    mods: map_json_row(row, 4)?,
                    modpack: map_json_option_row(row, 5)?,
                    ignored_updates: map_json_option_row(row, 6)?,
                    sync_data: map_json_option_row(row, 7)?,
                    custom_args: map_json_option_row(row, 8)?,
                    custom_args_enabled: row.get(9)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        profiles.sort_by(|a, b| a.name.cmp(&b.name));

        let prefs = conn
            .prepare("SELECT data FROM prefs")?
            .query_row((), |row| map_json_row(row, 0))
            .optional()?
            .unwrap_or_default();

        let auth_state = conn
            .prepare("SELECT data FROM auth")?
            .query_row((), |row| map_json_option_row(row, 0))
            .optional()?
            .flatten();

        Ok((
            SaveData {
                manager,
                games,
                profiles,
            },
            prefs,
            auth_state,
            false,
        ))
    }

    pub fn delete_profile(&self, id: i64) -> Result<()> {
        self.with_transaction(|tx| {
            tx.prepare("DELETE FROM profiles WHERE id = ?")?
                .execute([id])?;

            Ok(())
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
        self.with_transaction(|tx| self._save_manager(tx, manager))
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
        self.with_transaction(|tx| self.save_games(tx, iter::once(game)))
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
        self.with_transaction(|tx| self.save_profiles(tx, iter::once(profile)))
    }

    fn save_profiles<'a>(
        &self,
        tx: &rusqlite::Transaction,
        profiles: impl Iterator<Item = &'a Profile>,
    ) -> Result<()> {
        let mut stmt = tx.prepare(
            "INSERT OR REPLACE INTO profiles 
                (id, name, path, game_slug, mods, modpack, ignored_updates, sync_data, custom_args, custom_args_enabled) 
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )?;

        for profile in profiles {
            let mods = serde_json::to_string(&profile.mods)?;
            let modpack = profile
                .modpack
                .as_ref()
                .map(serde_json::to_string)
                .transpose()?;
            let ignored_updates = serde_json::to_string(&profile.ignored_updates)?;
            let sync_data = profile
                .sync
                .as_ref()
                .map(serde_json::to_string)
                .transpose()?;
            let custom_args = serde_json::to_string(&profile.custom_args)?;

            stmt.execute(params![
                profile.id,
                profile.name,
                profile.path.to_string_lossy(),
                profile.game.slug,
                mods,
                modpack,
                ignored_updates,
                sync_data,
                custom_args,
                profile.custom_args_enabled
            ])?;
        }

        Ok(())
    }

    pub fn save_prefs(&self, prefs: &Prefs) -> Result<()> {
        self.with_transaction(|tx| {
            let json = serde_json::to_string(prefs).context("failed to serialize to json")?;

            tx.prepare("INSERT OR REPLACE INTO prefs (id, data) VALUES (1, ?)")?
                .execute([json])?;

            Ok(())
        })
    }
}
