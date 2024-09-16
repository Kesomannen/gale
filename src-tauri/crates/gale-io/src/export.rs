use crate::{LegacyProfileManifest, LegacyProfileMod, LegacyProfileModKind, ModManager, R2Version};
use anyhow::Context;
use gale_core::prelude::*;
use gale_profile::ProfileModSource;
use gale_thunderstore::api::PackageId;
use sqlx::types::Json;
use std::{
    io::{BufWriter, Cursor, Seek, Write},
    path::{Path, PathBuf},
};
use uuid::Uuid;
use zip::{write::SimpleFileOptions, ZipWriter};

pub async fn as_code(profile_id: i64, state: &AppState) -> Result<Uuid> {
    let mut writer = Cursor::new(Vec::new());
    write_zip(profile_id, &mut writer, state).await?;

    let key = gale_thunderstore::api::create_profile(&state.reqwest, writer.into_inner())
        .await
        .context("failed to upload profile")?;

    Ok(key)
}

pub async fn to_file(profile_id: i64, path: impl AsRef<Path>, state: &AppState) -> Result<()> {
    let file = std::fs::File::create(path)
        .map(BufWriter::new)
        .context("failed to create file")?;

    write_zip(profile_id, file, state).await?;

    Ok(())
}

async fn write_zip(profile_id: i64, writer: impl Write + Seek, state: &AppState) -> Result<()> {
    let mut zip = ZipWriter::new(writer);

    let profile = sqlx::query!("SELECT name, path FROM profiles WHERE id = ?", profile_id)
        .fetch_one(&state.db)
        .await?;

    let mods = sqlx::query!(
        r#"
        SELECT
            enabled,
            source AS "source: Json<ProfileModSource>"
        FROM profile_mods
        WHERE profile_id = ?
        "#,
        profile_id
    )
    .map(|record| {
        let enabled = record.enabled;

        let (id, kind) = match record.source.0 {
            ProfileModSource::Thunderstore { identifier, .. } => {
                let (major, minor, patch) = identifier.version_split();
                let kind = LegacyProfileModKind::default(major, minor, patch);

                (PackageId::from(identifier), kind)
            }
            ProfileModSource::Github { owner, repo, tag } => {
                let id = PackageId::new(&owner, &repo);
                let kind = LegacyProfileModKind::github(tag);

                (id, kind)
            }
            ProfileModSource::Local { full_name, version } => {
                todo!()
            }
        };

        LegacyProfileMod { id, enabled, kind }
    })
    .fetch_all(&state.db)
    .await?;

    let manifest = LegacyProfileManifest {
        profile_name: profile.name,
        source: ModManager::Gale,
        mods,
    };

    zip.start_file("export.r2x", SimpleFileOptions::default())?;
    serde_yaml::to_writer(&mut zip, &manifest).context("failed to write profile manifest")?;

    write_config(&PathBuf::from(profile.path), &mut zip)?;

    Ok(())
}

fn write_config<W>(profile_path: &Path, zip: &mut ZipWriter<W>) -> Result<()>
where
    W: Write + Seek,
{
    for file in super::find_config_files(profile_path) {
        let path = file.to_string_lossy().replace('\\', "/");
        zip.start_file(path, SimpleFileOptions::default())?;

        let mut reader = std::fs::File::open(profile_path.join(file))?;
        std::io::copy(&mut reader, zip)?;
    }

    Ok(())
}
