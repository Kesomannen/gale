use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use eyre::{Context, Result, bail, ensure};
use tauri::AppHandle;
use tracing::warn;
use uuid::Uuid;

use crate::{
    profile::{
        Profile, ProfileMod,
        install::{ModInstall, restore_package_state},
    },
    state::ManagerExt,
    util,
};

/// Everything needed to undo an [`incremental_update`](super::incremental_update) after the install batch fails.
///
/// Removed mods' files are moved aside into `_state/revert` rather than deleted,
/// so restoring them is a pure filesystem replay with no network dependency.
#[derive(Default)]
pub(crate) struct ImportRevert {
    pub(super) removed: Vec<RemovedModBackup>,
    pub(super) toggled: Vec<Uuid>,
    /// Profile-relative paths claimed by the mods being installed.
    /// A file or dir under one of these that appears while the originals are moved aside
    /// is a remnant of the failed install and may be removed during restore.
    pub(super) replacement_paths: HashSet<PathBuf>,
}

pub(super) struct RemovedModBackup {
    pub(super) profile_mod: ProfileMod,
    index: usize,
    /// Profile-relative paths moved into `revert_dir/<uuid>/`.
    paths: Vec<PathBuf>,
    /// `_state/<full_name>.json` was copied into the revert dir.
    has_pkg_state: bool,
}

pub(crate) fn revert_dir(profile_path: &Path) -> PathBuf {
    profile_path.join("_state").join("revert")
}

/// Drops the revert dir once the caller has confirmed the installed state.
pub(crate) fn clear_revert_dir(profile_path: &Path) {
    let dir = revert_dir(profile_path);
    if dir.exists() {
        fs::remove_dir_all(&dir).unwrap_or_else(|err| {
            warn!("failed to remove revert dir {}: {err}", dir.display());
        });
    }
}

fn package_state_path(full_name: &str, profile: &Profile) -> PathBuf {
    profile
        .path
        .join("_state")
        .join(format!("{full_name}.json"))
}

/// Moves a removed mod's files into the revert dir, then removes the mod.
///
/// The uninstall afterwards still runs its bookkeeping (Track-mode state
/// cleanup) on the now-missing files.
///
/// The [`RemovedModBackup`] is recorded even when this fails part-way, so the
/// caller can restore whatever was moved.
pub(super) fn backup_removed_mod(
    profile: &mut Profile,
    package_uuid: Uuid,
    revert: &mut ImportRevert,
) -> Result<()> {
    let index = profile.index_of(package_uuid)?;
    let profile_mod = profile.mods[index].clone();
    let full_name = profile_mod.full_name().into_owned();

    let installer = profile.game.mod_loader.installer_for(&full_name);
    let mod_revert_dir = revert_dir(&profile.path).join(package_uuid.to_string());

    let mut backup = RemovedModBackup {
        profile_mod,
        index,
        paths: Vec::new(),
        has_pkg_state: false,
    };

    let result = (|| -> Result<()> {
        for path in installer.installed_paths(&backup.profile_mod, profile)? {
            // disabled files live under a `.old` suffix; move both variants if present
            for candidate in [path.clone(), with_old_extension(&path)] {
                if fs::symlink_metadata(&candidate).is_err() {
                    continue;
                }

                let rel = candidate
                    .strip_prefix(&profile.path)
                    .with_context(|| {
                        format!(
                            "mod file {} is outside the profile directory",
                            candidate.display()
                        )
                    })?
                    .to_path_buf();
                let dest = mod_revert_dir.join(&rel);

                fs::create_dir_all(dest.parent().unwrap())?;
                fs::rename(&candidate, &dest).with_context(|| {
                    format!("failed to back up mod file {}", candidate.display())
                })?;
                backup.paths.push(rel);
            }
        }

        // copy, don't move: `uninstall` still needs the original for Track-mode cleanup
        let state_file = package_state_path(&full_name, profile);
        backup.has_pkg_state = state_file.is_file();
        if backup.has_pkg_state {
            fs::create_dir_all(&mod_revert_dir)?;
            fs::copy(&state_file, mod_revert_dir.join("pkg_state.json"))?;
        }

        profile.force_remove_mod(package_uuid)
    })();

    revert.removed.push(backup);
    result
}

