use std::{
    collections::{BTreeMap, HashSet},
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use eyre::{Context, OptionExt, Result, bail, ensure};
use serde::Serialize;

use super::{AppliedState, PendingConfigReason, archive};
use crate::profile::export::{self, ConfigPath, ContentHash};

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PendingConfigUpdate {
    pub path: ConfigPath,
    pub reason: PendingConfigReason,
    pub declined: bool,
}

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ConfigApplyReport {
    pub installed: Vec<ConfigPath>,
    pub pending: Vec<PendingConfigUpdate>,
}

pub(super) fn snapshot_config(
    root: &Path,
    config_dirs: &[&str],
) -> Result<BTreeMap<ConfigPath, ContentHash>> {
    export::collect_config_files(root, config_dirs)?
        .iter()
        .map(|(path, bytes)| Ok((path.clone(), ContentHash::from_hash(blake3::hash(bytes)))))
        .collect()
}

pub(super) fn record_installer_written(
    before: &BTreeMap<ConfigPath, ContentHash>,
    after: &BTreeMap<ConfigPath, ContentHash>,
    state: &mut AppliedState,
) {
    for (path, hash) in after {
        if before.contains_key(path) {
            continue;
        }

        state.config.entry(path.clone()).or_default().written = Some(hash.clone());
    }
}

fn checked_target(profile_dir: &Path, path: &ConfigPath) -> Result<PathBuf> {
    let mut target = profile_dir.to_path_buf();
    for component in path.as_path().components() {
        target.push(component);
        match fs::symlink_metadata(&target) {
            Ok(metadata) => ensure!(
                !metadata.file_type().is_symlink(),
                "config path traverses a symlink: {}",
                target.display()
            ),
            Err(err) if err.kind() == io::ErrorKind::NotFound => (),
            Err(err) => {
                return Err(err).with_context(|| {
                    format!("failed to inspect config path: {}", target.display())
                });
            }
        }
    }

    Ok(target)
}

fn write_validated(target: &Path, file: &archive::ValidatedConfigFile) -> Result<()> {
    ensure!(
        file.hash == ContentHash::from_hash(blake3::hash(&file.bytes)),
        "validated config bytes do not match their hash: {}",
        target.display()
    );

    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create config dir: {}", parent.display()))?;
    }

    let staged = stage_validated(target, file)?;
    commit_staged(&staged, target, &file.hash)
}

fn staging_candidates(target: &Path) -> Result<impl Iterator<Item = String>> {
    let name = target
        .file_name()
        .ok_or_eyre("config target has no file name")?
        .to_string_lossy()
        .into_owned();

    Ok((0..16).map(move |_| format!("{name}.{}.gale-sync-tmp", uuid::Uuid::new_v4().simple())))
}

fn create_staging_file(
    target: &Path,
    candidates: impl Iterator<Item = String>,
) -> Result<(PathBuf, fs::File)> {
    for candidate in candidates {
        let path = target.with_file_name(candidate);
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(file) => return Ok((path, file)),
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(err) => {
                return Err(err)
                    .with_context(|| format!("failed to create staging file: {}", path.display()));
            }
        }
    }

    bail!(
        "failed to create a unique staging file for {}",
        target.display()
    )
}

fn stage_validated(target: &Path, file: &archive::ValidatedConfigFile) -> Result<PathBuf> {
    let (staged, mut handle) = create_staging_file(target, staging_candidates(target)?)?;

    let result = handle
        .write_all(&file.bytes)
        .and_then(|_| handle.sync_all());
    drop(handle);

    if let Err(err) = result {
        let _ = fs::remove_file(&staged);
        return Err(err)
            .with_context(|| format!("failed to stage config file: {}", staged.display()));
    }

    Ok(staged)
}

