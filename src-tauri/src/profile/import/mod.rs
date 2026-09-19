use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::{BufReader, Cursor, Read, Seek},
    path::{Path, PathBuf},
    sync::LazyLock,
};

use base64::{Engine, prelude::BASE64_STANDARD};
use eyre::{Context, OptionExt, Result, bail, eyre};
use futures_util::future;
use globset::{Glob, GlobSet, GlobSetBuilder};
use itertools::Itertools;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tempfile::tempdir;
use tracing::{info, trace, warn};
use uuid::Uuid;

use crate::{
    game::Game,
    prefs::Backends,
    profile::{
        export::{PROFILE_DATA_PREFIX, ProfileManifest},
        install::{InstallOptions, ModInstall},
    },
    state::ManagerExt,
    thunderstore::{Backend, ModId, Thunderstore},
    util::{self, error::IoResultExt},
};

pub mod commands;
mod local;
mod r2modman;
mod revert;

use super::Profile;
pub use local::{import_local_mod, import_local_mod_base64};
use revert::{ImportRevert, backup_removed_mod, replacement_paths, restore_after_failed_update};
pub(crate) use revert::{clear_revert_dir, restore_imported_profile, revert_dir};

pub fn read_file_at_path(path: PathBuf, thunderstore: &Thunderstore) -> Result<ImportData> {
    let file = File::open(&path).fs_context("opening file", &path)?;

    read_file(file, thunderstore)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImportData {
    pub manifest: ProfileManifest,
    pub path: PathBuf,
    pub delete_after_import: bool,
}

pub fn read_file_from(path: PathBuf, thunderstore: &Thunderstore) -> Result<ImportData> {
    let file = File::open(&path).fs_context("opening file", &path)?;

    read_file(file, thunderstore)
}

pub(super) fn read_file(
    source: impl Read + Seek,
    thunderstore: &Thunderstore,
) -> Result<ImportData> {
    let temp_dir = tempdir().context("failed to create temporary directory")?;
    util::zip::extract(source, temp_dir.path())?;

    let reader = File::open(temp_dir.path().join("export.r2x"))
        .map(BufReader::new)
        .context("failed to open profile manifest")?;

    let mut manifest: ProfileManifest =
        serde_yaml::from_reader(reader).context("failed to read profile manifest")?;

    resolve_manifest_sources(&mut manifest, thunderstore);

    Ok(ImportData {
        manifest,
        path: temp_dir.keep(),
        delete_after_import: true,
    })
}

pub(super) fn resolve_manifest_sources(
    manifest: &mut ProfileManifest,
    thunderstore: &Thunderstore,
) {
    for r2mod in &mut manifest.mods {
        // first try the backend stored in the manifest, if it's not there,
        // then try falling back to checking any other backend and update the source as needed
        if thunderstore
            .backend(r2mod.source)
            .find_ident(&r2mod.version_ident())
            .is_err()
            && let Ok(package) = thunderstore.find_ident(&r2mod.version_ident())
        {
            r2mod.source = package.package.backend;
        }
    }
}

fn read_base64(base64: &str, thunderstore: &Thunderstore) -> Result<ImportData> {
    let bytes = BASE64_STANDARD
        .decode(base64)
        .context("failed to decode base64 data")?;

    read_file(Cursor::new(bytes), thunderstore)
}

pub async fn read_code(key: Uuid, app: &AppHandle) -> Result<ImportData> {
    let response = future::join_all(
        Backends::All
            .iter()
            .map(async |backend| read_code_from_backend(backend, key, app).await),
    )
    .await
    .into_iter()
    .find_or_first(std::result::Result::is_ok)
    .unwrap()?;

    match response.strip_prefix(PROFILE_DATA_PREFIX) {
        Some(str) => read_base64(str, &app.lock_thunderstore()),
        None => Err(eyre!("invalid profile data")),
    }
}

async fn read_code_from_backend(backend: Backend, key: Uuid, app: &AppHandle) -> Result<String> {
    let response = app
        .http()
        .get(backend.profile_import(&key.to_string()))
        .send()
        .await?
        .error_for_status()
        .map_err(|err| match err.status() {
            Some(status) if status == StatusCode::NOT_FOUND => {
                eyre!("profile code is expired or invalid")
            }
            _ => err.into(),
        })?
        .text()
        .await?;

    Ok(response)
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct ImportOptions {
    import_all: bool,
    merge: bool,
    ignore_missing_mods: bool,
}

impl ImportOptions {
    pub fn ignore_missing_mods(mut self, ignore_missing_mods: bool) -> Self {
        self.ignore_missing_mods = ignore_missing_mods;
        self
    }
}

pub(super) struct ImportedProfile {
    pub id: i64,
    pub path: PathBuf,
    pub game: Game,
    pub created: bool,
    /// Originals moved aside by `incremental_update`, kept until the caller
    /// confirms the installed state. `None` for newly created profiles.
    pub revert: Option<ImportRevert>,
}

/// Which profile a manifest import should target.
#[derive(Debug)]
pub(super) enum ImportTarget {
    /// Apply the import to this exact profile id. Never activates it.
    Existing(i64),
    /// Resolve the manifest's profile name inside this game.
    Named { game: Game },
}

pub(super) async fn import_manifest(
    manifest: ProfileManifest,
    target: ImportTarget,
    options: ImportOptions,
    install_options: InstallOptions,
    app: &AppHandle,
) -> Result<ImportedProfile> {
    wait_for_profile_installs(&target, &manifest.name, app).await;

    let (to_install, imported) = prepare_import(&options, manifest, target, app)?;

    match app
        .install_queue()
        .install(to_install, imported.id, install_options, app)
        .await
    {
        // the install succeeded; the caller decides when the backed-up
        // originals are no longer needed (e.g. after verifying the
        // installed mod set)
        Ok(()) => Ok(imported),
        Err(err) => {
            if imported.created {
                cleanup_failed_profile(imported.id, app).unwrap_or_else(|err| {
                    warn!(
                        "failed to remove profile after failed or cancelled import: {}",
                        err
                    );
                });
            } else if let Some(revert) = imported.revert {
                if let Err(restore_err) = restore_imported_profile(imported.id, revert, app) {
                    return Err(eyre::eyre!(err).wrap_err(format!(
                        "failed to restore the previous mod set: {restore_err:#}"
                    )));
                }
            }

            Err(err.into())
        }
    }
}

pub(super) async fn import_profile(
    data: ImportData,
    options: ImportOptions,
    install_options: InstallOptions,
    app: &AppHandle,
) -> Result<i64> {
    info!(
        name = %data.manifest.name,
        options = ?options,
        install_options = ?install_options,
        "importing profile"
    );

    let game = app.lock_manager().active_game().game;
    let result = match import_manifest(
        data.manifest,
        ImportTarget::Named { game },
        options.clone(),
        install_options,
        app,
    )
    .await
    {
        Ok(imported) => {
            // the install succeeded; backed-up originals are no longer needed
            clear_revert_dir(&imported.path);

            match import_config(
                &imported.path,
                &data.path,
                imported.game.mod_loader.mod_config_dirs(),
                &options,
            )
            .context("error importing config")
            {
                Ok(()) => Ok(imported.id),
                Err(err) => {
                    if imported.created {
                        cleanup_failed_profile(imported.id, app).unwrap_or_else(|err| {
                            warn!(
                                "failed to remove profile after failed or cancelled import: {}",
                                err
                            );
                        });
                    }

                    Err(err)
                }
            }
        }
        Err(err) => Err(err),
    };

    if data.delete_after_import {
        fs::remove_dir_all(&data.path).unwrap_or_else(|err| {
            warn!("failed to remove source folder after import: {}", err);
        });
    }

    result
}

fn prepare_import(
    options: &ImportOptions,
    manifest: ProfileManifest,
    target: ImportTarget,
    app: &AppHandle,
) -> Result<(Vec<ModInstall>, ImportedProfile)> {
    let ProfileManifest {
        name,
        mods,
        ignored_version_updates,
        ignored_package_updates,
        ..
    } = manifest;

    let mut manager = app.lock_manager();
    let thunderstore = app.lock_thunderstore();

    let installs = mods
        .into_iter()
        .filter_map(|r2_mod| match r2_mod.to_install(&thunderstore) {
            Ok(install) => Some(Ok(install)),
            Err(err) if options.ignore_missing_mods => {
                warn!(
                    ?err,
                    ident = %r2_mod.version_ident(),
                    "ignoring missing mod during import",
                );
                None
            }
            Err(err) => Some(Err(err)),
        })
        .collect::<Result<Vec<_>>>()?;

    let (to_install, imported) = match target {
        ImportTarget::Existing(profile_id) => {
            let (game, profile) = manager.profile_by_id_mut(profile_id)?;

            let (to_install, revert) = incremental_update(options.merge, installs, profile)?;

            profile.ignored_version_updates = ignored_version_updates.into_iter().collect();
            profile.ignored_package_updates = ignored_package_updates.into_iter().collect();

            let imported = ImportedProfile {
                id: profile.id,
                path: profile.path.clone(),
                game,
                created: false,
                revert: Some(revert),
            };
            profile.save(app, true)?;

            (to_install, imported)
        }
        ImportTarget::Named { game } => {
            let game = manager
                .games
                .get_mut(&game)
                .ok_or_eyre("target game is not managed")?;

            let (profile, to_install, created, revert) = if let Some(profile_index) =
                game.find_profile_index(&name)
            {
                // overwrite an existing profile
                let profile = game.set_active_profile(profile_index)?;
                let (to_install, revert) = incremental_update(options.merge, installs, profile)?;

                (profile, to_install, false, Some(revert))
            } else {
                (
                    game.create_profile(name, None, app.db())?,
                    installs,
                    true,
                    None,
                )
            };

            profile.ignored_version_updates = ignored_version_updates.into_iter().collect();
            profile.ignored_package_updates = ignored_package_updates.into_iter().collect();

            let imported = ImportedProfile {
                id: profile.id,
                path: profile.path.clone(),
                game: game.game,
                created,
                revert,
            };
            game.save(app)?;

            (to_install, imported)
        }
    };

    Ok((to_install, imported))
}

pub(super) fn cleanup_failed_profile(profile_id: i64, app: &AppHandle) -> Result<()> {
    let mut manager = app.lock_manager();

    let (game, _) = manager.profile_by_id(profile_id)?;
    let managed_game = manager.games.get_mut(game).unwrap();

    if managed_game.profiles.len() > 1 {
        managed_game.delete_profile(profile_id, false, app.db())?;
        managed_game.save(app)?;
    } else {
        warn!("import failed for last profile");
    }

    Ok(())
}

/// Waits until the install queue has no pending or in-flight work for the
/// target profile, so a stale queued install can't satisfy or conflict with
/// the import.
async fn wait_for_profile_installs(target: &ImportTarget, name: &str, app: &AppHandle) {
    let profile_id = match target {
        ImportTarget::Existing(id) => Some(*id),
        ImportTarget::Named { game } => {
            let manager = app.lock_manager();
            manager
                .games
                .get(game)
                .and_then(|game| game.find_profile_index(name))
                .map(|index| manager.games[game].profiles[index].id)
        }
    };

    let Some(profile_id) = profile_id else {
        return;
    };

    loop {
        let notified = app.install_queue().wait_for_batch();
        tokio::pin!(notified);
        notified.as_mut().enable();

        if !app.install_queue().lock().has_any_for_profile(profile_id) {
            break;
        }

        notified.await;
    }
}

fn incremental_update(
    merge: bool,
    installs: impl IntoIterator<Item = ModInstall>,
    profile: &mut Profile,
) -> Result<(Vec<ModInstall>, ImportRevert)> {
    let current_mods: HashMap<ModId, bool> = profile
        .thunderstore_mods()
        .map(|(ts_mod, enabled)| (ts_mod.id.clone(), enabled))
        .collect();

    let current_ids: HashSet<&ModId> = current_mods.keys().collect();

    let mut new_mods: HashMap<ModId, ModInstall> = installs
        .into_iter()
        .map(|install| (install.mod_id().clone(), install))
        .collect();

    let new_ids: HashSet<&ModId> = new_mods.keys().collect();

    let revert_dir = revert_dir(&profile.path);
    if revert_dir.exists() {
        if revert_dir.read_dir()?.next().is_none() {
            fs::remove_dir(&revert_dir)?;
        } else {
            bail!(
                "a previous sync import left backed-up mod files at {}; \
                 inspect or remove it manually before retrying",
                revert_dir.display()
            );
        }
    }

    let mut revert = ImportRevert::default();

    let remove_mods: Vec<Uuid> = if merge {
        // remove only version mismatches
        current_mods
            .keys()
            .filter(|id| {
                new_mods.values().any(|install| {
                    install.mod_id().package_uuid == id.package_uuid
                        && install.mod_id().version_uuid != id.version_uuid
                })
            })
            .map(|id| id.package_uuid)
            .collect()
    } else {
        // remove all extra mods
        current_ids
            .difference(&new_ids)
            .map(|id| id.package_uuid)
            .collect()
    };

    for uuid in remove_mods {
        if let Err(err) = backup_removed_mod(profile, uuid, &mut revert) {
            return Err(restore_after_failed_update(profile, revert, err));
        }
    }

    let to_toggle: Vec<Uuid> = current_ids
        .intersection(&new_ids)
        .filter(|id| *current_mods.get(*id).unwrap() != new_mods.get(id).unwrap().enabled())
        .map(|id| id.package_uuid)
        .collect();
    for uuid in to_toggle {
        if let Err(err) = profile.force_toggle_mod(uuid) {
            return Err(restore_after_failed_update(profile, revert, err));
        }
        revert.toggled.push(uuid);
    }

    // we have to clone and collect the ids because new_ids.difference() borrows new_mods,
    // which prevents us from getting the ModInstalls back (since that requires mutably borrowing the map)

    let ids_to_install: Vec<ModId> = new_ids
        .difference(&current_ids)
        .map(|id| (*id).clone())
        .collect();

    let to_install: Vec<ModInstall> = ids_to_install
        .into_iter()
        .map(move |id| new_mods.remove(&id).unwrap())
        .collect();

    revert.replacement_paths = replacement_paths(&to_install, profile);

    Ok((to_install, revert))
}

#[tracing::instrument(skip_all, fields(dest = %dest.display(), src = %src.display()))]
pub fn import_config(
    dest: &Path,
    src: &Path,
    config_dirs: &[&str],
    options: &ImportOptions,
) -> Result<()> {
    let src_files: HashSet<PathBuf> = super::export::list_files(src)
        .filter(|path| options.import_all || is_always_imported(path))
        .collect();

    let dest_files: HashSet<PathBuf> = super::export::find_config(dest, config_dirs).collect();

    if !options.merge {
        // remove existing extra config files that are not in the imported profile
        for extra_file in dest_files.difference(&src_files) {
            let extra_path = dest.join(extra_file);
            trace!(
                relative_path = %extra_file.display(),
                "removing extra config file"
            );
            fs::remove_file(extra_path).fs_context("removing extra config file", extra_file)?;
        }
    }

    for file in src_files {
        let src_path = src.join(&file);
        let dest_path = if file.starts_with("config") {
            dest.join("BepInEx").join(&file)
        } else {
            dest.join(&file)
        };

        let need_copy = if dest_path.exists() {
            util::fs::checksum(&src_path)? != util::fs::checksum(&dest_path)?
        } else {
            true
        };

        if need_copy {
            trace!(
                relative_path = %file.display(),
                "copy file"
            );
            fs::create_dir_all(dest_path.parent().unwrap())?;
            fs::copy(src_path, dest_path)?;
        } else {
            trace!(
                relative_path = %file.display(),
                "file is identical, skipping copy"
            );
        }
    }

    Ok(())
}

fn is_always_imported(path: impl AsRef<Path>) -> bool {
    static EXCLUDE_SET: LazyLock<GlobSet> = LazyLock::new(|| {
        GlobSetBuilder::new()
            .add(Glob::new("export.r2x").unwrap())
            .add(Glob::new("mods.yml").unwrap())
            .add(Glob::new("*.{dll,exe,scr,com,pif,bat,cmd,ps1,vbs,vbe,js,jse,wsf,wsh,hta,msi,msix,sys,drv,cpl,ocx,lnk,reg,inf}").unwrap())
            .build()
            .unwrap()
    });

    !EXCLUDE_SET.is_match(path.as_ref())
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;
    use crate::{
        config::ConfigCache,
        game,
        profile::{ProfileMod, ProfileModKind, ThunderstoreMod},
    };

    fn mod_id(seed: u128) -> ModId {
        ModId {
            package_uuid: Uuid::from_u128(seed),
            version_uuid: Uuid::from_u128(seed + 0x1000),
            backend: Backend::Thunderstore,
        }
    }

    fn ts_mod(ident: &str, id: ModId, enabled: bool) -> ProfileMod {
        let mut profile_mod = ProfileMod::new(ProfileModKind::Thunderstore(ThunderstoreMod {
            ident: ident.parse().unwrap(),
            id,
        }));
        profile_mod.enabled = enabled;
        profile_mod
    }

    fn profile_at(path: &Path, mods: Vec<ProfileMod>) -> Profile {
        Profile {
            id: 0,
            name: "Test".to_owned(),
            path: path.to_owned(),
            mods,
            game: game::from_slug("among-us").unwrap(),
            ignored_version_updates: Default::default(),
            ignored_package_updates: Default::default(),
            config_cache: ConfigCache::default(),
            linked_config: Default::default(),
            modpack: None,
            sync: None,
            custom_args: String::new(),
            server_settings: None,
            missing: false,
        }
    }

    #[test]
    fn incremental_update_removes_extras_and_toggles() {
        let dir = tempdir().unwrap();

        let id_a1 = mod_id(1);
        let id_a2 = ModId {
            package_uuid: id_a1.package_uuid,
            version_uuid: Uuid::from_u128(0xA2),
            backend: Backend::Thunderstore,
        };
        let id_b = mod_id(2);
        let id_c = mod_id(3);

        let mut profile = profile_at(
            dir.path(),
            vec![
                ts_mod("Author-ModA-1.0.0", id_a1.clone(), true),
                ts_mod("Author-ModB-1.0.0", id_b.clone(), true),
                ts_mod("Author-ModC-1.0.0", id_c.clone(), true),
            ],
        );

        let installs = vec![
            ModInstall::test(
                "Author-ModA-2.0.0",
                id_a2.package_uuid,
                id_a2.version_uuid,
                true,
            ),
            ModInstall::test(
                "Author-ModC-1.0.0",
                id_c.package_uuid,
                id_c.version_uuid,
                false,
            ),
        ];

        let (to_install, revert) = incremental_update(false, installs, &mut profile).unwrap();

        // A v1 and B were removed, C was toggled off
        assert_eq!(profile.mods.len(), 1);
        assert_eq!(profile.mods[0].uuid(), id_c.package_uuid);
        assert!(!profile.mods[0].enabled);

        let mut removed: Vec<_> = revert
            .removed
            .iter()
            .map(|backup| backup.profile_mod.uuid())
            .collect();
        removed.sort();
        assert_eq!(removed, vec![id_a1.package_uuid, id_b.package_uuid]);
        assert_eq!(revert.toggled, vec![id_c.package_uuid]);

        assert_eq!(to_install.len(), 1);
        assert_eq!(to_install[0].uuid(), id_a2.package_uuid);
    }

    #[test]
    fn incremental_update_aborts_on_nonempty_revert_dir() {
        let dir = tempdir().unwrap();
        let revert_dir = revert_dir(dir.path());
        fs::create_dir_all(&revert_dir).unwrap();
        fs::write(revert_dir.join("leftover"), b"data").unwrap();

        let mut profile = profile_at(
            dir.path(),
            vec![ts_mod("Author-Mod-1.0.0", mod_id(1), true)],
        );

        let result = incremental_update(false, Vec::new(), &mut profile);

        assert!(result.is_err());
        // the leftover backup is preserved, not silently deleted
        assert!(revert_dir.join("leftover").is_file());
        assert_eq!(profile.mods.len(), 1);
    }

    #[test]
    fn incremental_update_restores_when_update_fails_after_backup() {
        let dir = tempdir().unwrap();
        let id_a = mod_id(1);
        let id_c = mod_id(3);

        let mut profile = profile_at(
            dir.path(),
            vec![
                ts_mod("Author-ModA-1.0.0", id_a.clone(), true),
                ts_mod("Author-ModC-1.0.0", id_c.clone(), true),
            ],
        );

        let plugin_dir_a = dir.path().join("BepInEx/plugins/Author-ModA");
        fs::create_dir_all(&plugin_dir_a).unwrap();
        fs::write(plugin_dir_a.join("plugin.dll"), b"data").unwrap();

        // ModC is kept but toggled off; a directory where a disabled file's
        // `.old` name must go makes the toggle fail after ModA was backed up
        let plugin_dir_c = dir.path().join("BepInEx/plugins/Author-ModC");
        fs::create_dir_all(&plugin_dir_c).unwrap();
        fs::write(plugin_dir_c.join("plugin.dll"), b"data").unwrap();
        fs::create_dir_all(plugin_dir_c.join("plugin.dll.old")).unwrap();

        let installs = vec![ModInstall::test(
            "Author-ModC-1.0.0",
            id_c.package_uuid,
            id_c.version_uuid,
            false,
        )];

        let result = incremental_update(false, installs, &mut profile);

        assert!(result.is_err());
        // ModA was moved out and then restored; ModC's toggle never applied
        assert_eq!(profile.mods.len(), 2);
        assert_eq!(profile.mods[0].uuid(), id_a.package_uuid);
        assert!(profile.mods[0].enabled);
        assert_eq!(profile.mods[1].uuid(), id_c.package_uuid);
        assert!(profile.mods[1].enabled);
        assert_eq!(fs::read(plugin_dir_a.join("plugin.dll")).unwrap(), b"data");
        // a successful restore cleans the revert dir so a retry can proceed
        assert!(!revert_dir(dir.path()).exists());
    }
}
