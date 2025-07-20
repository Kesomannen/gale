use std::{
    fs,
    io::{Cursor, Read, Write},
    path::{Path, PathBuf},
};

use base64::{prelude::BASE64_STANDARD, Engine};
use eyre::{bail, ensure, Context, Result};
use tauri::AppHandle;
use tempfile::NamedTempFile;
use uuid::Uuid;
use zip::ZipArchive;

use crate::{
    game::{ModLoader, ModLoaderKind},
    prefs::Prefs,
    profile::{install::InstallOptions, LocalMod, Profile, ProfileMod},
    state::ManagerExt,
    thunderstore::PackageManifest,
    util::{self, fs::PathExt},
};

pub async fn import_local_mod_base64(
    base64: String,
    app: &AppHandle,
    options: InstallOptions,
) -> Result<()> {
    let data = BASE64_STANDARD.decode(base64)?;

    let mut file = NamedTempFile::new().context("failed to create temp file")?;
    file.write_all(&data).context("failed to write temp file")?;

    import_local_mod(
        file.path().to_owned(),
        Some(LocalModKind::Zip),
        app,
        options,
    )
    .await
}

pub async fn import_local_mod(
    path: PathBuf,
    override_kind: Option<LocalModKind>,
    app: &AppHandle,
    options: InstallOptions,
) -> Result<()> {
    let (mut local_mod, kind) = read_local_mod(&path, override_kind)?;

    if let Some(deps) = &local_mod.dependencies {
        let (profile_id, mods) = {
            let manager = app.lock_manager();
            let profile = manager.active_profile();

            let mods = app
                .lock_thunderstore()
                .dependencies(deps)
                .filter(|dep| !profile.has_mod(dep.package.uuid))
                .map(|borrowed| borrowed.into())
                .collect::<Vec<_>>();

            (profile.id, mods)
        };

        app.install_queue()
            .install(mods, profile_id, options, app)
            .await
            .context("failed to install dependencies")?;
    }

    let prefs = app.lock_prefs();
    let mut manager = app.lock_manager();

    let mod_loader = manager.active_mod_loader();
    let profile = manager.active_profile_mut();

    let existing = profile
        .local_mods()
        .find(|(LocalMod { name, .. }, _)| *name == local_mod.name);

    let existing = existing.map(|(LocalMod { uuid, .. }, _)| *uuid);

    if let Some(uuid) = existing {
        profile
            .force_remove_mod(uuid)
            .context("failed to remove existing version")?;
    }

    match kind {
        LocalModKind::Zip => {
            local_mod.icon = install_from_zip(&path, profile, &local_mod.name, mod_loader, &prefs)
                .context("failed to install")?;
        }
        LocalModKind::Dll => match mod_loader.kind {
            ModLoaderKind::BepInEx { .. } => {
                let target: PathBuf = ["BepInEx", "plugins", &local_mod.name, &local_mod.name]
                    .iter()
                    .collect();

                let mut target = profile.path.join(target);
                target.set_extension("dll");

                fs::create_dir_all(target.parent().unwrap())
                    .context("failed to create plugin directory")?;
                fs::copy(path, target).context("failed to copy file")?;
            }
            _ => bail!("currently unsupported"),
        },
    }

    profile.mods.push(ProfileMod::new_local(local_mod));

    profile.save(app.db())?;

    Ok(())
}

#[derive(PartialEq, Eq)]
pub enum LocalModKind {
    Zip,
    Dll,
}

fn read_local_mod(
    path: &Path,
    override_kind: Option<LocalModKind>,
) -> Result<(LocalMod, LocalModKind)> {
    ensure!(path.is_file(), "path is not a file");

    let kind = match (override_kind, path.extension().and_then(|ext| ext.to_str())) {
        (Some(kind), _) => kind,
        (_, Some("dll")) => LocalModKind::Dll,
        (_, Some("zip")) => LocalModKind::Zip,
        _ => bail!("unsupported file type"),
    };

    let manifest = match kind {
        LocalModKind::Zip => read_zip_manifest(path)?,
        LocalModKind::Dll => None,
    };

    let uuid = Uuid::new_v4();
    let file_size = path.metadata()?.len();

    let local_mod = match manifest {
        Some(manifest) => LocalMod {
            uuid,
            file_size,
            name: manifest.name,
            author: manifest.author,
            description: Some(manifest.description),
            version: Some(manifest.version_number),
            dependencies: Some(manifest.dependencies),
            ..Default::default()
        },
        None => LocalMod {
            uuid,
            file_size,
            name: util::fs::file_name_owned(path.with_extension("")),
            ..Default::default()
        },
    };

    Ok((local_mod, kind))
}

fn read_zip_manifest(path: &Path) -> Result<Option<PackageManifest>> {
    let mut zip = util::fs::open_zip(path).context("failed to open zip archive")?;

    let manifest = zip.by_name("manifest.json");

    match manifest {
        Ok(mut file) => {
            let mut str = String::with_capacity(file.size() as usize);
            file.read_to_string(&mut str)
                .context("failed to read manifest")?;

            // remove BOM
            if str.starts_with("\u{feff}") {
                str.replace_range(0..3, "");
            }

            serde_json::from_str(&str)
                .context("failed to parse manifest")
                .map(Some)
        }
        Err(_) => Ok(None),
    }
}

fn install_from_zip(
    src: &Path,
    profile: &Profile,
    package_name: &str,
    mod_loader: &'static ModLoader,
    prefs: &Prefs,
) -> Result<Option<PathBuf>> {
    // dont use tempdir since we need the files on the same drive as the destination
    // for hard linking to work

    let temp_path = prefs.data_dir.join("temp").join("extract");
    fs::create_dir_all(&temp_path).context("failed to create temporary directory")?;

    let reader = fs::read(src)
        .map(Cursor::new)
        .context("failed to read file")?;
    let archive = ZipArchive::new(reader).context("failed to read archive")?;

    let mut installer = mod_loader.installer_for(package_name);
    installer.extract(archive, package_name, temp_path.clone())?;
    installer.install(&temp_path, package_name, profile)?;

    fs::remove_dir_all(temp_path).context("failed to remove temporary directory")?;

    let icon = installer
        .mod_dir(package_name, profile)
        .and_then(|path| path.join("icon.png").exists_or_none());

    Ok(icon)
}
