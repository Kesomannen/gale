use std::{
    fmt::{self, Display},
    result::Result as StdResult,
    sync::Mutex,
};

use serde::Serialize;

#[derive(Debug)]
pub struct CommandError(eyre::Error);

impl Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#}", self.0)
    }
}

impl Serialize for CommandError {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<T> From<T> for CommandError
where
    T: Into<eyre::Report>,
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

pub type Result<T> = StdResult<T, CommandError>;

pub type StateMutex<'r, S> = tauri::State<'r, Mutex<S>>;
