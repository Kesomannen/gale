use std::{
    collections::HashMap,
    fmt::Display,
    io::{Cursor, Seek, Write},
    path::{Path, PathBuf},
};

use bytes::Bytes;
use eyre::{anyhow, bail, ensure, eyre, Context, OptionExt, Result};
use futures_util::future::try_join_all;
use image::{imageops::FilterType, ImageFormat};
use itertools::Itertools;
use log::{debug, info, trace};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tauri::Url;
use uuid::Uuid;
use zip::{write::SimpleFileOptions, ZipWriter};

use crate::{game::Game, profile::Profile, thunderstore::*};

pub fn refresh_args(profile: &mut Profile) {
    if profile.modpack.is_none() {
        profile.modpack = Some(ModpackArgs {
            name: profile.name.replace([' ', '-'], ""),
            readme: format!("# {}\n\n", profile.name),
            changelog: "# Changelog\n\n## 1.0.0\n\n- Initial release".to_owned(),
            version_number: "1.0.0".to_owned(),
            ..Default::default()
        });
    }

    let includes = &mut profile.modpack.as_mut().unwrap().include_files;

    // remove deleted files
    includes.retain(|file, _| profile.path.join(file).exists());

    for path in super::find_includes(&profile.path) {
        includes.entry(path).or_insert(true);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ModpackArgs {
    pub name: String,
    pub description: String,
    pub author: String,
    pub categories: Vec<String>,
    pub nsfw: bool,
    pub readme: String,
    #[serde(default)]
    pub changelog: String,
    pub version_number: String,
    pub icon_path: PathBuf,
    pub website_url: String,
    pub include_disabled: bool,
    #[serde(default, rename = "includeFileMap")]
    pub include_files: HashMap<PathBuf, bool>,
}

impl Profile {
    pub(super) fn mods_to_pack<'a>(
        &'a self,
        args: &'a ModpackArgs,
    ) -> impl Iterator<Item = &'a ModId> + 'a {
        self.thunderstore_mods()
            .filter(move |(_, enabled)| args.include_disabled || *enabled)
            .map(|(ts_mod, _)| &ts_mod.id)
    }

    pub fn export_pack(
        &self,
        args: &ModpackArgs,
        writer: impl Write + Seek,
        thunderstore: &Thunderstore,
    ) -> Result<()> {
        ensure!(!args.name.is_empty(), "name cannot be empty");
        ensure!(!args.description.is_empty(), "description cannot be empty");

        let deps = self
            .mods_to_pack(args)
            .map(|mod_ref| {
                let borrowed = mod_ref.borrow(thunderstore)?;
                Ok(borrowed.version.ident.clone())
            })
            .collect::<Result<Vec<_>>>()
            .context("failed to resolve modpack dependencies")?;

        let version_number =
            semver::Version::parse(&args.version_number).context("invalid version number")?;

        let manifest = PackageManifest {
            name: args.name.clone(),
            description: args.description.clone(),
            website_url: args.website_url.clone(),
            dependencies: deps,
            installers: None,
            author: None,
            version_number,
        };

        let mut zip = ZipWriter::new(writer);

        if !args.readme.is_empty() {
            trace!("writing readme");
            zip.start_file("README.md", SimpleFileOptions::default())?;
            zip.write_all(args.readme.as_bytes())?;
        }

        if !args.changelog.is_empty() {
            trace!("writing changelog");
            zip.start_file("CHANGELOG.md", SimpleFileOptions::default())?;
            zip.write_all(args.changelog.as_bytes())?;
        }

        trace!("writing manifest");
        zip.start_file("manifest.json", SimpleFileOptions::default())?;
        serde_json::to_writer_pretty(&mut zip, &manifest)?;

        write_icon(&args.icon_path, &mut zip).context("failed to write icon")?;

        super::write_includes(
            args.include_files
                .iter()
                .filter(|(_, enabled)| **enabled)
                .map(|(file, _)| file),
            &self.path,
            &mut zip,
        )?;

        Ok(())
    }
}

fn write_icon<W>(path: &Path, zip: &mut ZipWriter<W>) -> Result<()>
where
    W: Write + Seek,
{
    let img = image::ImageReader::open(path)?.decode()?;
    let img = img.resize_exact(256, 256, FilterType::Lanczos3);

    let mut bytes = Vec::new();
    img.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)?;
    zip.start_file("icon.png", SimpleFileOptions::default())?;
    zip.write_all(&bytes)?;

    Ok(())
}

fn base_request(
    tail: impl Display,
    token: impl Display,
    client: &reqwest::Client,
) -> reqwest::RequestBuilder {
    let url = format!("https://thunderstore.io/api/experimental/{}/", tail);

    client.post(url).bearer_auth(token)
}

