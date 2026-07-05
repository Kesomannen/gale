use std::{
    fmt::Display,
    fs::File,
    io::{self, Cursor, Seek, Write},
    path::{Path, PathBuf},
    sync::LazyLock,
};

use base64::{Engine, prelude::BASE64_STANDARD};
use eyre::Context;
use globset::{Glob, GlobBuilder, GlobSet, GlobSetBuilder};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tracing::info;
use uuid::Uuid;
use walkdir::WalkDir;
use zip::{ZipWriter, write::SimpleFileOptions};

use super::{Profile, Result, install::ModInstall};
use crate::{
    game::Game,
    state::ManagerExt,
    thunderstore::{LegacyProfileCreateResponse, PackageIdent, Thunderstore, VersionIdent},
};
use crate::thunderstore::Backend;

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
    // This will make the field available for future comparison against the loaded mods.
    // Defaults to Thunderstore for now.
    #[allow(dead_code)]
    #[serde(skip)]
    pub backend: Backend,
}

impl R2Mod {
    pub fn version_ident(&self) -> VersionIdent {
        self.ident.with_version(&self.version)
    }

    pub fn into_install(&self, thunderstore: &Thunderstore) -> Result<ModInstall> {
        let borrowed_mod = thunderstore.find_ident(&self.version_ident())?;

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

pub(super) fn export_zip(profile: &Profile, writer: impl Write + Seek, game: Game) -> Result<()> {
    let mut zip = ZipWriter::new(writer);

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
                backend: ts_mod.id.backend,
            }
        })
        .collect();

    let manifest = ProfileManifest {
        name: profile.name.clone(),
        game: Some(game.slug.to_string()),
        mods,
        ignored_version_updates: profile.ignored_version_updates.iter().cloned().collect(),
        ignored_package_updates: profile.ignored_package_updates.iter().cloned().collect(),
    };

    zip.start_file("export.r2x", SimpleFileOptions::default())?;
    serde_yaml::to_writer(&mut zip, &manifest).context("failed to write profile manifest")?;

    write_config(
        find_config(&profile.path, game.mod_loader.mod_config_dirs()),
        &profile.path,
        &mut zip,
    )?;

    Ok(())
}

async fn export_code(app: &AppHandle) -> Result<Uuid> {
    let (backend, base64) = {
        let mut manager = app.lock_manager();

        let game = manager.active_game().game;
        let profile = manager.active_profile_mut();

        let mut data = Cursor::new(Vec::new());
        export_zip(profile, &mut data, game)?;

        let mut base64 = String::from(PROFILE_DATA_PREFIX);
        base64.push_str(&BASE64_STANDARD.encode(data.get_ref()));

        (profile.thunderstore_backend(), base64)
    };

    info!(len = base64.len(), "exporting profile code");

    let response = app
        .http()
        .post(backend.profile_export())
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

pub fn find_config<'a>(
    root: &'a Path,
    config_dirs: &'a [&str],
) -> impl Iterator<Item = PathBuf> + 'a {
    static INCLUDE_SET: LazyLock<GlobSet> = LazyLock::new(|| {
        GlobSetBuilder::new()
            .add(Glob::new("BepInEx/config/*").unwrap())
            .add(Glob::new("*.{cfg,txt,json,yml,yaml,ini}").unwrap())
            .build()
            .unwrap()
    });

    static EXCLUDE_SET: LazyLock<GlobSet> = LazyLock::new(|| {
        GlobSetBuilder::new()
            .add(Glob::new("{dotnet,_state,MelonLoader}/*").unwrap())
            .add(Glob::new("dotnet/*").unwrap())
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
        .filter(move |path| {
            (config_dirs.iter().any(|dir| path.starts_with(dir)) || INCLUDE_SET.is_match(path))
                && !EXCLUDE_SET.is_match(path)
        })
}
