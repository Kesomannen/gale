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

pub trait ResultExt<T> {
    /// Maps `NotFound` errors to `Ok(None)`
    fn not_found_ok(self) -> Result<Option<T>>;
}

impl<T> ResultExt<T> for Result<T> {
    fn not_found_ok(self) -> Result<Option<T>> {
        match self {
            Ok(value) => Ok(Some(value)),
            Err(Error::NotFound) => Ok(None),
            Err(err) => Err(err),
        }
    }
}
