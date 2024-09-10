use super::{error::ResponseExt, MarkdownResponse, Result, VersionId, THUNDERSTORE_URL};
use std::fmt::Display;

fn url(tail: impl Display) -> String {
    format!("{}/api/experimental/{}", THUNDERSTORE_URL, tail)
}

pub async fn get_changelog(client: &reqwest::Client, id: &VersionId) -> Result<String> {
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

pub async fn get_readme(client: &reqwest::Client, id: &VersionId) -> Result<String> {
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