fn with_old_extension(path: &Path) -> PathBuf {
    let mut path = path.to_path_buf();
    path.add_extension("old");
    path
}

/// Replays an [`ImportRevert`] on a profile: moves backed-up files back and
/// restores mod records and toggles.
///
/// Mods whose files can't be moved back stay removed (a degraded but
/// consistent state) and are reported in the returned error. The revert dir is
/// only cleaned up when every mod was restored.
fn restore_revert(profile: &mut Profile, revert: ImportRevert) -> Result<()> {
    let revert_dir = revert_dir(&profile.path);

    let mut unrestored = Vec::new();

    // undo removals in reverse order: each backup's index is the mod's position
    // in the list *after* the mods removed before it were taken out, so
    // reinserting last-removed-first reconstructs the original order
    for backup in revert.removed.into_iter().rev() {
        let uuid = backup.profile_mod.uuid();
        let full_name = backup.profile_mod.full_name().into_owned();
        let mod_revert_dir = revert_dir.join(uuid.to_string());

        let mut failed = false;
        for rel in &backup.paths {
            let src = mod_revert_dir.join(rel);
            let dest = profile.path.join(rel);

            if let Err(err) = restore_path(&src, &dest, rel, &revert.replacement_paths) {
                warn!(
                    %full_name,
                    path = %rel.display(),
                    "failed to restore backed-up mod file: {err}"
                );
                failed = true;
            }
        }

        if failed {
            unrestored.push(full_name);
            continue;
        }

        if backup.has_pkg_state {
            let backup_file = mod_revert_dir.join("pkg_state.json");
            restore_package_state(profile, &full_name, &backup_file).unwrap_or_else(|err| {
                warn!(%full_name, "failed to restore tracked mod files: {err:#}");
            });
        }

        // the record is still present when the backup failed before the mod
        // could be removed
        if profile.index_of(uuid).is_err() {
            profile
                .mods
                .insert(backup.index.min(profile.mods.len()), backup.profile_mod);
        }
    }

    for uuid in revert.toggled {
        profile
            .force_toggle_mod(uuid)
            .unwrap_or_else(|err| warn!(%uuid, "failed to restore mod state: {err}"));
    }

    if unrestored.is_empty() {
        fs::remove_dir_all(&revert_dir).unwrap_or_else(|err| {
            warn!(
                "failed to remove revert dir {}: {err}",
                revert_dir.display()
            );
        });
        Ok(())
    } else {
        // keep the revert dir so the files can be recovered manually
        bail!(
            "failed to restore mod(s): {}; backed-up files were left in {}",
            unrestored.join(", "),
            revert_dir.display()
        );
    }
}

/// Moves one backed-up path back into the profile.
///
/// If the destination is occupied, the occupant is only removed when the
/// failed install claimed that path (a leftover remnant); an identical file
/// already in place counts as restored. Anything else is left alone and
/// reported, since it can't be told apart from an unrelated user file.
fn restore_path(
    src: &Path,
    dest: &Path,
    rel: &Path,
    replacement_paths: &HashSet<PathBuf>,
) -> Result<()> {
    if fs::symlink_metadata(dest).is_ok() {
        if src.is_file() && dest.is_file() && util::fs::checksum(src)? == util::fs::checksum(dest)?
        {
            // an identical file is already in place
            fs::remove_file(src)?;
            return Ok(());
        }

        let claimed = replacement_paths
            .iter()
            .any(|path| rel.starts_with(path) || path.starts_with(rel));

        ensure!(
            claimed,
            "destination {} already exists and is not part of the failed install",
            dest.display()
        );

        if dest.is_dir() {
            fs::remove_dir_all(dest)?;
        } else {
            fs::remove_file(dest)?;
        }
    }

    fs::create_dir_all(dest.parent().unwrap())?;
    fs::rename(src, dest)?;
    Ok(())
}

