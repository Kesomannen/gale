use std::{
    fs,
    io::{Cursor, Read, Seek},
    path::{Path, PathBuf},
    sync::Mutex,
};

use anyhow::{anyhow, ensure, Context, Result};
use image::{imageops::FilterType, io::Reader as ImageReader};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use typeshare::typeshare;

use crate::{
    command_util::StateMutex,
    fs_util,
    manager::{commands::save, LocalMod, ProfileMod},
    prefs::Prefs,
    thunderstore::{
        models::{LegacyProfileCreateResponse, PackageManifest},
        BorrowedMod, ModRef, Thunderstore,
    },
    util::IoResultExt,
    NetworkClient,
};

use super::{config, downloader, ModManager, Profile};
use base64::{prelude::BASE64_STANDARD, Engine};
use uuid::Uuid;
use itertools::Itertools;
use reqwest::StatusCode;

pub mod commands;

pub fn setup(_app: &AppHandle) -> Result<()> {
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ExportManifest {
    profile_name: String,
    mods: Vec<ExportMod>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ExportMod {
    name: String,
    version: ExportVersion,
    enabled: bool,
}

impl ExportMod {
    fn mod_ref(self, thunderstore: &Thunderstore) -> Result<ModRef> {
        let package = thunderstore.find_package(&self.name)?;
        let semver: semver::Version = self.version.into();
        let version = package.get_version_with_num(&semver).with_context(|| {
            format!(
                "failed to find version {} for package {}",
                semver, self.name
            )
        })?;

        Ok(ModRef {
            package_uuid: package.uuid4,
            version_uuid: version.uuid4,
        })
    }
}

impl From<BorrowedMod<'_>> for ExportMod {
    fn from(value: BorrowedMod<'_>) -> Self {
        let version = &value.version.version_number;

        Self {
            name: value.package.full_name.clone(),
            version: ExportVersion {
                major: version.major,
                minor: version.minor,
                patch: version.patch,
            },
            enabled: true,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ExportVersion {
    major: u64,
    minor: u64,
    patch: u64,
}

impl From<ExportVersion> for semver::Version {
    fn from(value: ExportVersion) -> Self {
        semver::Version::new(value.major, value.minor, value.patch)
    }
}

const PROFILE_DATA_PREFIX: &str = "#r2modman\n";

fn export_file(profile: &Profile, dir: &mut PathBuf, thunderstore: &Thunderstore) -> Result<()> {
    dir.push(&profile.name);
    dir.set_extension("r2z");
    let mut zip = fs_util::zip(dir).fs_context("creating zip archive", dir)?;

    let mods = profile
        .remote_mods()
        .map(|mod_ref| {
            let borrowed_mod = mod_ref.borrow(thunderstore)?;
            Ok(ExportMod::from(borrowed_mod))
        })
        .collect::<Result<Vec<_>>>()
        .context("failed to resolve profile mods")?;

    let manifest = ExportManifest {
        profile_name: profile.name.clone(),
        mods,
    };

    let yaml = serde_yaml::to_string(&manifest).context("failed to serialize profile manifest")?;

    zip.write_str("export.r2x", &yaml)
        .context("failed to write profile manifest")?;

    write_config(profile, &mut zip)?;

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
        profile.refresh_config();

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

async fn import_file<S: Read + Seek>(source: S, app: &AppHandle) -> Result<()> {
    let mod_refs = {
        let manager = app.state::<Mutex<ModManager>>();
        let thunderstore = app.state::<Mutex<Thunderstore>>();
        let prefs = app.state::<Mutex<Prefs>>();

        let mut manager = manager.lock().unwrap();
        let thunderstore = thunderstore.lock().unwrap();
        let prefs = prefs.lock().unwrap();

        let temp_path = prefs.get_path_or_err("temp_dir")?.join("imports");
        fs::create_dir_all(&temp_path)?;

        zip_extract::extract(source, &temp_path, true)?;

        let manifest = fs::read_to_string(temp_path.join("export.r2x"))
            .context("failed to read profile manifest")?;

        let manifest: ExportManifest =
            serde_yaml::from_str(&manifest).context("failed to parse profile manifest")?;

        let name = manifest.profile_name.to_owned();
        let profile = manager.active_game_mut().create_profile(name)?;

        let mut config_dir = profile.path.clone();
        config_dir.push("BepInEx");
        config_dir.push("config");
        fs::create_dir_all(&config_dir)?;

        fs_util::copy_contents(&temp_path.join("config"), &config_dir, true)
            .context("error while importing config")?;

        manifest
            .mods
            .into_iter()
            .map(|export_mod| export_mod.mod_ref(&thunderstore))
            .collect::<Result<Vec<_>>>()
            .context("failed to resolve mod references")?
    };

    downloader::install_mod_refs(&mod_refs, app)
        .await
        .context("error while importing mods")?;

    Ok(())
}

async fn import_code(key: Uuid, app: &AppHandle) -> Result<()> {
    let client = app.state::<NetworkClient>();
    let client = &client.0;

    let response = client
        .get(&format!("https://thunderstore.io/api/experimental/legacyprofile/get/{key}/"))
        .send()
        .await?
        .error_for_status()
        .map_err(|err| match err.status() {
            Some(status) if status == StatusCode::NOT_FOUND => {
                anyhow!("profile code is expired or invalid")
            }
            _ => err.into(),
        })?
        .text()
        .await?;

    match response.strip_prefix(PROFILE_DATA_PREFIX) {
        Some(data) => {
            let bytes = BASE64_STANDARD
                .decode(data)
                .context("failed to decode base64 data")?;

            import_file(Cursor::new(bytes), app).await
        }
        None => Err(anyhow!("invalid profile data")),
    }
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
        .map(|mod_ref| {
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

    let mut zip = fs_util::zip(path)?;

    zip.write_str("manifest.json", &serde_json::to_string_pretty(&manifest)?)?;

    let readme = format!("# {}\n\n{}", manifest.name, manifest.description);
    zip.write_str("README.md", &readme)?;

    let img = ImageReader::open(&args.icon)?.decode()?;
    img.resize_exact(256, 256, FilterType::Lanczos3);
    zip.write("icon.png", img.as_bytes())?;

    write_config(profile, &mut zip)?;

    Ok(())
}

fn write_config(profile: &Profile, zip: &mut fs_util::Zip) -> Result<()> {
    zip.add_dir("config")?;

    for file in profile.config.iter().flatten() {
        let content = config::ser::to_string(file);
        let path = file.path_relative();
        let path = path.strip_prefix("BepInEx").unwrap();
        zip.write_str(path.to_str().unwrap(), &content)
            .fs_context("writing config file", path)?;
    }

    Ok(())
}

async fn import_local_mod(mut path: PathBuf, app: &AppHandle) -> Result<()> {
    ensure!(path.is_dir(), "mod path is not a directory");

    path.push("manifest.json");
    let manifest = match path.exists() {
        true => {
            let json = fs::read_to_string(&path).fs_context("reading manifest", &path)?;

            let manifest: PackageManifest =
                serde_json::from_str(&json).context("failed to parse manifest")?;

            Some(manifest)
        }
        false => None,
    };
    path.pop();

    let uuid = Uuid::new_v4();

    let mut local_mod = match manifest {
        Some(manifest) => LocalMod {
            uuid,
            name: manifest.name,
            author: manifest.author,
            description: Some(manifest.description),
            version: Some(manifest.version_number),
            dependencies: Some(manifest.dependencies),
            ..Default::default()
        },
        None => LocalMod {
            uuid,
            name: fs_util::file_name(&path),
            ..Default::default()
        },
    };

    if let Some(ref deps) = local_mod.dependencies {
        downloader::install_mods(|manager, thunderstore| {
            let profile = manager.active_profile();

            thunderstore
                .resolve_deps(deps)
                .filter_ok(|dep| !profile.has_mod(&dep.package.uuid4))
                .map_ok(|borrowed_mod| ModRef::from(&borrowed_mod))
                .collect::<Result<Vec<_>>>()
                .context("failed to resolve dependencies")
        }, app).await?;
    }

    {
        let manager = app.state::<Mutex<ModManager>>();
        let thunderstore = app.state::<Mutex<Thunderstore>>();
        let prefs = app.state::<Mutex<Prefs>>();

        let mut manager = manager.lock().unwrap();
        let thunderstore = thunderstore.lock().unwrap();
        let prefs = prefs.lock().unwrap();

        let profile = manager.active_profile_mut();

        if profile.local_mods().any(|m| m.name == local_mod.name) {
            profile.force_remove_mod(&uuid, &thunderstore)
                .context("failed to remove existing mod")?;
        }
    
        downloader::install_from_disk(&path, &profile.path, &local_mod.name)
            .context("failed to install local mod")?;

        let mut mod_path = profile.path.clone();
        mod_path.push("BepInEx");
        mod_path.push("plugins");
        mod_path.push(&local_mod.name);

        downloader::normalize_mod_structure(&mut mod_path)?;

        mod_path.push("icon.png");
        if mod_path.exists() {
            local_mod.icon = Some(mod_path);
        }
    
        profile.mods.push(ProfileMod::Local(local_mod));

        save(&manager, &prefs)?;
    }

    Ok(())
}
