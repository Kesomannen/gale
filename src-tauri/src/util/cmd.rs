use std::result::Result as StdResult;

use serde::Serialize;
use serde_json::json;

#[derive(Debug)]
pub struct CommandError(eyre::Error);

impl Serialize for CommandError {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let obj = json!({
            "message": format!("{:#}", self.0),
            "detail": format!("{:?}", self.0)
        });

        obj.serialize(serializer)
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
