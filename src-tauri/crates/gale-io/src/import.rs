use crate::{LegacyProfileManifest, LegacyProfileMod, LegacyProfileModKind};
use anyhow::{anyhow, Context};
use futures_util::future::try_join_all;
use gale_core::prelude::*;
use gale_profile::install::{InstallMetadata, InstallQueue, InstallSource};
use gale_thunderstore::api::{ApiResultExt, VersionId};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use std::{
    io::{BufReader, Cursor, Read, Seek},
    path::{Path, PathBuf},
};
use tauri::{AppHandle, Manager};
use tempfile::tempdir;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModImport {
    source: InstallSource,
    enabled: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ImportTarget {
    #[serde(rename_all = "camelCase")]
    New {
        name: String,
        path: PathBuf,
        community_id: i64,
    },
    #[serde(rename_all = "camelCase")]
    Overwrite { id: i64 },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImportData {
    mods: Vec<ModImport>,
    source_path: PathBuf,
    delete_after_import: bool,
}

pub async fn import(
    data: ImportData,
    target: ImportTarget,
    state: &AppState,
    app: &AppHandle,
) -> Result<()> {
    let ImportData {
        mods,
        source_path,
        delete_after_import,
    } = data;

    let (profile_id, profile_path) = match target {
        ImportTarget::New {
            name,
            path,
            community_id,
        } => {
            let id = gale_profile::create(&name, &path, community_id, state)
                .await
                .context("failed to create profile")?;

            (id, path)
        }
        ImportTarget::Overwrite { id } => {
            let path: PathBuf = sqlx::query!("SELECT path FROM profiles WHERE id = ?", id)
                .fetch_optional(&state.db)
                .await?
                .context("profile not found")?
                .path
                .into();

            std::fs::remove_dir_all(&path).context("failed to clear profile")?;
            std::fs::create_dir(&path).context("failed to recreate profile")?;

            (id, path)
        }
    };

    import_config(&profile_path, &source_path).context("failed to import config")?;

    let queue = app.state::<InstallQueue>();

    queue.enqueue_many(mods.into_iter().enumerate().map(|(i, import)| {
        let metadata = InstallMetadata::new(profile_id)
            .with_state(import.enabled)
            .with_index(i + 1); // sql uses 1-based indexing

        (import.source, metadata)
    }));

    if delete_after_import {
        std::fs::remove_dir_all(&source_path).context("failed to delete source")?;
    }

    Ok(())
}

fn import_config(profile_path: &Path, src: &Path) -> Result<()> {
    for file in super::find_config_files(src) {
        let source = src.join(&file);

        let target = match file.starts_with("config") {
            true => profile_path.join("BepInEx").join(file),
            false => profile_path.join(file),
        };

        let parent = target.parent().unwrap();
        std::fs::create_dir_all(parent)?;
        std::fs::copy(&source, &target)?;
    }

    Ok(())
}

pub async fn read_file(path: &Path, state: &AppState) -> Result<(String, ImportData)> {
    let reader = std::fs::File::open(path)
        .map(BufReader::new)
        .context("failed to open file")?;

    read_data(reader, state).await
}

pub async fn read_code(key: Uuid, state: &AppState) -> Result<(String, ImportData)> {
    let data = gale_thunderstore::api::get_profile(&state.reqwest, key)
        .await
        .map_404_to_none()
        .context("failed to fetch profile")?
        .context("profile code is expired or invalid")?;

    read_data(Cursor::new(data), state).await
}

async fn read_data(src: impl Read + Seek, state: &AppState) -> Result<(String, ImportData)> {
    let temp_dir = tempdir().context("failed to create temporary directory")?;
    gale_core::util::extract(src, temp_dir.path())?;

    let manifest = std::fs::File::open(temp_dir.path().join("export.r2x"))
        .map(BufReader::new)
        .context("failed to read manifest")?;

    let LegacyProfileManifest {
        profile_name,
        mods,
        source: _,
    } = serde_yaml_ng::from_reader(manifest).context("failed to parse manifest")?;

    let futures = mods
        .into_iter()
        .map(|r2mod| r2_to_import(r2mod, state))
        .collect::<Vec<_>>();

    let mods = try_join_all(futures).await?;

    let result = ImportData {
        mods,
        source_path: temp_dir.into_path(),
        delete_after_import: true,
    };

    Ok((profile_name, result))
}

async fn r2_to_import(r2mod: LegacyProfileMod, state: &AppState) -> Result<ModImport> {
    let LegacyProfileMod {
        id: identifier,
        kind,
        enabled,
    } = r2mod;

    let (owner, name) = identifier.split();

    let source = match kind {
        LegacyProfileModKind::Default { version } => {
            let version_uuid = get_version_uuid(owner, name, &version, state)
                .await?
                .ok_or_else(|| anyhow!("package not found: {}", identifier))?;

            let identifier: VersionId =
                (owner, name, version.major, version.minor, version.patch).into();

            InstallSource::Thunderstore {
                identifier,
                version_uuid,
            }
        }
        LegacyProfileModKind::Github { tag } => InstallSource::Github {
            owner: owner.to_string(),
            repo: name.to_string(),
            tag,
        },
    };

    Ok(ModImport { source, enabled })
}

async fn get_version_uuid(
    owner: &str,
    name: &str,
    version: &crate::R2Version,
    state: &AppState,
) -> Result<Option<Uuid>> {
    let record = sqlx::query!(
        r#"
                    SELECT
                        v.id AS "id: Uuid"
                    FROM
                        versions v
                        JOIN packages p
                            ON v.package_id = p.id
                    WHERE
                        p.owner = ? AND
                        p.name = ? AND
                        v.major = ? AND
                        v.minor = ? AND
                        v.patch = ?
                    "#,
        owner,
        name,
        version.major,
        version.minor,
        version.patch
    )
    .fetch_optional(&state.db)
    .await?;

    Ok(record.map(|record| record.id))
}
