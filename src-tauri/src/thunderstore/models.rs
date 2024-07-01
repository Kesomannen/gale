use std::{collections::{HashMap, HashSet}, hash::Hash};

use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Clone, Eq)]
pub struct PackageListing {
    pub categories: HashSet<String>,
    pub date_created: DateTime<Utc>,
    pub date_updated: DateTime<Utc>,
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
    pub fn latest(&self) -> &PackageVersion {
        &self.versions[0]
    }

    pub fn is_modpack(&self) -> bool {
        self.categories.contains("Modpacks")
    }

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
    pub date_created: DateTime<Utc>,
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

#[derive(Serialize, Debug)]
pub struct UserMediaInitiateUploadParams {
    pub filename: String,
    pub file_size_bytes: u64,
}

#[derive(Deserialize)]
pub struct UserMediaInitiateUploadResponse {
    pub user_media: UserMedia,
    pub upload_urls: Vec<UploadPartUrl>,
}

#[derive(Deserialize)]
pub struct UserMedia {
    pub uuid: Option<Uuid>,
    pub filename: String,
    pub size: u64,
    pub datetime_created: DateTime<Utc>,
    pub expiry: DateTime<Utc>,
    pub status: UserMediaStatus,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all="snake_case")]
pub enum UserMediaStatus {
    Initial,
    UploadInitiated,
    UploadCreated,
    UploadError,
    UploadComplete,
    UploadAborted,
}

#[derive(Deserialize, Debug)]
pub struct UploadPartUrl {
    pub part_number: u32,
    pub url: String,
    pub offset: u64,
    pub length: u64,
}

#[derive(Serialize, Debug)]
pub struct UserMediaFinishUploadParams {
    pub parts: Vec<CompletedPart>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CompletedPart {
    #[serde(rename = "ETag")]
    pub tag: String,
    #[serde(rename = "PartNumber")]
    pub part_number: u32,
}

#[derive(Serialize, Debug)]
pub struct PackageSubmissionMetadata {
    pub author_name: String,
    pub categories: Vec<String>,
    pub communities: Vec<String>,
    pub has_nsfw_content: bool,
    pub upload_uuid: String,
    pub community_categories: HashMap<String, Vec<String>>,
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
    pub categories: Option<HashSet<String>>,
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
    pub contains_nsfw: bool,
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
    pub config_file: Option<String>,
    #[serde(flatten)]
    pub data: FrontendMod,
}