/// The profile-relative paths the pending installs claim ownership of, used to
/// tell failed-install remnants apart from unrelated files during restore.
pub(super) fn replacement_paths(installs: &[ModInstall], profile: &Profile) -> HashSet<PathBuf> {
    let mut paths = HashSet::new();
    for install in installs {
        let profile_mod = install.profile_mod();
        let full_name = profile_mod.full_name().into_owned();
        let installer = profile.game.mod_loader.installer_for(&full_name);

        let installed = installer
            .installed_paths(&profile_mod, profile)
            .unwrap_or_else(|err| {
                warn!(%full_name, "failed to enumerate replacement mod paths: {err:#}");
                Vec::new()
            });

        for path in installed
            .into_iter()
            .chain(installer.mod_dir(&full_name, profile))
        {
            if let Ok(rel) = path.strip_prefix(&profile.path) {
                paths.insert(rel.to_path_buf());
            }
        }
    }
    paths
}

pub(crate) fn restore_imported_profile(
    profile_id: i64,
    revert: ImportRevert,
    app: &AppHandle,
) -> Result<()> {
    let mut manager = app.lock_manager();
    let (_, profile) = manager.profile_by_id_mut(profile_id)?;

    let result = restore_revert(profile, revert);
    profile.save(app, true)?;
    result
}

