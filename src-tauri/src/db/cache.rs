use std::time::Duration;

use chrono::Utc;
use eyre::Result;
use rusqlite::{params, OptionalExtension};
use tracing::{debug, info};
use uuid::Uuid;

use super::Db;

const SIZE_THRESHOLD: i64 = 1000;
const MAX_AGE: Duration = Duration::from_secs(60 * 60 * 24 * 30); // 30 days
const CACHE_TABLES: &[&str] = &["readme_cache", "changelog_cache"];

impl Db {
    pub fn evict_outdated_cache(&self) -> Result<()> {
        let conn = self.conn();

        for table in CACHE_TABLES {
            let count = conn
                .prepare(&format!("SELECT COUNT(*) FROM {table}"))?
                .query_row((), |row| row.get::<_, i64>(0))?;

            if count < SIZE_THRESHOLD {
                debug!(
                    "{} is below threshold ({} < {})",
                    table, count, SIZE_THRESHOLD
                );
                continue;
            }

            info!("evicting outdated records in {}", table);

            let cutoff = (Utc::now() - MAX_AGE).naive_utc();
            conn.prepare(&format!("DELETE FROM {table} WHERE created_at < $1"))?
                .execute(params![cutoff.to_string()])?;
        }

        Ok(())
    }

    pub fn get_cached(&self, table: &str, id: Uuid) -> Result<Option<Option<String>>> {
        let conn = self.conn();

        let content = conn
            .prepare(&format!(
                "SELECT content FROM {table} WHERE version_id = $1"
            ))?
            .query_row(params![id], |row| row.get(0))
            .optional()?;

        Ok(content)
    }

    pub fn insert_cached(&self, table: &str, id: Uuid, content: Option<&str>) -> Result<()> {
        let conn = self.conn();

        conn.prepare(&format!(
            "INSERT OR REPLACE INTO {table} (version_id, content) VALUES ($1, $2)"
        ))?
        .execute(params![id, content])?;

        Ok(())
    }
}
