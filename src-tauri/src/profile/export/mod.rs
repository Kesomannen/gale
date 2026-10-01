use std::{
    collections::HashSet,
    fmt::Display,
    fs::File,
    io::{self, Cursor, Seek, Write},
    path::{Path, PathBuf},
    sync::LazyLock,
};

use base64::{Engine, prelude::BASE64_STANDARD};
use eyre::{Context, eyre};
use globset::{Glob, GlobBuilder, GlobSet, GlobSetBuilder};
use itertools::Itertools;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tracing::{info, trace};
use uuid::Uuid;
use walkdir::WalkDir;
use zip::{ZipWriter, write::SimpleFileOptions};

use super::{Profile, Result, install::ModInstall};
use crate::thunderstore::Backend;
use crate::{
    state::ManagerExt,
    thunderstore::{LegacyProfileCreateResponse, PackageIdent, Thunderstore, VersionIdent},
};

mod changelog;
pub mod commands;
pub mod modpack;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProfileManifest {
    #[serde(rename = "profileName")]
    pub name: String,
    pub mods: Vec<R2Mod>,
    #[serde(default, rename = "community")]
    pub game: Option<String>,
    #[serde(default, rename = "ignoredUpdates")]
    pub ignored_version_updates: Vec<Uuid>,
    #[serde(default)]
    pub ignored_package_updates: Vec<Uuid>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct R2Mod {
    #[serde(rename = "name")]
    pub ident: PackageIdent,
    #[serde(alias = "versionNumber")]
    pub version: R2Version,
    pub enabled: bool,
    #[serde(default)]
    pub source: Backend,
}

impl R2Mod {
    pub fn version_ident(&self) -> VersionIdent {
        self.ident.with_version(&self.version)
    }