/// Restores an in-progress [`ImportRevert`] after `incremental_update` failed
/// part-way, then returns the error that should be surfaced.
pub(super) fn restore_after_failed_update(
    profile: &mut Profile,
    revert: ImportRevert,
    err: eyre::Report,
) -> eyre::Report {
    match restore_revert(profile, revert) {
        Ok(()) => err,
        Err(restore_err) => err.wrap_err(format!(
            "failed to fully restore the previous mod set; \
             backed-up files are preserved in {}: {restore_err:#}",
            revert_dir(&profile.path).display()
        )),
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;
    use crate::{
        config::ConfigCache,
        game,
        profile::{ProfileModKind, ThunderstoreMod},
        thunderstore::{Backend, ModId},
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
    fn revert_restores_removed_mods_and_files() {
        let dir = tempdir().unwrap();
        let id = mod_id(1);
        let ident = "Author-Mod-1.0.0";
        let package_name = "Author-Mod";

        let mut profile = profile_at(dir.path(), vec![ts_mod(ident, id.clone(), true)]);

        let plugin_dir = dir.path().join("BepInEx/plugins").join(package_name);
        fs::create_dir_all(&plugin_dir).unwrap();
        fs::write(plugin_dir.join("plugin.dll"), b"data").unwrap();

        let mut revert = ImportRevert::default();
        backup_removed_mod(&mut profile, id.package_uuid, &mut revert).unwrap();

        assert!(profile.mods.is_empty());
        assert!(!plugin_dir.exists());
        assert!(
            revert_dir(dir.path())
                .join(id.package_uuid.to_string())
                .join("BepInEx/plugins/Author-Mod/plugin.dll")
                .is_file()
        );

        restore_revert(&mut profile, revert).unwrap();

        assert_eq!(fs::read(plugin_dir.join("plugin.dll")).unwrap(), b"data");
        assert_eq!(profile.mods.len(), 1);
        assert_eq!(profile.mods[0].uuid(), id.package_uuid);
        assert!(profile.mods[0].enabled);
        assert!(!revert_dir(dir.path()).exists());
    }

    #[test]
    fn revert_restores_record_order() {
        let dir = tempdir().unwrap();
        let id_a = mod_id(1);
        let id_b = mod_id(2);

        let mut profile = profile_at(
            dir.path(),
            vec![
                ts_mod("Author-ModA-1.0.0", id_a.clone(), true),
                ts_mod("Author-ModB-1.0.0", id_b.clone(), true),
            ],
        );

        let mut revert = ImportRevert::default();
        backup_removed_mod(&mut profile, id_a.package_uuid, &mut revert).unwrap();
        backup_removed_mod(&mut profile, id_b.package_uuid, &mut revert).unwrap();
        assert!(profile.mods.is_empty());

        restore_revert(&mut profile, revert).unwrap();

        let names: Vec<_> = profile
            .mods
            .iter()
            .map(|m| m.full_name().into_owned())
            .collect();
        assert_eq!(names, ["Author-ModA", "Author-ModB"]);
    }

    #[test]
    fn revert_restore_reports_obstructed_targets() {
        let dir = tempdir().unwrap();
        let id = mod_id(1);

        let mut profile = profile_at(
            dir.path(),
            vec![ts_mod("Author-Mod-1.0.0", id.clone(), true)],
        );

        let plugin_dir = dir.path().join("BepInEx/plugins/Author-Mod");
        fs::create_dir_all(&plugin_dir).unwrap();
        fs::write(plugin_dir.join("plugin.dll"), b"data").unwrap();

        let mut revert = ImportRevert::default();
        backup_removed_mod(&mut profile, id.package_uuid, &mut revert).unwrap();

        // obstruct the restore target: an occupied dir where the mod's dir must go
        fs::create_dir_all(&plugin_dir).unwrap();
        fs::write(plugin_dir.join("obstruction"), b"x").unwrap();

        let result = restore_revert(&mut profile, revert);

        assert!(result.is_err());
        // the mod stays removed and its backup is preserved for manual recovery
        assert!(profile.mods.is_empty());
        assert!(revert_dir(dir.path()).exists());
    }

    #[test]
    fn revert_restore_skips_still_present_record() {
        let dir = tempdir().unwrap();
        let id = mod_id(1);
        let profile_mod = ts_mod("Author-Mod-1.0.0", id.clone(), true);

        let mut profile = profile_at(dir.path(), vec![profile_mod.clone()]);

        // the state a backup failure leaves behind: files were moved aside
        // but the mod record is still in the list
        let rel = PathBuf::from("BepInEx/plugins/Author-Mod");
        let backup_dir = revert_dir(dir.path())
            .join(id.package_uuid.to_string())
            .join(&rel);
        fs::create_dir_all(&backup_dir).unwrap();
        fs::write(backup_dir.join("plugin.dll"), b"data").unwrap();

        let mut revert = ImportRevert::default();
        revert.removed.push(RemovedModBackup {
            profile_mod,
            index: 0,
            paths: vec![rel],
            has_pkg_state: false,
        });

        restore_revert(&mut profile, revert).unwrap();

        // no duplicate record; the files are back in place
        assert_eq!(profile.mods.len(), 1);
        assert_eq!(profile.mods[0].uuid(), id.package_uuid);
        assert_eq!(
            fs::read(dir.path().join("BepInEx/plugins/Author-Mod/plugin.dll")).unwrap(),
            b"data"
        );
        assert!(!revert_dir(dir.path()).exists());
    }

    #[test]
    fn revert_restore_removes_claimed_remnants() {
        let dir = tempdir().unwrap();
        let id = mod_id(1);

        let mut profile = profile_at(
            dir.path(),
            vec![ts_mod("Author-Mod-1.0.0", id.clone(), true)],
        );

        let plugin_dir = dir.path().join("BepInEx/plugins/Author-Mod");
        fs::create_dir_all(&plugin_dir).unwrap();
        fs::write(plugin_dir.join("plugin.dll"), b"data").unwrap();
        fs::write(dir.path().join("BepInEx/plugins/Author-Mod.old"), b"old").unwrap();

        let mut revert = ImportRevert::default();
        backup_removed_mod(&mut profile, id.package_uuid, &mut revert).unwrap();
        // the pending replacement claims the same mod dir
        revert.replacement_paths = [PathBuf::from("BepInEx/plugins/Author-Mod")]
            .into_iter()
            .collect();

        // the failed install left partial files where the originals must go
        fs::create_dir_all(&plugin_dir).unwrap();
        fs::write(plugin_dir.join("partial.dll"), b"partial").unwrap();
        // and an identical copy of a backed-up file is already in place
        fs::write(dir.path().join("BepInEx/plugins/Author-Mod.old"), b"old").unwrap();

        restore_revert(&mut profile, revert).unwrap();

        assert_eq!(profile.mods.len(), 1);
        assert_eq!(profile.mods[0].uuid(), id.package_uuid);
        // the remnant was removed and the original restored
        assert_eq!(fs::read(plugin_dir.join("plugin.dll")).unwrap(), b"data");
        assert!(!plugin_dir.join("partial.dll").exists());
        assert_eq!(
            fs::read(dir.path().join("BepInEx/plugins/Author-Mod.old")).unwrap(),
            b"old"
        );
        assert!(!revert_dir(dir.path()).exists());
    }
}
