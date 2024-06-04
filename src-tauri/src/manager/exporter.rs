use anyhow::Context;
use base64::{prelude::BASE64_STANDARD, Engine};
use image::{imageops::FilterType, io::Reader as ImageReader, ImageFormat};
use serde::{Deserialize, Serialize};
use std::{
    fs, io::{self, Cursor}, path::{Path, PathBuf}
};
use typeshare::typeshare;
use uuid::Uuid;

use super::{ModManager, Profile, ProfileMod, ProfileModKind, Result};

use crate::{
    prefs::Prefs, thunderstore::{
        models::{LegacyProfileCreateResponse, PackageManifest},
        ModRef, Thunderstore,
    }, util::{self, cmd::StateMutex, error::IoResultExt}
};
use chrono::Utc;
use walkdir::WalkDir;

pub mod commands;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct R2Manifest<'a> {
    pub profile_name: &'a str,
    pub mods: Vec<R2Mod<'a>>,
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
    pub fn into_profile_mod(self, thunderstore: &Thunderstore) -> Result<ProfileMod> {
        let package = thunderstore.find_package(self.name)?;
        let semver = semver::Version::from(self.version);
        let version = package.get_version_with_num(&semver).with_context(|| {
            format!(
                "failed to find version {} for package {}",
                semver, self.name
            )
        })?;

        Ok(ProfileMod {
            enabled: self.enabled,
            install_time: Utc::now(),
            kind: ProfileModKind::Remote(ModRef {
                package_uuid: package.uuid4,
                version_uuid: version.uuid4,
            }),
        })
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

fn export_file(profile: &Profile, dir: &mut PathBuf, thunderstore: &Thunderstore) -> Result<()> {
    dir.push(&profile.name);
    dir.set_extension("r2z");
    let mut zip = util::io::zip(dir).fs_context("creating zip archive", dir)?;

    let mods = profile
        .remote_mods()
        .map(|(mod_ref, enabled)| R2Mod::from_mod_ref(mod_ref, enabled, thunderstore))
        .collect::<Result<Vec<_>>>()
        .context("failed to resolve profile mods")?;

    let manifest = R2Manifest {
        profile_name: &profile.name,
        mods,
    };

    let writer = zip.writer("export.r2x")?;
    serde_yaml::to_writer(writer, &manifest).context("failed to write profile manifest")?;

    write_includes(profile, &mut zip)?;

    Ok(())
}

async fn export_code(
    client: &reqwest::Client,
    manager: StateMutex<'_, ModManager>,
    thunderstore: StateMutex<'_, Thunderstore>,
    prefs: StateMutex<'_, Prefs>,
) -> Result<Uuid> {
    let base64 = {
        let mut manager = manager.lock().unwrap();
        let thunderstore = thunderstore.lock().unwrap();
        let prefs = prefs.lock().unwrap();

        let profile = manager.active_profile_mut();
        profile.refresh_config(Some(&thunderstore));

        let mut path = prefs.get_path_or_err("temp_dir")?.join("exports");
        fs::create_dir_all(&path)?;
        export_file(profile, &mut path, &thunderstore)?;

        let data = fs::read(path).unwrap();
        let mut base64 = String::from(PROFILE_DATA_PREFIX);
        base64.push_str(&BASE64_STANDARD.encode(data));

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

#[typeshare]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModpackArgs {
    pub name: String,
    pub description: String,
    pub version_number: semver::Version,
    pub icon: PathBuf,
    pub website_url: Option<String>,
}

fn export_pack(
    profile: &Profile,
    path: &Path,
    args: ModpackArgs,
    thunderstore: &Thunderstore,
) -> Result<()> {
    let dep_strings = profile
        .remote_mods()
        .filter(|(_, enabled)| *enabled) // filter out disabled mods
        .map(|(mod_ref, _)| {
            let borrowed_mod = mod_ref.borrow(thunderstore)?;
            Ok(borrowed_mod.version.full_name.clone())
        })
        .collect::<Result<Vec<_>>>()
        .context("failed to resolve modpack dependencies")?;

    let manifest = PackageManifest {
        name: args.name,
        description: args.description,
        version_number: args.version_number,
        website_url: args.website_url.unwrap_or_default(),
        dependencies: dep_strings,
        installers: None,
        author: None,
    };

    let mut zip = util::io::zip(path)?;

    zip.write_str("manifest.json", &serde_json::to_string_pretty(&manifest)?)?;

    let readme = format!("# {}\n\n{}", manifest.name, manifest.description);
    zip.write_str("README.md", &readme)?;

    let img = ImageReader::open(&args.icon)?.decode()?;
    let img = img.resize_exact(256, 256, FilterType::Lanczos3);

    let mut bytes = Vec::new();
    img.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)?;
    zip.write("icon.png", &bytes)?;

    write_includes(profile, &mut zip)?;

    Ok(())
}

fn write_includes(profile: &Profile, zip: &mut util::io::Zip) -> Result<()> {
    for (source, destination) in find_includes(&profile.path) {
        let writer = zip.writer(destination)?;
        let mut reader = fs::File::open(&source)?;
        io::copy(&mut reader, writer)?;
    }

    Ok(())
}

const INCLUDE_EXTENSIONS: [&str; 6] = ["cfg", "txt", "json", "yml", "yaml", "ini"];
const EXCLUDE_FILES: [&str; 4] = ["profile.json", "manifest.json", "mods.yml", "doorstop_config.ini"];

pub fn find_includes(root: &Path) -> impl Iterator<Item = (PathBuf, PathBuf)> + '_ {
    WalkDir::new(root)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.into_path())
        .filter(|path| {
            let name = path.file_name().unwrap();
            !EXCLUDE_FILES.iter().any(|exc| name == *exc)
        })
        .filter(|path| match path.extension() {
            Some(ext) => INCLUDE_EXTENSIONS.iter().any(|inc| *inc == ext),
            None => false,
        })
        .map(move |path| {
            let relative = path.strip_prefix(root).unwrap().to_path_buf();
            (path, relative)
        })
}
