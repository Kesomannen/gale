use anyhow::Context;
use futures_util::{pin_mut, TryStreamExt};
use gale_core::prelude::*;
use log::debug;
use sqlx::prelude::*;
use std::time::Instant;
use thunderstore::models::{PackageV1, PackageVersionV1};

pub async fn fetch_packages(state: &AppState, community_id: u32) -> Result<()> {
    let start = Instant::now();

    let slug: String = sqlx::query("SELECT slug FROM communities WHERE id = ?")
        .bind(community_id)
        .fetch_one(&state.db)
        .await?
        .get(0);

    debug!("fetching packages from {slug}");
    let stream = state.thunderstore.stream_packages_v1(slug).await?;
    pin_mut!(stream);

    let mut transaction = state.db.begin().await?;
    let mut count = 0;

    while let Some(package) = stream.try_next().await? {
        count += 1;

        if count % 100 == 0 {
            debug!("fetched {count} packages");
        }

        insert_package(&package, community_id, &mut transaction)
            .await
            .context("failed to insert package")?;

        for version in &package.versions {
            insert_version(version, &package, &mut transaction)
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
    community: u32,
    transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
) -> Result<()> {
    sqlx::query(
        "INSERT OR REPLACE INTO packages
        (
            id,
            name,
            description,
            date_created,
            date_updated,
            donation_link,
            has_nsfw_content,
            is_deprecated,
            is_pinned,
            owner,
            rating_score,
            downloads,
            latest_version_id,
            community_id
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    ",
    )
    .bind(&package.uuid4.as_bytes()[..])
    .bind(&package.name)
    .bind(&package.latest().description)
    .bind(package.date_created.to_string())
    .bind(package.date_updated.to_string())
    .bind(package.donation_link.as_ref().map(|url| url.as_str()))
    .bind(package.has_nsfw_content)
    .bind(package.is_deprecated)
    .bind(package.is_pinned)
    .bind(&package.owner)
    .bind(package.rating_score)
    .bind(
        package
            .versions
            .iter()
            .map(|version| version.downloads as i64)
            .sum::<i64>(),
    )
    .bind(&package.latest().uuid4.as_bytes()[..])
    .bind(community)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}

async fn insert_version(
    version: &PackageVersionV1,
    package: &PackageV1,
    transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
) -> Result<()> {
    sqlx::query(
        "INSERT OR REPLACE INTO versions
        (
            id,
            package_id,
            date_created,
            description,
            downloads,
            file_size,
            full_name,
            is_active,
            name,
            website_url,
            major,
            minor,
            patch
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    ",
    )
    .bind(&version.uuid4.as_bytes()[..])
    .bind(&package.uuid4.as_bytes()[..])
    .bind(version.date_created.to_string())
    .bind(&version.description)
    .bind(version.downloads)
    .bind(0)
    .bind(&version.full_name)
    .bind(version.is_active)
    .bind(&version.name)
    .bind(if version.website_url.is_empty() {
        None
    } else {
        Some(&version.website_url)
    })
    .bind(version.version_number.major as u32)
    .bind(version.version_number.minor as u32)
    .bind(version.version_number.patch as u32)
    .execute(&mut **transaction)
    .await?;

    Ok(())
}