fn commit_staged(staged: &Path, target: &Path, hash: &ContentHash) -> Result<()> {
    let result = (|| -> Result<()> {
        let written = fs::read(staged).with_context(|| {
            format!("failed to verify staged config file: {}", staged.display())
        })?;
        ensure!(
            ContentHash::from_hash(blake3::hash(&written)) == *hash,
            "staged config file does not match its hash: {}",
            staged.display()
        );

        fs::rename(staged, target)
            .with_context(|| format!("failed to replace config file: {}", target.display()))?;
        Ok(())
    })();

    if result.is_err() {
        let _ = fs::remove_file(staged);
    }

    result
}

fn record_applied(state: &mut AppliedState, path: &ConfigPath, hash: ContentHash) {
    let record = state.config.entry(path.clone()).or_default();
    record.applied = Some(hash.clone());
    record.written = Some(hash);
    record.declined = None;
    state.pending.remove(path);
}

pub(super) fn apply_available_config(
    profile_dir: &Path,
    config: &BTreeMap<ConfigPath, archive::ValidatedConfigFile>,
    state: &mut AppliedState,
) -> Result<ConfigApplyReport> {
    state.pending.retain(|path, _| config.contains_key(path));

    let mut report = ConfigApplyReport::default();

    for (path, file) in config {
        let target = checked_target(profile_dir, path)?;
        let local = if target.exists() {
            ensure!(
                target.is_file(),
                "synced config path is not a regular file: {}",
                target.display()
            );
            let bytes = fs::read(&target)
                .with_context(|| format!("failed to read config file: {}", target.display()))?;
            Some(ContentHash::from_hash(blake3::hash(&bytes)))
        } else {
            None
        };
        let hash = file.hash.clone();

        if local.as_ref() == Some(&hash) {
            record_applied(state, path, hash);
            continue;
        }

        if state
            .config
            .get(path)
            .and_then(|record| record.declined.as_ref())
            == Some(&hash)
        {
            let reason = if local.is_none() {
                PendingConfigReason::DeletedLocally
            } else {
                PendingConfigReason::ModifiedLocally
            };
            state.pending.entry(path.clone()).or_insert(reason);
            continue;
        }

        match local {
            None => {
                if state.config.contains_key(path) {
                    state
                        .pending
                        .insert(path.clone(), PendingConfigReason::DeletedLocally);
                    continue;
                }

                write_validated(&target, file)?;
                record_applied(state, path, hash);
                report.installed.push(path.clone());
            }
            Some(local_hash) => {
                let record = state.config.get(path);

                if record.and_then(|r| r.applied.as_ref()) == Some(&hash) {
                    state.pending.remove(path);
                    continue;
                }

                if record.and_then(|r| r.written.as_ref()) == Some(&local_hash) {
                    write_validated(&target, file)?;
                    record_applied(state, path, hash);
                    report.installed.push(path.clone());
                    continue;
                }

                state
                    .pending
                    .insert(path.clone(), PendingConfigReason::ModifiedLocally);
            }
        }
    }

    report.pending = review_items(state);
    Ok(report)
}

pub(super) fn review_items(state: &AppliedState) -> Vec<PendingConfigUpdate> {
    state
        .pending
        .iter()
        .filter(|(path, _)| {
            state
                .latest
                .as_ref()
                .is_some_and(|latest| latest.config.contains_key(*path))
        })
        .map(|(path, reason)| {
            let declined = state
                .latest
                .as_ref()
                .and_then(|latest| latest.config.get(path))
                .zip(state.config.get(path).and_then(|r| r.declined.as_ref()))
                .is_some_and(|(entry, declined)| entry.hash == *declined);

            PendingConfigUpdate {
                path: path.clone(),
                reason: *reason,
                declined,
            }
        })
        .collect()
}

