use anyhow::Context;
use base64::{prelude::BASE64_STANDARD, Engine};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self},
    path::{Path, PathBuf},
};
use uuid::Uuid;

use super::{downloader::ModInstall, ModManager, Profile, Result};

use crate::{
    prefs::Prefs,
    thunderstore::{models::LegacyProfileCreateResponse, ModRef, Thunderstore},
    util::{self, cmd::StateMutex, error::IoResultExt},
};
use walkdir::WalkDir;

pub mod commands;
pub mod modpack;

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
    let mut zip = util::zip::builder(dir).fs_context("creating zip archive", dir)?;

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

    write_includes(find_includes(&profile.path), &profile.path, &mut zip)?;

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

fn write_includes<P, I>(files: I, source: &Path, zip: &mut util::zip::ZipBuilder) -> Result<()>
where
    P: AsRef<Path>,
    I: Iterator<Item = P>,
{
    for file in files {
        let writer = zip.writer(&file)?;
        let mut reader = fs::File::open(source.join(file))?;

        io::copy(&mut reader, writer)?;
    }

    Ok(())
}

pub fn find_includes(root: &Path) -> impl Iterator<Item = PathBuf> + '_ {
    const INCLUDE_EXTENSIONS: [&str; 6] = ["cfg", "txt", "json", "yml", "yaml", "ini"];
    const EXCLUDE_FILES: [&str; 4] = [
        "profile.json",
        "manifest.json",
        "mods.yml",
        "doorstop_config.ini",
    ];

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
        .map(move |path| path.strip_prefix(root).unwrap().to_path_buf())
}
