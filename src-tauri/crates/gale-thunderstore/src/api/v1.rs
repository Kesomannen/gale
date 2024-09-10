use super::{error::ResponseExt, PackageV1, Result, THUNDERSTORE_URL};
use async_stream::try_stream;
use futures_util::Stream;
use std::fmt::Display;

fn url(community: impl Display, tail: impl Display) -> String {
    format!("{}/c/{}/api/v1/{}/", THUNDERSTORE_URL, community, tail)
}

pub async fn stream_packages(
    client: &reqwest::Client,
    community: impl Display,
) -> Result<impl Stream<Item = Result<PackageV1>>> {
    let url = url(community, "package");
    let mut response = client.get(&url).send().await?.wrap_err()?;

    Ok(try_stream! {
        let mut buffer = Vec::new();
        let mut string = String::new();

        let mut is_first = true;

        while let Some(chunk) = response.chunk().await? {
            buffer.extend_from_slice(&chunk);

            let chunk = match std::str::from_utf8(&buffer) {
                Ok(chunk) => chunk,
                Err(_) => continue,
            };

            if is_first {
                is_first = false;
                string.extend(chunk.chars().skip(1)); // remove leading [
            } else {
                string.push_str(chunk);
            }

            buffer.clear();

            while let Some(index) = string.find("}]},") {
                let (json, _) = string.split_at(index + 3);
                yield serde_json::from_str::<PackageV1>(json)?;
                string.replace_range(..index + 4, "");
            }
        }
    })
}
