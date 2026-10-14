use std::{
    collections::BTreeMap,
    path::{Component, Path, PathBuf},
};

use eyre::{Result as EyreResult, ensure};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de, ser};

use crate::util;

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
    #[serde(
        serialize_with = "serialize_files",
        deserialize_with = "deserialize_files"
    )]
    pub files: BTreeMap<PathBuf, ManifestEntry>,
}

impl Default for DeploymentManifest {
    fn default() -> Self {
        Self {
            version: VERSION,
            files: BTreeMap::new(),
        }
    }
}

pub fn normalize_relative_path(path: &Path) -> EyreResult<PathBuf> {
    ensure!(
        !path.as_os_str().is_empty() && util::fs::is_enclosed(path),
        "unsafe relative path: {}",
        path.display()
    );
    ensure!(path.to_str().is_some(), "path is not valid Unicode");

    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(component) => normalized.push(component),
            Component::Prefix(_) | Component::RootDir => unreachable!(),
        }
    }

    ensure!(!normalized.as_os_str().is_empty(), "relative path is empty");

    Ok(normalized)
}

fn serialize_files<S>(
    files: &BTreeMap<PathBuf, ManifestEntry>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let files = files
        .iter()
        .map(|(path, entry)| {
            let path = path
                .to_str()
                .ok_or_else(|| ser::Error::custom("manifest path is not valid Unicode"))?
                .replace('\\', "/");

            Ok((path, entry))
        })
        .collect::<std::result::Result<BTreeMap<_, _>, S::Error>>()?;

    files.serialize(serializer)
}

fn deserialize_files<'de, D>(
    deserializer: D,
) -> std::result::Result<BTreeMap<PathBuf, ManifestEntry>, D::Error>
where
    D: Deserializer<'de>,
{
    BTreeMap::<String, ManifestEntry>::deserialize(deserializer)?
        .into_iter()
        .map(|(path, entry)| {
            let path = path.replace('\\', "/");
            normalize_relative_path(Path::new(&path))
                .map(|path| (path, entry))
                .map_err(de::Error::custom)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, path::PathBuf};

    use super::{DeploymentManifest, ManifestEntry};

    #[test]
    fn normalizes_manifest_paths() {
        let manifest: DeploymentManifest = serde_json::from_value(serde_json::json!({
            "version": 1,
            "files": {
                "./BepInEx\\plugins/../config\\Test.cfg": {
                    "hash": "hash",
                    "size": 1
                }
            }
        }))
        .unwrap();

        assert!(
            manifest
                .files
                .contains_key(&PathBuf::from("BepInEx/config/Test.cfg"))
        );
    }

    #[test]
    fn rejects_manifest_paths_outside_the_profile() {
        let result = serde_json::from_value::<DeploymentManifest>(serde_json::json!({
            "version": 1,
            "files": {
                "../outside.dll": {
                    "hash": "hash",
                    "size": 1
                }
            }
        }));

        assert!(result.is_err());
    }

    #[test]
    fn serializes_manifest_paths_with_forward_slashes() {
        let manifest = DeploymentManifest {
            files: BTreeMap::from([(
                PathBuf::from(r"BepInEx\plugins\Test.dll"),
                ManifestEntry {
                    hash: "hash".to_owned(),
                    size: 1,
                },
            )]),
            ..DeploymentManifest::default()
        };
        let value = serde_json::to_value(manifest).unwrap();

        assert!(value["files"].get("BepInEx/plugins/Test.dll").is_some());
    }
}
