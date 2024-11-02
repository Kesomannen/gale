use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::thunderstore::VersionIdent;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct LocalMod {
    pub name: String,
    pub icon: Option<PathBuf>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub version: Option<semver::Version>,
    pub dependencies: Option<Vec<VersionIdent>>,
    pub uuid: Uuid,
    #[serde(default)]
    pub file_size: u64,
}

impl LocalMod {
    pub fn ident(&self) -> VersionIdent {
        let version = self.version.as_ref().map(|vers| vers.to_string());

        VersionIdent::new(
            self.author.as_deref().unwrap_or(""),
            &self.name,
            version.as_deref().unwrap_or(""),
        )
    }
}
