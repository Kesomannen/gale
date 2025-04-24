use std::{
    fmt::Display,
    fs::File,
    io::{self, Cursor, Seek, Write},
    path::{Path, PathBuf},
};

use base64::{prelude::BASE64_STANDARD, Engine};
use eyre::{anyhow, Context};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use uuid::Uuid;
use walkdir::WalkDir;
use zip::{write::SimpleFileOptions, ZipWriter};

use super::{install::ModInstall, Profile, Result};
use crate::{
    game::Game,
    state::ManagerExt,
    thunderstore::{LegacyProfileCreateResponse, ModId, Thunderstore},
};

mod changelog;
pub mod commands;
pub mod modpack;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LegacyProfileManifest {
    pub profile_name: String,
    pub mods: Vec<R2Mod>,
    #[serde(default)]
    pub community: Option<String>,
    #[serde(default)]
    pub ignored_updates: Vec<Uuid>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct R2Mod {
    #[serde(rename = "name")]
    pub full_name: String,
    #[serde(alias = "versionNumber")]
    pub version: R2Version,
    pub enabled: bool,
}

impl R2Mod {
    pub fn into_install(self, thunderstore: &Thunderstore) -> Result<ModInstall> {
        let package = thunderstore.find_package(&self.full_name)?;

        let version = self.version.to_string();
        let version = package.get_version_with_num(&version).ok_or_else(|| {
            anyhow!(
                "failed to find version {} for package {}",
                version,
                self.full_name
            )
        })?;

        let id = ModId {
            package_uuid: package.uuid,
            version_uuid: version.uuid,
        };

        Ok(ModInstall::new(id).with_state(self.enabled))
    }

    pub fn ident(&self) -> String {
        format!(
            "{}-{}.{}.{}",
            self.full_name, self.version.major, self.version.minor, self.version.patch
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct R2Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

impl Display for R2Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl From<semver::Version> for R2Version {
    fn from(value: semver::Version) -> Self {
        Self {
            major: value.major,
            minor: value.minor,
            patch: value.patch,
        }
    }
}

pub const PROFILE_DATA_PREFIX: &str = "#r2modman\n";

pub(super) fn export_zip(profile: &Profile, writer: impl Write + Seek, game: Game) -> Result<()> {
    let mut zip = ZipWriter::new(writer);

    let mods = profile
        .thunderstore_mods()
        .map(|(ts_mod, enabled)| {
            let full_name = ts_mod.ident.full_name().to_string();
            let version = ts_mod
                .ident
                .version()
                .parse::<semver::Version>()
                .unwrap()
                .into();

            R2Mod {
                full_name,
                version,
                enabled,
            }
        })
        .collect();

    let manifest = LegacyProfileManifest {
        profile_name: profile.name.clone(),
        ignored_updates: profile.ignored_updates.iter().cloned().collect(),
        community: Some(game.slug.to_string()),
        mods,
    };

    zip.start_file("export.r2x", SimpleFileOptions::default())?;
    serde_yaml::to_writer(&mut zip, &manifest).context("failed to write profile manifest")?;

    write_config(find_default_config(&profile.path), &profile.path, &mut zip)?;

    Ok(())
}

async fn export_code(app: &AppHandle) -> Result<Uuid> {
    let base64 = {
        let mut manager = app.lock_manager();

        let game = manager.active_game().game;
        let profile = manager.active_profile_mut();
        profile.refresh_config();

        let mut data = Cursor::new(Vec::new());
        export_zip(profile, &mut data, game)?;

        let mut base64 = String::from(PROFILE_DATA_PREFIX);
        base64.push_str(&BASE64_STANDARD.encode(data.get_ref()));

        base64
    };

    const URL: &str = "https://thunderstore.io/api/experimental/legacyprofile/create/";

    let response = app
        .http()
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

fn write_config<P, I, W>(files: I, source: &Path, zip: &mut ZipWriter<W>) -> Result<()>
where
    P: AsRef<Path>,
    I: Iterator<Item = P>,
    W: Write + Seek,
{
    for file in files {
        let path = file.as_ref().to_string_lossy().replace('\\', "/");
        zip.start_file(path, SimpleFileOptions::default())?;

        let mut reader = File::open(source.join(file))?;

        io::copy(&mut reader, zip)?;
    }

    Ok(())
}

const COMMON_EXTENSIONS: &[&str] = &["cfg", "txt", "json", "yml", "yaml", "ini", "xml"];

const GENERATED_FILES: &[&str] = &[
    "profile.json",
    "manifest.json",
    "mods.yml",
    "doorstop_config.ini",
    "snapshots",
    "_state",
];

pub enum IncludeExtensions {
    /// All extensions.
    All,
    /// Only common config extensions (see [`COMMON_EXTENSIONS`]).
    Default,
}

pub enum IncludeGenerated {
    /// Include every file (as long as they fit [`IncludeExtensions`]).
    Yes,
    /// Skip common mod-manager generated files (see [`GENERATED_FILES`]).
    No,
}

pub fn find_config(
    root: &Path,
    include_extensions: IncludeExtensions,
    include_generated: IncludeGenerated,
) -> impl Iterator<Item = PathBuf> + '_ {
    WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(move |entry| entry.into_path().strip_prefix(root).unwrap().to_path_buf())
        .filter(move |path| {
            matches!(include_generated, IncludeGenerated::Yes)
                || !GENERATED_FILES
                    .iter()
                    .any(|exc| path.starts_with(exc) || path.ends_with(exc))
        })
        .filter(move |path| {
            matches!(include_extensions, IncludeExtensions::All)
                || path
                    .extension()
                    .is_some_and(|ext| COMMON_EXTENSIONS.iter().any(|inc| *inc == ext))
        })
}

/// Alias for [`find_config`] with [`IncludeExtensions`] and [`IncludeGenerated`] set
/// to their default values.
pub fn find_default_config(root: &Path) -> impl Iterator<Item = PathBuf> + '_ {
    find_config(root, IncludeExtensions::Default, IncludeGenerated::No)
}
