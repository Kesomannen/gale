use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::paths::DeployPathBuf;

pub const FILE_NAME: &str = ".gale-server-manifest.json";
pub const VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ManifestEntry {
    pub hash: String,
    pub size: u64,
}

/// Records which remote files Gale deployed and therefore owns.
///
/// Only files listed here may be removed from the server, and only files
/// inside the spec's mirror directories are considered for removal on sight;
/// everything else is removed solely because a previous manifest recorded it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct DeploymentManifest {
    pub version: u32,
    pub files: BTreeMap<DeployPathBuf, ManifestEntry>,
}

impl Default for DeploymentManifest {
    fn default() -> Self {
        Self {
            version: VERSION,
            files: BTreeMap::new(),
        }
    }
}
