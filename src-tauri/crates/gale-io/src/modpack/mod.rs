use std::{
    collections::HashMap,
    io::{BufWriter, Cursor, Seek, Write},
    path::{Path, PathBuf},
};

use anyhow::{ensure, Context};
use futures_util::try_join;
use gale_core::prelude::*;
use gale_profile::ProfileModSource;
use gale_thunderstore::api::{PackageManifest, PackageMetadata, VersionId};
use image::{imageops::FilterType, ImageFormat};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use url::Url;
use zip::{write::SimpleFileOptions, ZipWriter};

mod changelog;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ModpackArgs {
    name: String,
    description: String,
    author: String,
    categories: Vec<String>,
    nsfw: bool,
    readme: String,
    changelog: String,
    version_number: String,
    icon_path: PathBuf,
    website_url: String,
    include_disabled: bool,
    include_files: HashMap<PathBuf, bool>,
}

pub async fn export_to_file(
    profile_id: i64,
    path: &Path,
    args: &ModpackArgs,
    state: &AppState,
) -> Result<()> {
    let file = std::fs::File::create(path)
        .map(BufWriter::new)
        .context("failed to create file")?;

    export_to_writer(profile_id, file, args, state).await?;

    Ok(())
}

pub async fn export_and_publish(
    profile_id: i64,
    args: ModpackArgs,
    community_id: i64,
    state: &AppState,
) -> Result<()> {
    let mut buff = Cursor::new(Vec::new());
    export_to_writer(profile_id, &mut buff, &args, state).await?;

    let token = String::new();
    publish(buff.into_inner(), args, &token, community_id, state).await?;

    Ok(())
}

async fn export_to_writer(
    profile_id: i64,
    writer: impl Write + Seek,
    args: &ModpackArgs,
    state: &AppState,
) -> Result<()> {
    ensure!(!args.name.is_empty(), "name cannot be empty");
    ensure!(!args.description.is_empty(), "description cannot be empty");

    let (profile_path, dependencies) = try_join!(
        state.profile_path(profile_id),
        get_dependencies(profile_id, args, state)
    )?;

    let version_number =
        semver::Version::parse(&args.version_number).context("invalid version number")?;

    let manifest = PackageManifest {
        name: args.name.clone(),
        description: args.description.clone(),
        website_url: args.website_url.clone(),
        dependencies,
        installers: None,
        author: None,
        version_number,
    };

    let mut zip = ZipWriter::new(writer);

    if !args.readme.is_empty() {
        zip.start_file("README.md", SimpleFileOptions::default())?;
        write!(zip, "{}", args.readme)?;
    }

    if !args.changelog.is_empty() {
        zip.start_file("CHANGELOG.md", SimpleFileOptions::default())?;
        write!(zip, "{}", args.changelog)?;
    }

    zip.start_file("manifest.json", SimpleFileOptions::default())?;
    serde_json::to_writer_pretty(&mut zip, &manifest)?;

    write_icon(&args.icon_path, &mut zip).context("failed to write icon")?;

    super::export::write_config(
        args.include_files
            .iter()
            .filter(|(_, enabled)| **enabled)
            .map(|(file, _)| file),
        &profile_path,
        &mut zip,
    )?;

    Ok(())
}

async fn get_dependencies(
    profile_id: i64,
    args: &ModpackArgs,
    state: &AppState,
) -> Result<Vec<VersionId>> {
    sqlx::query!(
        r#"
        SELECT source AS "source: Json<ProfileModSource>"
        FROM profile_mods
        WHERE 
            profile_id = ? AND
            json_extract (source, '$.type') = 'thunderstore' AND
            (? = 1 OR enabled = 1)
        "#,
        profile_id,
        args.include_disabled
    )
    .map(|record| match record.source.0 {
        ProfileModSource::Thunderstore { identifier, .. } => identifier,
        _ => unreachable!(),
    })
    .fetch_all(&state.db)
    .await
    .map_into()
}

fn write_icon<W: Write + Seek>(path: &Path, zip: &mut ZipWriter<W>) -> Result<()> {
    let img = image::ImageReader::open(path)?.decode()?;
    let img = img.resize_exact(256, 256, FilterType::Lanczos3);

    let mut writer = Cursor::new(Vec::new());
    img.write_to(&mut writer, ImageFormat::Png)?;

    zip.start_file("icon.png", SimpleFileOptions::default())?;
    zip.write_all(&writer.into_inner())?;

    Ok(())
}

async fn publish(
    data: Vec<u8>,
    args: ModpackArgs,
    token: &str,
    community_id: i64,
    state: &AppState,
) -> Result<()> {
    ensure!(args.description.len() <= 250, "description is too long");
    ensure!(!args.readme.is_empty(), "readme cannot be empty");
    ensure!(!args.author.is_empty(), "author cannot be empty");

    if !args.website_url.is_empty() {
        Url::parse(&args.website_url).context("invalid website URL")?;
    }

    let slug = sqlx::query!("SELECT slug FROM communities WHERE id = ?", community_id)
        .fetch_optional(&state.db)
        .await?
        .context("community not found")?
        .slug;

    let ModpackArgs {
        name,
        author,
        categories,
        nsfw,
        ..
    } = args;

    let metadata = PackageMetadata {
        author,
        global_categories: Vec::new(),
        communities: vec![slug.clone()],
        has_nsfw_content: nsfw,
        categories: [(slug, categories)].into(),
        upload_uuid: None,
    };

    gale_thunderstore::api::publish(&state.reqwest, name, data, metadata, token).await?;

    Ok(())
}
