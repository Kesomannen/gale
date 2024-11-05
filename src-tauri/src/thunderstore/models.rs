use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    path::PathBuf,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{PackageIdent, VersionIdent};
use crate::{game::Game, profile::Profile};

#[derive(Serialize, Deserialize, Debug, Clone, Eq)]
pub struct PackageListing {
    #[serde(rename = "full_name")]
    pub ident: PackageIdent,
    pub categories: HashSet<String>,
    pub date_created: DateTime<Utc>,
    pub date_updated: DateTime<Utc>,
    pub donation_link: Option<String>,
    pub has_nsfw_content: bool,
    pub is_deprecated: bool,
    pub is_pinned: bool,
    pub package_url: String,
    pub rating_score: u32,
    #[serde(rename = "uuid4")]
    pub uuid: Uuid,
    pub versions: Vec<PackageVersion>,
}

impl PackageListing {
    pub fn owner(&self) -> &str {
        self.ident.owner()
    }

    pub fn name(&self) -> &str {
        self.ident.name()
    }

    pub fn full_name(&self) -> &str {
        self.ident.as_str()
    }

    pub fn latest(&self) -> &PackageVersion {
        &self.versions[0]
    }

    pub fn is_modpack(&self) -> bool {
        self.categories.contains("Modpacks")
    }

    pub fn get_version(&self, uuid: Uuid) -> Option<&PackageVersion> {
        self.versions.iter().find(|v| v.uuid == uuid)
    }

    pub fn get_version_with_num(&self, version: &str) -> Option<&PackageVersion> {
        self.versions.iter().find(|v| v.version() == version)
    }

    pub fn total_downloads(&self) -> u32 {
        self.versions.iter().map(|v| v.downloads).sum()
    }

    pub fn owner_url(&self, game: Game) -> String {
        format!(
            "https://thunderstore.io/c/{}/p/{}/",
            game.slug,
            self.owner()
        )
    }

    pub fn url(&self, game: Game) -> String {
        format!(
            "https://thunderstore.io/c/{}/p/{}/{}/",
            game.slug,
            self.owner(),
            self.name()
        )
    }
}

impl Hash for PackageListing {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
    }
}

impl PartialEq for PackageListing {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq)]
pub struct PackageVersion {
    #[serde(rename = "full_name")]
    pub ident: VersionIdent,
    pub date_created: DateTime<Utc>,
    pub dependencies: Vec<VersionIdent>,
    pub description: String,
    pub downloads: u32,
    pub file_size: u64,
    pub is_active: bool,
    #[serde(rename = "uuid4")]
    pub uuid: Uuid,
    pub website_url: String,
}

impl PackageVersion {
    pub fn owner(&self) -> &str {
        self.ident.owner()
    }

    pub fn name(&self) -> &str {
        self.ident.name()
    }

    pub fn version(&self) -> &str {
        self.ident.version()
    }

    pub fn full_name(&self) -> &str {
        self.ident.full_name()
    }

    pub fn parsed_version(&self) -> semver::Version {
        self.ident
            .version()
            .parse()
            .expect("thunderstore package has invalid version")
    }

    pub fn download_url(&self) -> String {
        format!(
            "https://thunderstore.io/package/download/{}/",
            self.ident.path()
        )
    }
}

impl PartialEq for PackageVersion {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl Hash for PackageVersion {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
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
    pub dependencies: Vec<VersionIdent>,
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
    //pub filename: String,
    //pub size: u64,
    //pub datetime_created: DateTime<Utc>,
    //pub expiry: DateTime<Utc>,
    //pub status: UserMediaStatus,
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

#[derive(Serialize, Debug)]
pub struct PackageSubmissionMetadata {
    pub author_name: String,
    pub categories: Vec<String>,
    pub communities: Vec<String>,
    pub has_nsfw_content: bool,
    pub upload_uuid: Uuid,
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
    pub categories: Option<Vec<String>>,
    pub version: Option<semver::Version>,
    pub author: Option<String>,
    pub rating: Option<u32>,
    pub downloads: Option<u32>,
    pub file_size: u64,
    pub website_url: Option<String>,
    pub donate_url: Option<String>,
    pub dependencies: Option<Vec<VersionIdent>>,
    pub is_pinned: bool,
    pub is_deprecated: bool,
    pub contains_nsfw: bool,
    pub uuid: Uuid,
    pub is_installed: bool,
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
    pub config_file: Option<PathBuf>,
    #[serde(flatten)]
    pub data: FrontendMod,
}

pub trait IntoFrontendMod {
    fn into_frontend(self, profile: &Profile) -> FrontendMod;
}