pub async fn publish(
    data: Bytes,
    game: Game,
    args: ModpackArgs,
    token: String,
    client: reqwest::Client,
) -> Result<()> {
    ensure!(args.description.len() <= 250, "description is too long");
    ensure!(!args.readme.is_empty(), "readme cannot be empty");
    ensure!(!args.author.is_empty(), "author cannot be empty");

    if !args.website_url.is_empty() {
        Url::parse(&args.website_url).context("invalid website URL")?;
    }

    info!("publishing modpack");

    let response = initiate_upload(args.name.clone(), data.len() as u64, &token, &client)
        .await
        .context("failed to initiate upload")?;

    let uuid = response.user_media.uuid.ok_or_eyre("no uuid in response")?;

    let tasks = response.upload_urls.into_iter().map(|part| {
        let data = data.clone();
        let client = client.clone();
        tauri::async_runtime::spawn(upload_chunk(part, data, client))
    });

    let parts = match try_join_all(tasks)
        .await
        .map_err(|err| anyhow!(err))
        .and_then(|parts| parts.into_iter().collect::<Result<Vec<_>>>())
    {
        Ok(parts) => parts,
        Err(err) => {
            tauri::async_runtime::spawn(async move { abort_upload(&uuid, &token, client).await });
            return Err(err.wrap_err("failed to upload file"));
        }
    };

    finish_upload(parts, &uuid, &token, &client)
        .await
        .context("failed to finalize upload")?;

    submit_package(uuid, game, args, &token, &client)
        .await
        .context("failed to submit package")?;

    Ok(())
}

async fn initiate_upload(
    name: String,
    size: u64,
    token: &str,
    client: &reqwest::Client,
) -> Result<UserMediaInitiateUploadResponse> {
    debug!(
        "initiating modpack upload for {}, size: {} bytes",
        name, size
    );

    let response = base_request("usermedia/initiate-upload", token, client)
        .json(&UserMediaInitiateUploadParams {
            filename: name,
            file_size_bytes: size,
        })
        .send()
        .await?
        .map_auth_err()?
        .json::<UserMediaInitiateUploadResponse>()
        .await?;

    debug!("recieved {} upload urls", response.upload_urls.len());

    Ok(response)
}

async fn upload_chunk(
    part: UploadPartUrl,
    data: Bytes,
    client: reqwest::Client,
) -> Result<CompletedPart> {
    let start = part.offset as usize;
    let end = start + part.length as usize;
    let chunk = data.slice(start..end);

    let response = client
        .put(&part.url)
        .body(chunk)
        .send()
        .await?
        .error_for_status()?;

    let tag = response
        .headers()
        .get("ETag")
        .ok_or_eyre("no ETag in response")?
        .to_str()
        .context("ETag is not valid utf-8")?
        .to_owned();

    debug!("uploaded part {} with tag {}", part.part_number, tag);

    Ok(CompletedPart {
        tag,
        part_number: part.part_number,
    })
}

async fn abort_upload(uuid: &Uuid, token: &str, client: reqwest::Client) -> Result<()> {
    info!("aborting upload");

    base_request(format!("usermedia/{}/abort-upload", uuid), token, &client)
        .json(&uuid)
        .send()
        .await?
        .map_auth_err()?;

    Ok(())
}

async fn finish_upload(
    parts: Vec<CompletedPart>,
    uuid: &Uuid,
    token: &str,
    client: &reqwest::Client,
) -> Result<()> {
    debug!("finishing upload");

    base_request(format!("usermedia/{}/finish-upload", uuid), token, client)
        .json(&UserMediaFinishUploadParams { parts })
        .send()
        .await?
        .map_auth_err()?;

    Ok(())
}

async fn submit_package(
    uuid: Uuid,
    game: Game,
    args: ModpackArgs,
    token: &str,
    client: &reqwest::Client,
) -> Result<()> {
    let metadata = PackageSubmissionMetadata {
        author_name: args.author,
        has_nsfw_content: args.nsfw,
        upload_uuid: uuid,
        categories: Vec::new(),
        communities: vec![game.slug.to_string()],
        community_categories: HashMap::from([(game.slug.to_string(), args.categories)]),
    };

    debug!("submitting package");

    let response = base_request("submission/submit", token, client)
        .json(&metadata)
        .send()
        .await?;

    let status = response.status();

    if response.status().is_success() {
        return Ok(());
    }

    if status == StatusCode::BAD_REQUEST {
        if let Ok(Some(err)) = handle_bad_request(response).await {
            bail!("{}", err)
        }
    }

    bail!("unexpected error: {}", status);

    async fn handle_bad_request(response: reqwest::Response) -> Result<Option<String>> {
        #[derive(Deserialize)]
        struct Error {
            file: Vec<String>,
        }

        let err = response.json::<Error>().await?;

        if err.file.is_empty() {
            return Ok(None);
        }

        Ok(Some(
            err.file
                .into_iter()
                .map(|err| match err.split_once(':') {
                    Some((_field, msg)) => msg.trim().to_owned(),
                    None => err,
                })
                .collect_vec()
                .join(", "),
        ))
    }
}

trait ReqwestResponseExt {
    fn map_auth_err_with<F>(self, f: F) -> eyre::Result<reqwest::Response>
    where
        F: FnOnce(StatusCode) -> Option<eyre::Error>;

    fn map_auth_err(self) -> eyre::Result<reqwest::Response>;
}

impl ReqwestResponseExt for reqwest::Response {
    fn map_auth_err_with<F>(self, f: F) -> eyre::Result<reqwest::Response>
    where
        F: FnOnce(StatusCode) -> Option<eyre::Error>,
    {
        self.error_for_status().map_err(|err| match err.status() {
            Some(status) => match status {
                StatusCode::UNAUTHORIZED => eyre!("thunderstore API token is invalid"),
                _ => match f(status) {
                    Some(err) => err,
                    None => eyre!(err),
                },
            },
            None => eyre!(err),
        })
    }

    fn map_auth_err(self) -> eyre::Result<reqwest::Response> {
        self.map_auth_err_with(|_| None)
    }
}
