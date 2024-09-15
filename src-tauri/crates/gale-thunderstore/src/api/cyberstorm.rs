use super::{error::ResponseExt, FiltersResponse, Result, THUNDERSTORE_URL};
use std::fmt::Display;

fn url(tail: impl Display) -> String {
    format!("{}/api/cyberstorm/{}", THUNDERSTORE_URL, tail)
}

pub async fn get_filters(
    client: &reqwest::Client,
    community: impl Display,
) -> Result<FiltersResponse> {
    let url = url(format!("community/{}/filters", community));

    let response = client
        .get(&url)
        .send()
        .await?
        .wrap_err()?
        .json::<FiltersResponse>()
        .await?;

    Ok(response)
}
