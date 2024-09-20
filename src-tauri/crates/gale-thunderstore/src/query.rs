use crate::api::{self, VersionId};
use anyhow::Context;
use chrono::NaiveDateTime;
use gale_core::prelude::*;
use log::trace;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::Uuid};
use std::time::Instant;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OrderBy {
    Relevance,
    LastUpdated,
    Created,
    Downloads,
    Rating,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryArgs {
    search_term: String,
    max_results: u32,
    game_id: u32,
    ascending: bool,
    order_by: OrderBy,
}

#[derive(Serialize, Debug, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ListedPackageInfo {
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
    version_uuid: Uuid,
}

pub async fn query_packages(args: QueryArgs, state: &AppState) -> Result<Vec<ListedPackageInfo>> {
    let start = Instant::now();

    let mut query = r#"
SELECT
    p.id,
    p.name,
    p.owner,
    p.description,
    p.is_pinned,
    p.is_deprecated,
    p.rating_score,
    p.downloads,
    p.has_nsfw_content,
    v.id AS version_uuid,
    v.major,
    v.minor,
    v.patch
FROM
    packages p
    JOIN versions v ON
        v.id = p.latest_version_id"#
        .to_owned();

    if args.search_term.len() >= 3 {
        query.push_str(
            r#"
    JOIN packages_fts ON
        packages_fts.package_id = p.id
WHERE
    p.game_id = ? AND
    packages_fts MATCH ?
"#,
        );
    } else {
        query.push_str("\nWHERE p.game_id = ?\n");
    }

    query.push_str(match args.order_by {
        OrderBy::Relevance if args.search_term.len() >= 3 => {
            "ORDER BY -bm25(packages_fts, 0, 10.0, 5.0, 5.0)"
        }
        OrderBy::Relevance | OrderBy::LastUpdated => "ORDER BY v.date_created",
        OrderBy::Created => "ORDER BY p.date_created",
        OrderBy::Downloads => "ORDER BY p.downloads",
        OrderBy::Rating => "ORDER BY p.rating_score",
    });

    query.push_str(if args.ascending { " ASC" } else { " DESC" });
    query.push_str("\nLIMIT ?");

    let mut query = sqlx::query_as::<_, ListedPackageInfo>(&query);
    query = query.bind(args.game_id);
    if args.search_term.len() >= 3 {
        query = query.bind(&args.search_term);
    }
    query = query.bind(args.max_results);

    let results = query.fetch_all(&state.db).await?;

    trace!("found {} results in {:?}", results.len(), start.elapsed());

    Ok(results)
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PackageInfo {
    name: String,
    owner: String,
    downloads: i64,
    rating_score: i64,
    website_url: Option<String>,
    donation_url: Option<String>,
    readme: Option<String>,
    changelog: Option<String>,
    versions: Vec<VersionInfo>,
    dependencies: Vec<DependencyInfo>,
}

#[derive(Serialize, Debug, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct VersionInfo {
    id: Uuid,
    major: i64,
    minor: i64,
    patch: i64,
    downloads: i64,
    file_size: i64,
    date_created: NaiveDateTime,
    #[serde(skip)]
    website_url: Option<String>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DependencyInfo {
    name: String,
    owner: String,
    major: i64,
    minor: i64,
    patch: i64,
}

pub async fn query_package(package_uuid: Uuid, state: &AppState) -> Result<PackageInfo> {
    let record = sqlx::query!(
        "SELECT
            name,
            owner,
            donation_link,
            downloads,
            rating_score
        FROM 
            packages
        WHERE 
            id = ?",
        package_uuid
    )
    .fetch_one(&state.db)
    .await?;

    let (name, owner, downloads, rating_score, donation_url) = (
        record.name,
        record.owner,
        record.downloads,
        record.rating_score,
        record.donation_link,
    );

    let versions = sqlx::query_as::<_, VersionInfo>(
        "SELECT
            id,
            major,
            minor,
            patch,
            downloads,
            file_size,
            date_created,
            website_url
        FROM
            versions
        WHERE
            package_id = ?
        ORDER BY
            major DESC,
            minor DESC,
            patch DESC",
    )
    .bind(package_uuid)
    .fetch_all(&state.db)
    .await?;

    let latest = versions
        .first()
        .expect("package should have at least one version");

    let dependencies = sqlx::query_as!(
        DependencyInfo,
        "SELECT
            name,
            owner,
            major,
            minor,
            patch
        FROM
            dependencies
        WHERE
            dependent_id = ?",
        latest.id
    )
    .fetch_all(&state.db)
    .await?;

    let id: VersionId = (
        &owner,
        &name,
        latest.major as u32,
        latest.minor as u32,
        latest.patch as u32,
    )
        .into();

    let (readme, changelog) = tokio::join!(
        api::get_readme(&state.reqwest, &id),
        api::get_changelog(&state.reqwest, &id)
    );

    let readme = readme.context("failed to fetch readme")?;
    let changelog = changelog.context("failed to fetch changelog")?;

    let package = PackageInfo {
        name,
        owner,
        donation_url,
        website_url: latest.website_url.clone(),
        readme,
        changelog,
        versions,
        dependencies,
        downloads,
        rating_score,
    };

    Ok(package)
}
