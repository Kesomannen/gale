use std::{
    borrow::Cow,
    fs,
    path::{Path, PathBuf},
};

use eyre::Result;

use super::{FileInstallMethod, PackageInstaller, PackageZip};
use crate::profile::{
    Profile, ProfileMod,
    install::{self, fs::ConflictResolution},
};

pub struct BepinexInstaller;

/// Sets up Gale's launcher and the universal doorstop in the profile, which
/// BepInEx needs to load on macOS. Failures are only logged: the launch path
/// does the same setup again and reports errors there.
#[cfg(target_os = "macos")]
fn macos_post_install(profile_dir: &Path) {
    use std::time::Duration;

    use tracing::warn;

    use crate::profile::launch::macos;

    const HTTP_TIMEOUT: Duration = Duration::from_secs(30);

    if let Err(err) = macos::ensure_launcher(profile_dir) {
        warn!("failed to write macOS BepInEx launcher: {:#}", err);
    }

    let profile_dir = profile_dir.to_path_buf();
    tauri::async_runtime::spawn(async move {
        // mirror the app client's timeouts (state.rs): without them a stalled
        // connection would park this task forever instead of failing and logging
        let base = match reqwest::Client::builder()
            .connect_timeout(HTTP_TIMEOUT)
            .read_timeout(HTTP_TIMEOUT)
            .build()
        {
            Ok(client) => client,
            Err(err) => {
                warn!("failed to build HTTP client for macOS doorstop: {:#}", err);
                return;
            }
        };
        let http = reqwest_middleware::ClientBuilder::new(base).build();

        if let Err(err) = macos::ensure_doorstop_with(&profile_dir, &http).await {
            warn!("failed to set up doorstop for macOS: {:#}", err);
        }
    });
}

fn get_core_path(package_name: &str) -> PathBuf {
    const CORE_PATH: &str = "BepInEx/core";
    match package_name {
        "ResoniteModding-BepInExRenderer" => PathBuf::from("Renderer").join(CORE_PATH),
        _ => PathBuf::from(CORE_PATH),
    }
}

fn scan(profile: &Profile, package_name: &str) -> Result<Vec<PathBuf>> {
    let core_dir = profile.path.join(get_core_path(package_name));

    if !core_dir.exists() {
        return Ok(Vec::new());
    }

    Ok(core_dir
        .read_dir()?
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|ty| ty.is_file()))
        .map(|entry| entry.path())
        .collect())
}

impl PackageInstaller for BepinexInstaller {
    fn extract(&mut self, archive: PackageZip, _package_name: &str, dest: PathBuf) -> Result<()> {
        install::fs::extract(archive, dest, |relative_path| {
            let mut components = relative_path.components();
            if components.clone().count() == 1 {
                // ignore top-level files, such as manifest.json and icon.png
                return Ok(None);
            }

            // remove the top-level dir (usually called BepInExPack)
            components.next();

            Ok(Some(Cow::Borrowed(components.as_path())))
        })
    }

    fn install(&mut self, src: &Path, _package_name: &str, profile: &Profile) -> Result<()> {
        install::fs::install(src, profile, |relative_path, _| {
            if relative_path.extension().is_some_and(|ext| ext == "cfg") {
                Ok((FileInstallMethod::Copy, ConflictResolution::Skip))
            } else {
                Ok((FileInstallMethod::Link, ConflictResolution::Overwrite))
            }
        })?;

        #[cfg(target_os = "macos")]
        macos_post_install(&profile.path);

        Ok(())
    }

    fn toggle(&mut self, enabled: bool, profile_mod: &ProfileMod, profile: &Profile) -> Result<()> {
        for file in scan(profile, &profile_mod.full_name())? {
            install::fs::toggle_file(file, enabled)?;
        }

        Ok(())
    }

    fn uninstall(&mut self, profile_mod: &ProfileMod, profile: &Profile) -> Result<()> {
        for file in scan(profile, &profile_mod.full_name())? {
            fs::remove_file(file)?;
        }

        Ok(())
    }

    fn mod_dir(&self, package_name: &str, profile: &Profile) -> Option<PathBuf> {
        Some(profile.path.join(get_core_path(package_name)))
    }
}
