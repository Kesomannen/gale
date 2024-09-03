use chrono::{DateTime, NaiveDateTime, Utc};
use gale_core::prelude::*;
use log::debug;
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use std::time::Instant;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryArgs {
    search_term: String,
    max_results: u32,
    community_id: u32,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Package {
    id: Uuid,
    name: String,
    owner: String,
    description: String,
    is_pinned: bool,
    is_deprecated: bool,
    rating_score: i64,
    has_nsfw_content: bool,
}

pub async fn query_packages(args: QueryArgs, state: &AppState) -> Result<Vec<Package>> {
    let start = Instant::now();
    let results = sqlx::query_as!(
        Package,
        r#"SELECT
            p.id AS "id: Uuid",
            p.name,
            p.owner,
            p.description,
            p.is_pinned,
            p.is_deprecated,
            p.rating_score,
            p.has_nsfw_content
        FROM
            packages p
            JOIN packages_fts fts ON
                fts.package_id = p.id
        WHERE
            p.community_id = ? AND
            fts.packages_fts MATCH ?
        ORDER BY
            p.is_pinned DESC,
            bm25(packages_fts, 0, 10, 2, 5) ASC
        LIMIT ?
    "#,
        args.community_id,
        args.search_term,
        args.max_results
    )
    .fetch_all(&state.db)
    .await?;

    debug!(
        "query_packages returned {} results in {:?}",
        results.len(),
        start.elapsed()
    );

    Ok(results)
}
