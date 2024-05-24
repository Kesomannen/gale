use std::hash::Hash;

use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use uuid::Uuid;

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Eq)]
pub struct PackageListing {
    pub categories: Vec<String>,
    pub date_created: String,
    pub date_updated: String,
    pub donation_link: Option<String>,
    pub full_name: String,
    pub has_nsfw_content: bool,
    pub is_deprecated: bool,
    pub is_pinned: bool,
    pub name: String,
    pub owner: String,
    pub package_url: String,
    pub rating_score: u32,
    pub uuid4: Uuid,
    pub versions: Vec<PackageVersion>,
}

impl PackageListing {
    pub fn get_version(&self, uuid: &Uuid) -> Option<&PackageVersion> {
        self.versions.iter().find(|v| v.uuid4 == *uuid)
    }

    pub fn get_version_with_num(&self, version: &semver::Version) -> Option<&PackageVersion> {
        self.versions.iter().find(|v| v.version_number == *version)
    }

    pub fn total_downloads(&self) -> u32 {
        self.versions.iter().map(|v| v.downloads).sum()
    }
}

impl Hash for PackageListing {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uuid4.hash(state);
    }
}

impl PartialEq for PackageListing {
    fn eq(&self, other: &Self) -> bool {
        self.uuid4 == other.uuid4
    }
}

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Eq)]
pub struct PackageVersion {
    pub date_created: String,
    pub dependencies: Vec<String>,
    pub description: String,
    pub download_url: String,
    pub downloads: u32,
    pub file_size: u64,
    pub full_name: String,
    pub icon: String,
    pub is_active: bool,
    pub name: String,
    pub uuid4: Uuid,
    pub version_number: semver::Version,
    pub website_url: String,
}

impl PartialEq for PackageVersion {
    fn eq(&self, other: &Self) -> bool {
        self.uuid4 == other.uuid4
    }
}

impl Hash for PackageVersion {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uuid4.hash(state);
    }
}

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct LegacyProfileCreateResponse {
    pub key: Uuid,
}

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PackageManifest {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    pub description: String,
    pub version_number: semver::Version,
    pub dependencies: Vec<String>,
    pub website_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub installers: Option<Vec<PackageInstaller>>,
}

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PackageInstaller {
    pub identifier: String,
}

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum FrontendModKind {
    Local,
    #[default]
    Remote,
}

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FrontendMod {
    pub name: String,
    pub description: Option<String>,
    pub categories: Option<Vec<String>>,
    pub version: Option<semver::Version>,
    pub author: Option<String>,
    pub rating: Option<u32>,
    pub downloads: Option<u32>,
    pub website_url: Option<String>,
    pub donate_url: Option<String>,
    pub icon: Option<String>,
    pub dependencies: Option<Vec<String>>,
    pub is_pinned: bool,
    pub is_deprecated: bool,
    pub uuid: Uuid,
    pub last_updated: Option<String>,
    pub versions: Vec<FrontendVersion>,
    #[serde(rename = "type")]
    pub kind: FrontendModKind,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontendVersion {
    pub name: semver::Version,
    pub uuid: Uuid,
}

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FrontendProfileMod {
    pub enabled: bool,
    #[serde(flatten)]
    pub data: FrontendMod,
}