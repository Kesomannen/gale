use crate::ProfileModSource;
use anyhow::anyhow;
use futures_util::TryStreamExt;
use gale_core::prelude::*;
use serde::Serialize;
use sqlx::types::Json;

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProfileInfo {
    name: String,
    path: String,
    community_id: i64,
    mods: Vec<ProfileModInfo>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProfileModInfo {
    id: i64,
    index: i64,
    name: String,
    version: String,
    enabled: bool,
    href: String,
    kind: ProfileModKind,
}

#[derive(Serialize, Debug, Clone)]
pub enum ProfileModKind {
    Thunderstore,
    Local,
}

pub async fn single(id: i64, state: &AppState) -> Result<ProfileInfo> {
    let (name, path, community_id, community_slug) = sqlx::query!(
        "SELECT
            p.name,
            p.path,
            c.id,
            c.slug
        FROM
            profiles p
            JOIN communities c ON p.community_id = c.id
        WHERE p.id = ?
        ",
        id
    )
    .map(|record| (record.name, record.path, record.id, record.slug))
    .fetch_optional(&state.db)
    .await?
    .ok_or(anyhow!("profile not found"))?;

    let mut stream = sqlx::query!(
        r#"SELECT
            id,
            enabled,
            order_index,
            source AS "source: Json<ProfileModSource>"
        FROM profile_mods
        WHERE profile_id = ?"#,
        id
    )
    .fetch(&state.db);

    let mut mods = Vec::new();

    while let Some(record) = stream.try_next().await? {
        let kind = match record.source.0 {
            ProfileModSource::Thunderstore { .. } => ProfileModKind::Thunderstore,
            ProfileModSource::Local { .. } => ProfileModKind::Local,
        };

        let (name, version, href) = match record.source.0 {
            ProfileModSource::Thunderstore { identifier, .. } => {
                let href = format!(
                    "{}/c/{}/p/{}/",
                    gale_thunderstore::api::THUNDERSTORE_URL,
                    community_slug,
                    identifier.path()
                );

                (
                    identifier.name().to_owned(),
                    identifier.version().to_owned(),
                    href,
                )
            }
            ProfileModSource::Local { id: _ } => todo!(),
        };

        mods.push(ProfileModInfo {
            id: record.id,
            index: record.order_index,
            name,
            version,
            enabled: record.enabled,
            href,
            kind,
        });
    }

    Ok(ProfileInfo {
        name,
        path,
        community_id,
        mods,
    })
}
