use super::*;
use anyhow::Context;
use base64::{prelude::BASE64_STANDARD, Engine};
use bytes::Bytes;
use futures_util::{future::try_join_all, try_join};
use std::fmt::Display;
use uuid::Uuid;

const PROFILE_DATA_PREFIX: &str = "#r2modman\n";

fn url(tail: impl Display) -> String {
    format!("{}/api/experimental/{}", THUNDERSTORE_URL, tail)
}

pub async fn get_changelog(client: &reqwest::Client, id: &VersionId) -> Result<Option<String>> {
    let url = url(format!("package/{}/changelog", id.path()));

    let response = client
        .get(&url)
        .send()
        .await?
        .wrap_err()?
        .json::<MarkdownResponse>()
        .await?;

    Ok(response.markdown)
}

pub async fn get_readme(client: &reqwest::Client, id: &VersionId) -> Result<Option<String>> {
    let url = url(format!("package/{}/readme", id.path()));

    let response = client
        .get(&url)
        .send()
        .await?
        .wrap_err()?
        .json::<MarkdownResponse>()
        .await?;

    Ok(response.markdown)
}

pub async fn create_profile(client: &reqwest::Client, data: impl AsRef<[u8]>) -> Result<Uuid> {
    let base64 = format!("{}{}", PROFILE_DATA_PREFIX, BASE64_STANDARD.encode(data));

    let url = url("legacyprofile/create");

    let response = client
        .post(url)
        .header("Content-Type", "application/octet-stream")
        .body(base64)
        .send()
        .await?
        .wrap_err()?
        .json::<LegacyProfileCreateResponse>()
        .await?;

    Ok(response.key)
}

pub async fn get_profile(client: &reqwest::Client, key: Uuid) -> Result<Vec<u8>> {
    let url = url(format_args!("legacyprofile/get/{}", key));

    let response = client.get(url).send().await?.wrap_err()?.text().await?;

    match response.strip_prefix(PROFILE_DATA_PREFIX) {
        Some(str) => BASE64_STANDARD
            .decode(str)
            .map_err(|_| Error::InvalidProfileFormat),
        None => Err(Error::InvalidProfileFormat),
    }
}

async fn initiate_upload(
    client: &reqwest::Client,
    name: impl Into<String>,
    size: u64,
    token: impl Display,
) -> Result<UserMediaInitiateUploadResponse> {
    let url = url("usermedia/initiate-upload");

    let response = client
        .post(url)
        .bearer_auth(token)
        .json(&UserMediaInitiateUploadParams {
            filename: name.into(),
            file_size_bytes: size,
        })
        .send()
        .await?
        .wrap_err()?
        .json()
        .await?;

    Ok(response)
}

async fn finish_upload(client: &reqwest::Client, uuid: Uuid, parts: Vec<CompletedPart>, token: impl Display) -> Result<UserMedia> {
    let url = url(format_args!("usermedia/{}/finish-upload", uuid));

    let response = client
        .post(url)
        .bearer_auth(token)
        .json(&UserMediaFinishUploadParams { parts })
        .send()
        .await?
        .wrap_err()?
        .json()
        .await?;

    Ok(response)
}

async fn upload_chunk(
    client: reqwest::Client,
    part: UploadPartUrl,
    bytes: Bytes,
) -> Result<CompletedPart> {
    let slice = bytes.slice(part.offset as usize..(part.offset + part.length) as usize);

    let response = client.put(&part.url).body(slice).send().await?.error_for_status()?;

    let tag = response
        .headers()
        .get("ETag")
        .expect("upload url response should have an ETag header")
        .to_str()
        .expect("ETag should be valid UTF-8")
        .to_owned();

    Ok(CompletedPart {
        tag,
        part_number: part.part_number,
    })
}

pub async fn submit_package(
    client: &reqwest::Client,
    upload_uuid: Uuid,
    mut metadata: PackageMetadata,
    token: impl Display,
) -> Result<PackageSubmissionResult> {
    let url = url("submission/submit");
    metadata.upload_uuid = Some(upload_uuid);

    let response = client
        .post(url)
        .bearer_auth(token)
        .json(&metadata)
        .send()
        .await?
        .wrap_err()?
        .json()
        .await?;

    Ok(response)
}

pub async fn publish(
    client: &reqwest::Client,
    name: impl Into<String>,
    data: impl Into<Bytes>,
    metadata: PackageMetadata,
    token: impl Display,
) -> Result<PackageSubmissionResult> {
    let bytes = data.into();
    let response = initiate_upload(client, name, bytes.len() as u64, &token).await?;

    let uuid = response.user_media.uuid.expect("initial user media request should return a uuid");

    let chunks = response
        .upload_urls
        .into_iter()
        .map(|part| upload_chunk(client.clone(), part, bytes.clone()))
        .collect::<Vec<_>>();

    let parts = try_join_all(chunks).await?;

    finish_upload(client, uuid, parts, &token).await?;
    submit_package(client, uuid, metadata, token).await
}
   
