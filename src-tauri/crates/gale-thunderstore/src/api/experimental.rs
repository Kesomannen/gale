use super::{
    error::ResponseExt, Error, LegacyProfileCreateResponse, MarkdownResponse, Result, VersionId,
    THUNDERSTORE_URL,
};
use base64::{prelude::BASE64_STANDARD, Engine};
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
