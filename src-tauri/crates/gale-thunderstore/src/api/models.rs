use super::VersionId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, hash::Hash};
use url::Url;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Eq)]
pub struct PackageV1 {
    pub categories: HashSet<String>,
    pub date_created: DateTime<Utc>,
    pub date_updated: DateTime<Utc>,
    pub donation_link: Option<Url>,
    pub full_name: String,
    pub has_nsfw_content: bool,
    pub is_deprecated: bool,
    pub is_pinned: bool,
    pub name: String,
    pub owner: String,
    pub package_url: Url,
    pub rating_score: u32,
    pub uuid4: Uuid,
    pub versions: Vec<PackageVersionV1>,
}

impl PackageV1 {
    pub fn latest(&self) -> &PackageVersionV1 {
        &self.versions[0]
    }

    pub fn is_modpack(&self) -> bool {
        self.categories.contains("Modpacks")
    }

    pub fn get_version(&self, uuid: &Uuid) -> Option<&PackageVersionV1> {
        self.versions.iter().find(|v| v.uuid4 == *uuid)
    }

    pub fn get_version_with_num(&self, version: &semver::Version) -> Option<&PackageVersionV1> {
        self.versions.iter().find(|v| v.version_number == *version)
    }

    pub fn total_downloads(&self) -> u32 {
        self.versions.iter().map(|v| v.downloads).sum()
    }
}

impl Hash for PackageV1 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uuid4.hash(state);
    }
}

impl PartialEq for PackageV1 {
    fn eq(&self, other: &Self) -> bool {
        self.uuid4 == other.uuid4
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq)]
pub struct PackageVersionV1 {
    pub date_created: DateTime<Utc>,
    pub dependencies: Vec<VersionId>,
    pub description: String,
    pub download_url: Url,
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

impl PartialEq for PackageVersionV1 {
    fn eq(&self, other: &Self) -> bool {
        self.uuid4 == other.uuid4
    }
}

impl Hash for PackageVersionV1 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uuid4.hash(state);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LegacyProfileCreateResponse {
    pub key: Uuid,
}

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
#[serde(rename_all = "snake_case")]
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

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageSubmissionResult {
    pub package_version: PackageVersion,
    pub available_communities: Vec<AvailableCommunity>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AvailableCommunity {
    pub community: Community,
    pub categories: PackageCategory,
    pub url: Url,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Community {
    pub identifier: String,
    pub name: String,
    pub discord_url: Option<Url>,
    pub wiki_url: Option<Url>,
    pub require_package_listing_approval: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageCategory {
    pub name: String,
    pub slug: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageVersionMetrics {
    pub downloads: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageMetrics {
    pub downloads: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RenderMarkdownParams {
    pub markdown: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RenderMarkdownResponse {
    pub html: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MarkdownResponse {
    pub markdown: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PackageIndexEntry {
    pub namespace: String,
    pub name: String,
    pub version_number: semver::Version,
    pub file_format: String,
    pub file_size: u64,
    pub dependencies: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq)]
pub struct PackageVersion {
    pub namespace: String,
    pub name: String,
    pub version_number: semver::Version,
    pub full_name: String,
    pub description: String,
    pub icon: Url,
    pub dependencies: Vec<String>,
    pub download_url: Url,
    pub downloads: u32,
    pub date_created: DateTime<Utc>,
    pub website_url: String,
    pub is_active: bool,
}

impl PartialEq for PackageVersion {
    fn eq(&self, other: &Self) -> bool {
        self.full_name == other.full_name
    }
}

impl Hash for PackageVersion {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.full_name.hash(state);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq)]
pub struct Package {
    pub namespace: String,
    pub name: String,
    pub full_name: String,
    pub owner: String,
    pub package_url: Url,
    pub date_created: DateTime<Utc>,
    pub date_updated: DateTime<Utc>,
    pub rating_score: i32,
    pub is_pinned: bool,
    pub is_deprecated: bool,
    pub total_downloads: i32,
    pub latest: PackageVersion,
    pub community_listings: Vec<PackageListingExperimental>,
}

impl PartialEq for Package {
    fn eq(&self, other: &Self) -> bool {
        self.full_name == other.full_name
    }
}

impl Hash for Package {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.full_name.hash(state);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PackageListingExperimental {
    pub has_nsfw_content: bool,
    pub categories: HashSet<String>,
    pub community: String,
    pub review_status: ReviewStatus,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReviewStatus {
    Unreviewed,
    Approved,
    Rejected,
}