pub(super) fn apply_selected(
    profile_dir: &Path,
    config: &BTreeMap<ConfigPath, archive::ValidatedConfigFile>,
    state: &mut AppliedState,
    files: &[ConfigPath],
) -> Result<Vec<ConfigPath>> {
    let mut seen = HashSet::new();
    for path in files {
        ensure!(
            seen.insert(path.clone()),
            "duplicate selected config path: {path}"
        );
    }

    {
        let latest = state
            .latest
            .as_ref()
            .ok_or_eyre("no published sync state")?;
        for path in files {
            let advertised = latest
                .config
                .get(path)
                .ok_or_eyre("selected config file is no longer published: {path}")?;
            let file = config
                .get(path)
                .ok_or_eyre("selected config file is missing from the archive: {path}")?;
            ensure!(
                file.hash == advertised.hash,
                "selected config file no longer matches the published revision: {path}"
            );
        }
    }

    let mut written = Vec::with_capacity(files.len());
    for path in files {
        let file = &config[path];
        let hash = file.hash.clone();

        write_validated(&checked_target(profile_dir, path)?, file)?;
        record_applied(state, path, hash);
        written.push(path.clone());
    }

    Ok(written)
}

pub(super) fn decline_selected(state: &mut AppliedState, files: &[ConfigPath]) -> Result<()> {
    let mut seen = HashSet::new();
    for path in files {
        ensure!(
            seen.insert(path.clone()),
            "duplicate selected config path: {path}"
        );
    }

    let mut entries = Vec::with_capacity(files.len());
    {
        let latest = state
            .latest
            .as_ref()
            .ok_or_eyre("no published sync state")?;
        for path in files {
            let advertised = latest
                .config
                .get(path)
                .ok_or_eyre("selected config file is no longer published: {path}")?;
            ensure!(
                state.pending.contains_key(path),
                "config file is not pending review: {path}"
            );
            entries.push((path.clone(), advertised.hash.clone()));
        }
    }

    for (path, hash) in entries {
        state.config.entry(path).or_default().declined = Some(hash);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;
    use crate::profile::{
        export::{ModRevision, SyncFileEntry, SyncManifest},
        sync::AppliedFile,
    };

    fn path(path: &str) -> ConfigPath {
        ConfigPath::try_from(path.to_owned()).unwrap()
    }

    fn hash(bytes: &[u8]) -> ContentHash {
        ContentHash::from_hash(blake3::hash(bytes))
    }

    fn vfile(bytes: &[u8]) -> archive::ValidatedConfigFile {
        archive::ValidatedConfigFile {
            hash: hash(bytes),
            bytes: bytes.to_vec(),
        }
    }

    fn write(root: &Path, path: &ConfigPath, bytes: &[u8]) {
        let target = root.join(path.as_path());
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(target, bytes).unwrap();
    }

    fn archive_map(
        entries: &[(&str, &[u8])],
    ) -> BTreeMap<ConfigPath, archive::ValidatedConfigFile> {
        entries.iter().map(|&(p, b)| (path(p), vfile(b))).collect()
    }

    fn latest(entries: &[(&str, &[u8])]) -> SyncManifest {
        SyncManifest {
            version: 1,
            mods_revision: ModRevision::from_hash(blake3::hash(b"rev")),
            config: entries
                .iter()
                .map(|&(p, b)| (path(p), SyncFileEntry { hash: hash(b) }))
                .collect(),
        }
    }

    fn applied_file(
        applied: Option<ContentHash>,
        written: Option<ContentHash>,
        declined: Option<ContentHash>,
    ) -> AppliedFile {
        AppliedFile {
            applied,
            written,
            declined,
        }
    }

    #[test]
    fn identical_local_file_records_applied() {
        let dir = tempdir().unwrap();
        let p = path("a.cfg");
        write(dir.path(), &p, b"data");

        let mut state = AppliedState::default();
        let config = archive_map(&[("a.cfg", b"data")]);
        let report = apply_available_config(dir.path(), &config, &mut state).unwrap();

        assert!(report.installed.is_empty());
        let record = &state.config[&p];
        assert_eq!(record.applied, Some(hash(b"data")));
        assert_eq!(record.written, Some(hash(b"data")));
        assert!(record.declined.is_none());
        assert!(state.pending.is_empty());
    }

    #[test]
    fn declined_same_revision_not_rewritten() {
        let dir = tempdir().unwrap();
        let p = path("a.cfg");
        write(dir.path(), &p, b"local");

        let mut state = AppliedState::default();
        state
            .config
            .insert(p.clone(), applied_file(None, None, Some(hash(b"remote"))));
        state
            .pending
            .insert(p.clone(), PendingConfigReason::ModifiedLocally);
        state.latest = Some(latest(&[("a.cfg", b"remote")]));

        let config = archive_map(&[("a.cfg", b"remote")]);
        let report = apply_available_config(dir.path(), &config, &mut state).unwrap();

        assert!(report.installed.is_empty());
        assert_eq!(fs::read(dir.path().join(p.as_path())).unwrap(), b"local");
        assert_eq!(state.pending[&p], PendingConfigReason::ModifiedLocally);

        let review = review_items(&state);
        assert_eq!(review.len(), 1);
        assert!(review[0].declined);
        assert_eq!(review[0].reason, PendingConfigReason::ModifiedLocally);
    }

    #[test]
    fn player_edit_after_applied_revision_skips() {
        let dir = tempdir().unwrap();
        let p = path("a.cfg");
        write(dir.path(), &p, b"edited");

        let mut state = AppliedState::default();
        state.config.insert(
            p.clone(),
            applied_file(Some(hash(b"remote")), Some(hash(b"remote")), None),
        );
        state
            .pending
            .insert(p.clone(), PendingConfigReason::ModifiedLocally);

        let config = archive_map(&[("a.cfg", b"remote")]);
        let report = apply_available_config(dir.path(), &config, &mut state).unwrap();

        assert!(report.installed.is_empty());
        assert_eq!(fs::read(dir.path().join(p.as_path())).unwrap(), b"edited");
        assert!(!state.pending.contains_key(&p));
    }

    #[test]
    fn absent_no_record_installs() {
        let dir = tempdir().unwrap();
        let mut state = AppliedState::default();
        let config = archive_map(&[("a.cfg", b"remote")]);
        let report = apply_available_config(dir.path(), &config, &mut state).unwrap();

        assert_eq!(report.installed, vec![path("a.cfg")]);
        assert_eq!(fs::read(dir.path().join("a.cfg")).unwrap(), b"remote");
        let record = &state.config[&path("a.cfg")];
        assert_eq!(record.applied, Some(hash(b"remote")));
        assert_eq!(record.written, Some(hash(b"remote")));
    }

    #[test]
    fn absent_prior_record_prompts_deleted() {
        let dir = tempdir().unwrap();
        let p = path("a.cfg");

        let mut state = AppliedState::default();
        state.config.insert(
            p.clone(),
            applied_file(Some(hash(b"old")), Some(hash(b"old")), None),
        );

        let config = archive_map(&[("a.cfg", b"remote")]);
        let report = apply_available_config(dir.path(), &config, &mut state).unwrap();

        assert!(report.installed.is_empty());
        assert!(!dir.path().join("a.cfg").exists());
        assert_eq!(state.pending[&p], PendingConfigReason::DeletedLocally);
    }

    #[test]
    fn untouched_written_file_auto_updates() {
        let dir = tempdir().unwrap();
        let p = path("a.cfg");
        write(dir.path(), &p, b"v1");

        let mut state = AppliedState::default();
        state.config.insert(
            p.clone(),
            applied_file(Some(hash(b"v1")), Some(hash(b"v1")), None),
        );

        let config = archive_map(&[("a.cfg", b"v2")]);
        let report = apply_available_config(dir.path(), &config, &mut state).unwrap();

        assert_eq!(report.installed, vec![p.clone()]);
        assert_eq!(fs::read(dir.path().join(p.as_path())).unwrap(), b"v2");
        let record = &state.config[&p];
        assert_eq!(record.applied, Some(hash(b"v2")));
        assert_eq!(record.written, Some(hash(b"v2")));
        assert!(state.pending.is_empty());
    }

    #[test]
    fn unknown_local_file_prompts_modified() {
        let dir = tempdir().unwrap();
        let p = path("a.cfg");
        write(dir.path(), &p, b"custom");

        let mut state = AppliedState::default();
        let config = archive_map(&[("a.cfg", b"remote")]);
        let report = apply_available_config(dir.path(), &config, &mut state).unwrap();

        assert!(report.installed.is_empty());
        assert_eq!(fs::read(dir.path().join(p.as_path())).unwrap(), b"custom");
        assert_eq!(state.pending[&p], PendingConfigReason::ModifiedLocally);
    }

    #[test]
    fn extra_local_config_untouched() {
        let dir = tempdir().unwrap();
        write(dir.path(), &path("extra.cfg"), b"mine");

        let mut state = AppliedState::default();
        let config = archive_map(&[("a.cfg", b"remote")]);
        apply_available_config(dir.path(), &config, &mut state).unwrap();

        assert_eq!(fs::read(dir.path().join("extra.cfg")).unwrap(), b"mine");
        assert!(!state.config.contains_key(&path("extra.cfg")));
    }

    #[test]
    fn corrupt_validated_file_errors() {
        let dir = tempdir().unwrap();
        let p = path("a.cfg");
        let mut bad = vfile(b"remote");
        bad.hash = hash(b"other");

        let config = BTreeMap::from([(p.clone(), bad)]);
        let mut state = AppliedState::default();
        assert!(apply_available_config(dir.path(), &config, &mut state).is_err());
        assert!(!dir.path().join("a.cfg").exists());
        assert!(!state.config.contains_key(&p));
    }

    #[test]
    fn installer_written_records_new_paths_only() {
        let mut before = BTreeMap::new();
        before.insert(path("a.cfg"), hash(b"v1"));

        let mut after = before.clone();
        after.insert(path("b.cfg"), hash(b"v2"));
        *after.get_mut(&path("a.cfg")).unwrap() = hash(b"v1b");

        let mut state = AppliedState::default();
        state.config.insert(
            path("a.cfg"),
            applied_file(Some(hash(b"applied")), None, None),
        );

        record_installer_written(&before, &after, &mut state);

        assert_eq!(state.config[&path("b.cfg")].written, Some(hash(b"v2")));
        assert_eq!(state.config[&path("a.cfg")].applied, Some(hash(b"applied")));
        assert!(state.config[&path("a.cfg")].written.is_none());
    }

    #[test]
    fn apply_selected_applies_and_validates() {
        let dir = tempdir().unwrap();
        let p = path("a.cfg");
        write(dir.path(), &p, b"local");

        let mut state = AppliedState::default();
        state.latest = Some(latest(&[("a.cfg", b"remote"), ("b.cfg", b"bee")]));
        state
            .config
            .insert(p.clone(), applied_file(None, None, Some(hash(b"remote"))));
        state
            .pending
            .insert(p.clone(), PendingConfigReason::ModifiedLocally);

        let config = archive_map(&[("a.cfg", b"remote"), ("b.cfg", b"bee")]);

        assert!(apply_selected(dir.path(), &config, &mut state, &[p.clone(), p.clone()]).is_err());
        assert!(apply_selected(dir.path(), &config, &mut state, &[path("nope.cfg")]).is_err());

        let mut bad = vfile(b"remote");
        bad.hash = hash(b"other");
        let bad_config = BTreeMap::from([(p.clone(), bad), (path("b.cfg"), vfile(b"bee"))]);
        assert!(apply_selected(dir.path(), &bad_config, &mut state, &[p.clone()]).is_err());

        let partial = archive_map(&[("a.cfg", b"remote")]);
        assert!(apply_selected(dir.path(), &partial, &mut state, &[path("b.cfg")]).is_err());

        let mut no_latest = AppliedState::default();
        assert!(apply_selected(dir.path(), &config, &mut no_latest, &[p.clone()]).is_err());

        let written = apply_selected(dir.path(), &config, &mut state, &[p.clone()]).unwrap();
        assert_eq!(written, vec![p.clone()]);
        assert_eq!(fs::read(dir.path().join(p.as_path())).unwrap(), b"remote");
        let record = &state.config[&p];
        assert_eq!(record.applied, Some(hash(b"remote")));
        assert_eq!(record.written, Some(hash(b"remote")));
        assert!(record.declined.is_none());
        assert!(!state.pending.contains_key(&p));
    }

    #[test]
    fn decline_selected_records_revision() {
        let p = path("a.cfg");
        let q = path("b.cfg");

        let mut state = AppliedState::default();
        state.latest = Some(latest(&[("a.cfg", b"remote"), ("b.cfg", b"bee")]));
        state
            .pending
            .insert(p.clone(), PendingConfigReason::ModifiedLocally);

        assert!(decline_selected(&mut state, &[p.clone(), p.clone()]).is_err());
        assert!(decline_selected(&mut state, &[q.clone()]).is_err());
        assert!(decline_selected(&mut state, &[path("nope.cfg")]).is_err());

        let mut no_latest = AppliedState::default();
        no_latest
            .pending
            .insert(p.clone(), PendingConfigReason::ModifiedLocally);
        assert!(decline_selected(&mut no_latest, &[p.clone()]).is_err());

        decline_selected(&mut state, &[p.clone()]).unwrap();
        assert_eq!(state.config[&p].declined, Some(hash(b"remote")));
        assert_eq!(state.pending[&p], PendingConfigReason::ModifiedLocally);

        let review = review_items(&state);
        assert_eq!(review.len(), 1);
        assert!(review[0].declined);
    }

    #[test]
    fn new_revision_re_prompts_after_decline() {
        let dir = tempdir().unwrap();
        let p = path("a.cfg");
        write(dir.path(), &p, b"local");

        let mut state = AppliedState::default();
        state.latest = Some(latest(&[("a.cfg", b"v1")]));
        state
            .config
            .insert(p.clone(), applied_file(None, None, Some(hash(b"v1"))));
        state
            .pending
            .insert(p.clone(), PendingConfigReason::ModifiedLocally);
        assert!(review_items(&state)[0].declined);

        state.latest = Some(latest(&[("a.cfg", b"v2")]));
        let config = archive_map(&[("a.cfg", b"v2")]);
        apply_available_config(dir.path(), &config, &mut state).unwrap();

        assert_eq!(fs::read(dir.path().join(p.as_path())).unwrap(), b"local");
        assert_eq!(state.pending[&p], PendingConfigReason::ModifiedLocally);
        let review = review_items(&state);
        assert_eq!(review.len(), 1);
        assert!(!review[0].declined);
    }

    #[test]
    fn symlink_target_rejected() {
        let dir = tempdir().unwrap();
        let real = dir.path().join("real.cfg");
        fs::write(&real, b"real").unwrap();

        let p = path("link.cfg");
        let link = dir.path().join(p.as_path());

        #[cfg(windows)]
        let result = std::os::windows::fs::symlink_file(&real, &link);
        #[cfg(unix)]
        let result = std::os::unix::fs::symlink(&real, &link);
        if result.is_err() {
            return;
        }

        let mut state = AppliedState::default();
        let config = archive_map(&[("link.cfg", b"remote")]);
        assert!(apply_available_config(dir.path(), &config, &mut state).is_err());
        assert!(!state.config.contains_key(&p));
    }

    #[test]
    fn symlinked_dir_component_rejected() {
        let dir = tempdir().unwrap();
        let real_dir = dir.path().join("real_dir");
        fs::create_dir_all(&real_dir).unwrap();
        fs::write(real_dir.join("x.cfg"), b"real").unwrap();

        let link = dir.path().join("linkdir");
        #[cfg(windows)]
        let result = std::os::windows::fs::symlink_dir(&real_dir, &link);
        #[cfg(unix)]
        let result = std::os::unix::fs::symlink(&real_dir, &link);
        if result.is_err() {
            return;
        }

        let mut state = AppliedState::default();
        let config = archive_map(&[("linkdir/x.cfg", b"remote")]);
        assert!(apply_available_config(dir.path(), &config, &mut state).is_err());
        assert_eq!(fs::read(real_dir.join("x.cfg")).unwrap(), b"real");
    }

    #[test]
    fn snapshot_config_hashes_files() {
        let dir = tempdir().unwrap();
        write(dir.path(), &path("BepInEx/config/a.cfg"), b"data");
        write(dir.path(), &path("other/b.bin"), b"x");

        let map = snapshot_config(dir.path(), &["BepInEx/config"]).unwrap();
        assert_eq!(map.len(), 1);
        assert_eq!(map[&path("BepInEx/config/a.cfg")], hash(b"data"));
    }

    #[test]
    fn replacement_preserves_original_when_staged_content_fails_verification() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("a.cfg");
        fs::write(&target, b"original").unwrap();

        let file = vfile(b"new");
        let staged = stage_validated(&target, &file).unwrap();
        fs::write(&staged, b"corrupt").unwrap();

        assert!(commit_staged(&staged, &target, &file.hash).is_err());
        assert_eq!(fs::read(&target).unwrap(), b"original");
        assert!(!staged.exists());
    }

    #[test]
    fn replacement_rejects_mismatched_hash_without_touching_original() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("a.cfg");
        fs::write(&target, b"original").unwrap();

        let file = archive::ValidatedConfigFile {
            hash: hash(b"other"),
            bytes: b"new".to_vec(),
        };

        assert!(write_validated(&target, &file).is_err());
        assert_eq!(fs::read(&target).unwrap(), b"original");
        assert!(!fs::read_dir(dir.path()).unwrap().any(|entry| {
            entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .ends_with(".gale-sync-tmp")
        }));
    }

    #[test]
    fn replacement_succeeds_and_leaves_no_staging_file() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("a.cfg");
        fs::write(&target, b"original").unwrap();

        write_validated(&target, &vfile(b"new")).unwrap();
        assert_eq!(fs::read(&target).unwrap(), b"new");
        assert!(!fs::read_dir(dir.path()).unwrap().any(|entry| {
            entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .ends_with(".gale-sync-tmp")
        }));
    }

    #[test]
    fn staging_never_overwrites_existing_file_at_candidate_path() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("a.cfg");
        fs::write(&target, b"original").unwrap();
        let taken = dir.path().join("a.cfg.taken.gale-sync-tmp");
        fs::write(&taken, b"sentinel").unwrap();

        let (staged, mut handle) = create_staging_file(
            &target,
            [
                "a.cfg.taken.gale-sync-tmp".to_string(),
                "a.cfg.fresh.gale-sync-tmp".to_string(),
            ]
            .into_iter(),
        )
        .unwrap();
        assert_eq!(
            staged.file_name().unwrap().to_string_lossy(),
            "a.cfg.fresh.gale-sync-tmp"
        );

        handle.write_all(b"new").unwrap();
        drop(handle);

        commit_staged(&staged, &target, &hash(b"new")).unwrap();
        assert_eq!(fs::read(&target).unwrap(), b"new");
        assert_eq!(fs::read(&taken).unwrap(), b"sentinel");
        assert!(!staged.exists());
    }

    #[test]
    fn staging_fails_when_all_candidates_are_taken() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("a.cfg");
        fs::write(&target, b"original").unwrap();
        let taken = dir.path().join("a.cfg.taken.gale-sync-tmp");
        fs::write(&taken, b"sentinel").unwrap();

        assert!(
            create_staging_file(
                &target,
                ["a.cfg.taken.gale-sync-tmp".to_string()].into_iter()
            )
            .is_err()
        );
        assert_eq!(fs::read(&taken).unwrap(), b"sentinel");
        assert_eq!(fs::read(&target).unwrap(), b"original");
    }
}
