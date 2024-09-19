use crate::{
    LegacyProfileManifest, LegacyProfileMod, LegacyProfileModKind, LegacyVersion, ModManager,
};
use anyhow::Context;
use gale_core::prelude::*;
use gale_profile::ProfileModSource;
use sqlx::types::Json;
use std::{
    io::{BufWriter, Cursor, Seek, Write},
    path::{Path, PathBuf},
};
use uuid::Uuid;
use zip::{write::SimpleFileOptions, ZipWriter};

pub async fn as_code(profile_id: i64, state: &AppState) -> Result<Uuid> {
    let mut writer = Cursor::new(Vec::new());
    to_zip(profile_id, &mut writer, state).await?;

    let key = gale_thunderstore::api::create_profile(&state.reqwest, writer.into_inner())
        .await
        .context("failed to upload profile")?;

    Ok(key)
}

pub async fn to_file(profile_id: i64, path: impl AsRef<Path>, state: &AppState) -> Result<()> {
    let file = std::fs::File::create(path)
        .map(BufWriter::new)
        .context("failed to create file")?;

    to_zip(profile_id, file, state).await?;

    Ok(())
}

async fn to_zip(profile_id: i64, writer: impl Write + Seek, state: &AppState) -> Result<()> {
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
    .map(|record| LegacyProfileMod::from_source(record.enabled, record.source.0))
    .fetch_all(&state.db)
    .await?;

    let manifest = LegacyProfileManifest {
        profile_name: profile.name,
        source: ModManager::Gale,
        mods,
    };

    zip.start_file("export.r2x", SimpleFileOptions::default())?;
    serde_yaml_ng::to_writer(&mut zip, &manifest).context("failed to write profile manifest")?;

    let path: PathBuf = profile.path.into();
    write_config(super::find_config_files(&path), &path, &mut zip)?;

    Ok(())
}

pub fn write_config<P, I, W>(files: I, profile_path: &Path, zip: &mut ZipWriter<W>) -> Result<()>
where
    P: AsRef<Path>,
    I: IntoIterator<Item = P>,
    W: Write + Seek,
{
    for file in files {
        zip.start_file_from_path(&file, SimpleFileOptions::default())?;

        let mut reader = std::fs::File::open(profile_path.join(file))?;
        std::io::copy(&mut reader, zip)?;
    }

    Ok(())
}

impl LegacyProfileMod {
    fn from_source(enabled: bool, source: ProfileModSource) -> Self {
        let (name, kind) = match source {
            ProfileModSource::Thunderstore { identifier, .. } => {
                let (major, minor, patch) = identifier.version_split();
                let kind = LegacyProfileModKind::Default {
                    version: LegacyVersion {
                        major,
                        minor,
                        patch,
                    },
                };

                (identifier.into_string(), kind)
            }
            ProfileModSource::Github { owner, repo, tag } => {
                let kind = LegacyProfileModKind::Github { tag };

                (format!("{}-{}", owner, repo), kind)
            }
            ProfileModSource::Local {
                name: _,
                version: _,
            } => {
                todo!()
            }
        };

        Self {
            name,
            enabled,
            kind,
        }
    }
}
