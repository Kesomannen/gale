use crate::api::{self, VersionId};
use anyhow::Context;
use chrono::NaiveDateTime;
use futures_util::join;
use gale_core::prelude::*;
use log::trace;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::Uuid};
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
    let results = sqlx::query_as!(
        ListedPackageInfo,
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
            v.patch,
            v.id AS "version_uuid: Uuid"
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
            p.rating_score DESC
        LIMIT ?
    "#,
        args.community_id,
        args.search_term,
        args.max_results
    )
    .fetch_all(&state.db)
    .await?;

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

    let (readme, changelog) = join!(
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
