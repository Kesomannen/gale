use crate::api::{self, PackageV1, PackageVersionV1, VersionId};
use anyhow::Context;
use futures_util::{pin_mut, TryStreamExt};
use gale_core::prelude::*;
use log::{debug, trace};
use sqlx::prelude::*;
use std::{collections::HashMap, time::Instant};
use uuid::Uuid;

pub async fn fetch_packages(state: &AppState, game_id: u32) -> Result<()> {
    let start = Instant::now();

    let slug: String = sqlx::query("SELECT slug FROM games WHERE id = ?")
        .bind(game_id)
        .fetch_one(&state.db)
        .await?
        .get(0);

    debug!("fetching packages from {slug}");

    let categories = api::get_filters(&state.reqwest, &slug)
        .await?
        .package_categories;

    for category in &categories {
        sqlx::query!(
            "INSERT OR REPLACE INTO
            categories (id, name, slug, community_id)
            VALUES (?, ?, ?, ?)",
            category.id,
            category.name,
            category.slug,
            game_id
        )
        .execute(&state.db)
        .await?;
    }

    let categories = categories
        .into_iter()
        .map(|category| {
            let id: i64 = category.id.parse().expect("category id should be a number");
            (category.name, id)
        })
        .collect::<HashMap<_, _>>();

    let stream = api::stream_packages(&state.reqwest, slug).await?;
    pin_mut!(stream);

    let mut count = 0;
    let mut transaction = state.db.begin().await?;

    trace!("inserting new packages");

    while let Some(package) = stream.try_next().await? {
        count += 1;

        if count % 100 == 0 {
            trace!("fetched {count} packages");
        }

        insert_package(&package, game_id, &categories, &mut transaction)
            .await
            .context("failed to insert package")?;

        let uuid = package.uuid4;
        for version in package.versions {
            insert_version(version, uuid, &mut transaction)
                .await
                .context("failed to insert version")?;
        }
    }

    transaction.commit().await?;

    debug!("fetched {count} packages in {:?}", start.elapsed());

    Ok(())
}

async fn insert_package(
    package: &PackageV1,
    game_id: u32,
    category_map: &HashMap<String, i64>,
    transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
) -> Result<()> {
    let donation_link = package.donation_link.as_ref().map(|url| url.as_str());
    let total_downloads = package
        .versions
        .iter()
        .map(|version| version.downloads as i64)
        .sum::<i64>();

    sqlx::query!(
        "INSERT OR REPLACE INTO packages
        (
            id,
            name,
            description,
            date_created,
            donation_link,
            has_nsfw_content,
            is_deprecated,
            is_pinned,
            owner,
            rating_score,
            downloads,
            latest_version_id,
            game_id
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        package.uuid4,
        package.name,
        package.latest().description,
        package.date_created,
        donation_link,
        package.has_nsfw_content,
        package.is_deprecated,
        package.is_pinned,
        package.owner,
        package.rating_score,
        total_downloads,
        package.latest().uuid4,
        game_id
    )
    .execute(&mut **transaction)
    .await?;

    for category in &package.categories {
        let category_id = match category_map.get(category) {
            Some(id) => *id,
            None => {
                log::warn!(
                    "package {} has unknown category: {}",
                    package.full_name,
                    category
                );
                continue;
            }
        };

        sqlx::query!(
            "INSERT OR REPLACE INTO package_categories (package_id, category_id) VALUES (?, ?)",
            package.uuid4,
            category_id
        )
        .execute(&mut **transaction)
        .await?;
    }

    Ok(())
}

async fn insert_version(
    version: PackageVersionV1,
    package_uuid: Uuid,
    transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
) -> Result<()> {
    let file_size = version.file_size as i64;
    let website_url = match version.website_url.is_empty() {
        true => None,
        false => Some(&version.website_url),
    };
    let major = version.version_number.major as i64;
    let minor = version.version_number.minor as i64;
    let patch = version.version_number.patch as i64;

    sqlx::query!(
        "INSERT OR REPLACE INTO versions
        (
            id,
            package_id,
            date_created,
            downloads,
            file_size,
            is_active,
            website_url,
            major,
            minor,
            patch
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        version.uuid4,
        package_uuid,
        version.date_created,
        version.downloads,
        file_size,
        version.is_active,
        website_url,
        major,
        minor,
        patch
    )
    .execute(&mut **transaction)
    .await?;

    /*
    for dependency in version.dependencies {
        insert_dependency(version.uuid4, dependency, transaction)
            .await
            .context("failed to insert dependency")?;
    }
    */

    Ok(())
}

async fn insert_dependency(
    version_uuid: Uuid,
    dependency: VersionId,
    transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
) -> Result<()> {
    let (major, minor, path) = dependency.version_split();
    let owner = dependency.owner();
    let name = dependency.name();

    sqlx::query!(
        "INSERT OR REPLACE INTO dependencies
        (
            dependent_id,
            owner,
            name,
            major,
            minor,
            patch
        )
        VALUES (?, ?, ?, ?, ?, ?)",
        version_uuid,
        owner,
        name,
        major,
        minor,
        path
    )
    .execute(&mut **transaction)
    .await?;

    Ok(())
}
