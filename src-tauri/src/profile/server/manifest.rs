use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub const FILE_NAME: &str = ".gale-server-manifest.json";
pub const VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ManifestEntry {
    pub hash: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct DeploymentManifest {
    pub version: u32,
    pub old_files_cleaned: bool,
    pub files: BTreeMap<String, ManifestEntry>,
}

impl Default for DeploymentManifest {
    fn default() -> Self {
        Self {
            version: VERSION,
            old_files_cleaned: false,
            files: BTreeMap::new(),
        }
    }
}

pub fn is_safe_relative_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.contains('\\')
        && !path.contains('\0')
        && path
            .split('/')
            .all(|component| !component.is_empty() && component != "." && component != "..")
}

#[cfg(test)]
mod tests {
    use super::is_safe_relative_path;

    #[test]
    fn accepts_profile_paths() {
        assert!(is_safe_relative_path("BepInEx/plugins/Example.dll"));
        assert!(is_safe_relative_path("doorstop_config.ini"));
    }

    #[test]
    fn rejects_paths_that_can_escape_the_server_directory() {
        for path in [
            "",
            "/etc/passwd",
            "../outside",
            "inside/../outside",
            "inside//file",
            "inside\\..\\outside",
            "./file",
        ] {
            assert!(!is_safe_relative_path(path), "accepted {path:?}");
        }
    }
}
