use reqwest::StatusCode;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("resource not found")]
    NotFound,

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait ResponseExt: Sized {
    fn wrap_err(self) -> Result<Self>;
}

impl ResponseExt for reqwest::Response {
    fn wrap_err(self) -> Result<Self> {
        match self.error_for_status() {
            Ok(res) => Ok(res),
            Err(err) => match err.status() {
                Some(StatusCode::NOT_FOUND) => Err(Error::NotFound),
                _ => Err(Error::Reqwest(err)),
            },
        }
    }
}
