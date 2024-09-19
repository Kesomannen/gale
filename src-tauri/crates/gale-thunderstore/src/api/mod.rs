use error::ResponseExt;
use futures_util::{Stream, StreamExt};

mod id;
pub use id::*;

mod models;
pub use models::*;

mod error;
pub use error::{ApiResultExt, Error, Result};

mod experimental;
pub use experimental::*;

mod v1;
pub use v1::*;

mod cyberstorm;
pub use cyberstorm::*;

pub const THUNDERSTORE_URL: &str = "https://thunderstore.io";

pub async fn download(
    client: &reqwest::Client,
    id: &VersionId,
) -> Result<(u64, impl Stream<Item = Result<bytes::Bytes>>)> {
    let url = format!("{}/package/download/{}/", THUNDERSTORE_URL, id.path());

    let response = client.get(&url).send().await?.wrap_err()?;

    let len = response.content_length().unwrap_or_default();
    let stream = response
        .bytes_stream()
        .map(|res| res.map_err(Error::Reqwest));

    Ok((len, stream))
}