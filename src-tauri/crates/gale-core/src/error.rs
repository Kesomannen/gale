use serde::Serialize;
use std::fmt::{Debug, Display};

pub type Error = anyhow::Error;
pub type Result<T> = std::result::Result<T, Error>;

pub struct CmdError(anyhow::Error);

impl CmdError {
    pub fn new(error: anyhow::Error) -> Self {
        Self(error)
    }

    pub fn into_inner(self) -> anyhow::Error {
        self.0
    }

    pub fn as_inner(&self) -> &anyhow::Error {
        &self.0
    }
}

impl AsRef<anyhow::Error> for CmdError {
    fn as_ref(&self) -> &anyhow::Error {
        &self.0
    }
}

impl Display for CmdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#}", self.0)
    }
}

impl Debug for CmdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self.0)
    }
}

impl From<anyhow::Error> for CmdError {
    fn from(error: anyhow::Error) -> Self {
        Self(error)
    }
}

impl std::error::Error for CmdError {}

impl Serialize for CmdError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{:#}", self.0))
    }
}

pub type CmdResult<T> = std::result::Result<T, CmdError>;
