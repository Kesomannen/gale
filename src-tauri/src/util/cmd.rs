use std::{
    fmt::{self, Display},
    sync::Mutex,
};

use serde::Serialize;

#[derive(Debug)]
pub struct CommandError(pub anyhow::Error);

impl std::error::Error for CommandError {}

impl Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#}", self.0)
    }
}

impl Serialize for CommandError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<anyhow::Error> for CommandError {
    fn from(error: anyhow::Error) -> Self {
        Self(error)
    }
}

pub type Result<T> = std::result::Result<T, CommandError>;

pub type StateMutex<'r, S> = tauri::State<'r, Mutex<S>>;