    pub fn to_install(&self, thunderstore: &Thunderstore) -> Result<ModInstall> {
        // Prefer backend, otherwise fallback to generic lookup
        let borrowed_mod = thunderstore
            .backend(self.source)
            .find_ident(&self.version_ident())
            .or_else(|_| thunderstore.find_ident(&self.version_ident()))?;

        Ok(ModInstall::new(borrowed_mod).with_state(self.enabled))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

pub async fn export_zip<W>(app: &AppHandle, profile_id: i64, mut writer: W) -> Result<W>
where
    W: Write + Seek + Send + 'static,
{
    let (manifest, config_paths, profile_path) = {
        let manager = app.lock_manager();
        let (_, profile) = manager.profile_by_id(profile_id)?;

        let (manifest, config_paths) = prepare_export(profile)?;

        (manifest, config_paths, profile.path.clone())
    };

    tokio::task::spawn_blocking(move || {
        write_zip(&mut writer, &profile_path, &config_paths, &manifest)?;
        Ok::<_, eyre::Report>(writer)
    })
    .await?
}

fn prepare_export(profile: &Profile) -> Result<(ProfileManifest, Vec<PathBuf>)> {
    let mods = profile
        .thunderstore_mods()
        .map(|(ts_mod, enabled)| {
            let ident = ts_mod.ident.without_version();
            let version = ts_mod
                .ident
                .version()
                .parse::<semver::Version>()
                .expect("thunderstore version was not a semver")
                .into();

            R2Mod {
                ident,
                version,
                enabled,
                source: ts_mod.id.backend,
            }
        })
        .collect();

    let manifest = ProfileManifest {
        name: profile.name.clone(),
        game: Some(profile.game.slug.to_string()),
        mods,
        ignored_version_updates: profile.ignored_version_updates.iter().copied().collect(),
        ignored_package_updates: profile.ignored_package_updates.iter().copied().collect(),
    };

    let config_paths = list_export_files(profile)
        .filter_ok(|file| {
            if file.included {
                true
            } else {
                trace!(path = %file.path.display(), "excluding file from export");
                false
            }
        })
        .map_ok(|file| profile.path.join(file.path))
        .collect::<Result<Vec<_>>>()?;

    Ok((manifest, config_paths))
}

fn write_zip<W>(
    writer: &mut W,
    profile_path: &Path,
    paths: &[PathBuf],
    manifest: &ProfileManifest,
) -> Result<()>
where
    W: Write + Seek,
{
    let mut zip = ZipWriter::new(writer);

    let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    zip.start_file("export.r2x", opts)?;
    serde_yaml::to_writer(&mut zip, &manifest).context("failed to write profile manifest")?;

    write_config(paths.iter(), profile_path, &mut zip, opts)?;

    zip.finish()?;

    Ok(())
}

#[derive(Debug, Serialize)]
pub struct ExportedCode {
    pub code: Uuid,
    pub backend: Backend,
}

#[derive(Debug, thiserror::Error)]
enum ExportCodeError {
    #[error("profile export is too large ({size} bytes)")]
    TooLarge { size: usize },
    #[error(transparent)]
    Other(#[from] eyre::Report),
}

impl From<reqwest_middleware::Error> for ExportCodeError {
    fn from(value: reqwest_middleware::Error) -> Self {
        Self::Other(value.into())
    }
}

impl From<reqwest::Error> for ExportCodeError {
    fn from(value: reqwest::Error) -> Self {
        Self::Other(value.into())
    }
}

async fn export_code(app: &AppHandle) -> Result<ExportedCode, ExportCodeError> {
    let (profile_id, backend) = {
        let manager = app.lock_manager();
        let profile = manager.active_profile();

        let backend = if profile.has_hexium_exclusive_mods(&app.lock_thunderstore()) {
            Backend::Hexium
        } else {
            Backend::Thunderstore
        };

        (profile.id, backend)
    };

    let writer = Cursor::new(Vec::new());
    let data = export_zip(app, profile_id, writer).await?;

    let mut base64 = String::from(PROFILE_DATA_PREFIX);
    base64.push_str(&BASE64_STANDARD.encode(data.get_ref()));

    let len = base64.len();

    info!(len, "exporting profile code");

    let response = app
        .http()
        .post(backend.profile_export())
        .header("Content-Type", "application/octet-stream")
        .body(base64)
        .send()
        .await?;

    let response = match response.status() {
        status if status.is_success() => response.json::<LegacyProfileCreateResponse>().await?,
        StatusCode::PAYLOAD_TOO_LARGE => {
            return Err(ExportCodeError::TooLarge { size: len });
        }
        _ => {
            return Err(ExportCodeError::Other(eyre!(
                "failed to export profile code: {}",
                response.status()
            )));
        }
    };

    Ok(ExportedCode {
        code: response.key,
        backend,
    })
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportFile {
    pub path: PathBuf,
    pub size: usize,
    pub included: bool,
}

fn refresh_excluded_export_files(profile: &mut Profile) -> Result<()> {
    let config_files = find_config(&profile.path, profile.game.mod_loader.mod_config_dirs())
        .collect::<HashSet<_>>();

    profile
        .excluded_export_files
        .retain(|path| config_files.contains(path));

    Ok(())
}

fn list_export_files(profile: &Profile) -> impl Iterator<Item = Result<ExportFile>> {
    find_config(&profile.path, profile.game.mod_loader.mod_config_dirs()).map(|path| {
        let full_path = profile.path.join(&path);
        let size = std::fs::metadata(full_path).map(|meta| meta.len() as usize)?;

        let included = profile.excluded_export_files.get(&path).is_none();

        Ok(ExportFile {
            path,
            size,
            included,
        })
    })
}

fn write_config<P, I, W>(
    files: I,
    source: &Path,
    zip: &mut ZipWriter<W>,
    opts: SimpleFileOptions,
) -> Result<()>
where
    P: AsRef<Path>,
    I: Iterator<Item = P>,
    W: Write + Seek,
{
    for file in files {
        let file = file.as_ref();

        let path = file.to_string_lossy().replace('\\', "/");
        zip.start_file(path, opts)?;

        trace!(path = %file.display(), "writing config file to zip");

        let mut reader = File::open(source.join(file))?;

        io::copy(&mut reader, zip)?;
    }

    Ok(())
}

pub(super) fn find_config<'a>(
    root: &'a Path,
    config_dirs: &'a [&str],
) -> impl Iterator<Item = PathBuf> + 'a {
    static INCLUDE_SET: LazyLock<GlobSet> = LazyLock::new(|| {
        GlobSetBuilder::new()
            .add(Glob::new("*.{cfg,txt,json,yml,yaml,ini}").unwrap())
            .build()
            .unwrap()
    });

    static EXCLUDE_SET: LazyLock<GlobSet> = LazyLock::new(|| {
        GlobSetBuilder::new()
            .add(Glob::new("{dotnet,_state,snapshots,MelonLoader}/*").unwrap())
            .add(Glob::new("GDWeave/{GDWeave.log,core/*,mods/*}").unwrap())
            .add(Glob::new("mods.yml").unwrap())
            .add(
                GlobBuilder::new("BepInEx/plugins/*/manifest.json")
                    .literal_separator(true)
                    .build()
                    .unwrap(),
            )
            .build()
            .unwrap()
    });

    list_files(root).filter(move |path| {
        (config_dirs.iter().any(|dir| path.starts_with(dir)) || INCLUDE_SET.is_match(path))
            && !EXCLUDE_SET.is_match(path)
    })
}

pub(super) fn list_files(root: &Path) -> impl Iterator<Item = PathBuf> + '_ {
    WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(move |entry| {
            entry
                .into_path()
                .strip_prefix(root)
                .expect("path should be child of root")
                .to_path_buf()
        })
}
