use error::ResponseExt;
use futures_util::{Stream, StreamExt};

mod id;
pub use id::{PackageId, SyntaxError, VersionId};

mod models;
pub use models::*;

mod error;
pub use error::{Error, Result, ResultExt};

mod experimental;
pub use experimental::*;

mod v1;
pub use v1::*;

pub const THUNDERSTORE_URL: &str = "https://thunderstore.io";

pub async fn download(
    client: &reqwest::Client,
    id: &VersionId,
) -> Result<impl Stream<Item = Result<bytes::Bytes>>> {
    let url = format!("{}/package/download/{}/", THUNDERSTORE_URL, id.path());

    Ok(client
        .get(&url)
        .send()
        .await?
        .wrap_err()?
        .bytes_stream()
        .map(|res| res.map_err(Error::Reqwest)))
}
