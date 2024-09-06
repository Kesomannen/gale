use futures_util::FutureExt;
use gale_core::prelude::*;
use log::{debug, trace};
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
    downloads: i64,
    has_nsfw_content: bool,
    major: i64,
    minor: i64,
    patch: i64,
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
            p.downloads,
            p.has_nsfw_content,
            v.major,
            v.minor,
            v.patch
        FROM
            packages p
            JOIN packages_fts fts ON
                fts.package_id = p.id
            JOIN versions v ON
                v.id = p.latest_version_id
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

    trace!(
        "found {} results in {:?}",
        results.len(),
        start.elapsed()
    );

    Ok(results)
}

#[derive(Serialize, Debug)]
pub struct PackageInfo {
    name: String,
    owner: String,
    versions: Vec<VersionInfo>,
}

#[derive(Serialize, Debug)]
pub struct VersionInfo {
    major: i64,
    minor: i64,
    patch: i64,
}

pub async fn query_package(id: Uuid, state: &AppState) -> Result<PackageInfo> {
    let mut package = sqlx::query!(
        "SELECT
            name,
            owner
        FROM 
            packages
        WHERE id = ?",
        id
    )
    .map(|row| PackageInfo {
        name: row.name,
        owner: row.owner,
        versions: Vec::new(),
    })
    .fetch_one(&state.db)
    .await?;

    package.versions = sqlx::query_as!(
        VersionInfo,
        "SELECT
            major,
            minor,
            patch
        FROM
            versions
        WHERE
            package_id = ?
        ORDER BY
            major DESC,
            minor DESC,
            patch DESC",
        id
    )
    .fetch_all(&state.db)
    .await?;

    trace!("found package {:#?}", package);

    Ok(package)
}
