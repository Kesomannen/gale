use anyhow::Context;
use base64::{prelude::BASE64_STANDARD, Engine};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Cursor, Seek, Write},
    path::{Path, PathBuf},
};
use uuid::Uuid;

use super::{downloader::ModInstall, ModManager, Profile, Result};

use crate::{
    thunderstore::{models::LegacyProfileCreateResponse, ModRef, Thunderstore},
    util::cmd::StateMutex,
};
use walkdir::WalkDir;
use zip::ZipWriter;

pub mod commands;
pub mod modpack;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LegacyProfileManifest<'a> {
    pub profile_name: &'a str,
    pub mods: Vec<R2Mod<'a>>,
    #[serde(default)]
    pub source: ImportSource,
    #[serde(default)]
    pub ignored_updates: Vec<Uuid>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum ImportSource {
    Gale,
    #[default]
    R2,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct R2Mod<'a> {
    pub name: &'a str,
    #[serde(alias = "versionNumber")]
    pub version: ExportVersion,
    pub enabled: bool,
}

impl<'a> R2Mod<'a> {
    pub fn into_install(self, thunderstore: &Thunderstore) -> Result<ModInstall> {
        let package = thunderstore.find_package(self.name)?;
        let semver = semver::Version::from(self.version);
        let version = package.get_version_with_num(&semver).with_context(|| {
            format!(
                "failed to find version {} for package {}",
                semver, self.name
            )
        })?;

        let mod_ref = ModRef {
            package_uuid: package.uuid4,
            version_uuid: version.uuid4,
        };

        Ok(ModInstall::new(mod_ref).with_state(self.enabled))
    }

    pub fn full_name(&self) -> String {
        format!(
            "{}-{}.{}.{}",
            self.name, self.version.major, self.version.minor, self.version.patch
        )
    }

    fn from_mod_ref(
        mod_ref: &ModRef,
        enabled: bool,
        thunderstore: &'a Thunderstore,
    ) -> Result<Self> {
        let borrowed = mod_ref.borrow(thunderstore)?;
        Ok(Self {
            name: &borrowed.package.full_name,
            version: ExportVersion::from(&borrowed.version.version_number),
            enabled,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExportVersion {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

impl From<ExportVersion> for semver::Version {
    fn from(value: ExportVersion) -> Self {
        semver::Version::new(value.major, value.minor, value.patch)
    }
}

impl From<&semver::Version> for ExportVersion {
    fn from(value: &semver::Version) -> Self {
        Self {
            major: value.major,
            minor: value.minor,
            patch: value.patch,
        }
    }
}

pub const PROFILE_DATA_PREFIX: &str = "#r2modman\n";

fn export_zip(
    profile: &Profile,
    writer: impl Write + Seek,
    thunderstore: &Thunderstore,
) -> Result<()> {
    let mut zip = ZipWriter::new(writer);

    let mods = profile
        .remote_mods()
        .map(|(mod_ref, _, enabled)| R2Mod::from_mod_ref(mod_ref, enabled, thunderstore))
        .collect::<Result<Vec<_>>>()
        .context("failed to resolve profile mods")?;

    let manifest = LegacyProfileManifest {
        profile_name: &profile.name,
        ignored_updates: profile.ignored_updates.iter().cloned().collect(),
        source: ImportSource::Gale,
        mods,
    };

    zip.start_file("export.r2x", Default::default())?;
    serde_yaml::to_writer(&mut zip, &manifest).context("failed to write profile manifest")?;

    write_includes(find_includes(&profile.path), &profile.path, &mut zip)?;

    Ok(())
}

async fn export_code(
    client: &reqwest::Client,
    manager: StateMutex<'_, ModManager>,
    thunderstore: StateMutex<'_, Thunderstore>,
) -> Result<Uuid> {
    let base64 = {
        let mut manager = manager.lock().unwrap();
        let thunderstore = thunderstore.lock().unwrap();

        let profile = manager.active_profile_mut();
        profile.refresh_config();

        let mut data = Cursor::new(Vec::new());
        export_zip(profile, &mut data, &thunderstore)?;

        let mut base64 = String::from(PROFILE_DATA_PREFIX);
        base64.push_str(&BASE64_STANDARD.encode(data.get_ref()));

        base64
    };

    const URL: &str = "https://thunderstore.io/api/experimental/legacyprofile/create/";

    let response = client
        .post(URL)
        .header("Content-Type", "application/octet-stream")
        .body(base64)
        .send()
        .await?
        .error_for_status()?
        .json::<LegacyProfileCreateResponse>()
        .await?;

    Ok(response.key)
}

fn write_includes<P, I, W>(files: I, source: &Path, zip: &mut ZipWriter<W>) -> Result<()>
where
    P: AsRef<Path>,
    I: Iterator<Item = P>,
    W: Write + Seek,
{
    for file in files {
        zip.start_file(file.as_ref().to_string_lossy(), Default::default())?;
        let mut reader = fs::File::open(source.join(file))?;

        io::copy(&mut reader, zip)?;
    }

    Ok(())
}

pub fn find_includes(root: &Path) -> impl Iterator<Item = PathBuf> + '_ {
    // Include any files in the BepInEx/config directory,
    // and any other files with the following extensions:
    const INCLUDE_EXTENSIONS: [&str; 6] = ["cfg", "txt", "json", "yml", "yaml", "ini"];
    const EXCLUDE_FILES: [&str; 5] = [
        "profile.json",
        "manifest.json",
        "mods.yml",
        "doorstop_config.ini",
        "snapshots",
    ];

    let config_dir = ["BepInEx", "config"].iter().collect::<PathBuf>();

    WalkDir::new(root)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .map(move |entry| entry.into_path().strip_prefix(root).unwrap().to_path_buf())
        .filter(|path| {
            !EXCLUDE_FILES
                .iter()
                .any(|exc| path.starts_with(exc) || path.ends_with(exc))
        })
        .filter(move |path| {
            path.starts_with(&config_dir)
                || path
                    .extension()
                    .is_some_and(|ext| INCLUDE_EXTENSIONS.iter().any(|inc| *inc == ext))
        })
}
