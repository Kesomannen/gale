use std::fmt::{self, Display};

use serde::Serialize;

#[derive(Debug)]
pub struct CommandError(anyhow::Error);

impl std::error::Error for CommandError {}

impl Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#}", self.0)
    }
}

impl Serialize for CommandError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{:#}", self.0))
    }
}

impl From<anyhow::Error> for CommandError {
    fn from(error: anyhow::Error) -> Self {
        Self(error)
    }
}

pub type CommandResult<T> = std::result::Result<T, CommandError>;