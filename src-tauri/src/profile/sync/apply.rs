use std::{
    collections::{BTreeMap, HashSet},
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use eyre::{Context, OptionExt, Result, bail, ensure};
use serde::Serialize;

use super::{AppliedState, ConfigUpdatePolicy, PendingConfigReason, archive};
use crate::profile::export::{self, ConfigPath, ContentHash};

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConfigReviewItem {
    pub path: ConfigPath,
    pub reason: PendingConfigReason,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConfigPolicyEntry {
    pub path: ConfigPath,
    pub policy: ConfigUpdatePolicy,
}

#[derive(Debug, Serialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConfigReviewState {
    pub pending: Vec<ConfigReviewItem>,
    pub declined: Vec<ConfigReviewItem>,
    pub policies: Vec<ConfigPolicyEntry>,
}

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ConfigApplyReport {
    pub installed: Vec<ConfigPath>,
    pub pending: Vec<ConfigReviewItem>,
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
    record.policy_set_at = None;
    state.pending.remove(path);
    state.declined.remove(path);
}

pub(super) fn apply_available_config(
    profile_dir: &Path,
    config: &BTreeMap<ConfigPath, archive::ValidatedConfigFile>,
    state: &mut AppliedState,
) -> Result<ConfigApplyReport> {
    state.pending.retain(|path, _| config.contains_key(path));
    state.declined.retain(|path, _| config.contains_key(path));

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

        let prev = state.config.get(path);
        let prev_applied = prev.and_then(|record| record.applied.as_ref());
        let prev_written = prev.and_then(|record| record.written.as_ref());
        let prev_declined = prev.and_then(|record| record.declined.as_ref());
        let policy = prev.map(|record| record.policy).unwrap_or_default();
        let policy = if prev.and_then(|record| record.policy_set_at.as_ref()) == Some(&hash) {
            ConfigUpdatePolicy::Ask
        } else {
            policy
        };

        if local.as_ref() == Some(&hash) {
            record_applied(state, path, hash);
            continue;
        }

        if prev_applied == Some(&hash) {
            state.pending.remove(path);
            state.declined.remove(path);
            continue;
        }

        if prev_declined == Some(&hash) {
            state.pending.remove(path);
            let reason = if local.is_none() {
                PendingConfigReason::DeletedLocally
            } else {
                PendingConfigReason::ModifiedLocally
            };
            state.declined.insert(path.clone(), reason);
            continue;
        }

        state.declined.remove(path);

        match local {
            None if prev.is_none() => {
                write_validated(&target, file)?;
                record_applied(state, path, hash);
                report.installed.push(path.clone());
            }
            None => match policy {
                ConfigUpdatePolicy::AlwaysKeep => {
                    state.pending.remove(path);
                    let record = state.config.entry(path.clone()).or_default();
                    record.declined = Some(hash);
                    record.policy_set_at = None;
                    state
                        .declined
                        .insert(path.clone(), PendingConfigReason::DeletedLocally);
                }
                ConfigUpdatePolicy::Ask | ConfigUpdatePolicy::AlwaysApply => {
                    state
                        .pending
                        .insert(path.clone(), PendingConfigReason::DeletedLocally);
                }
            },
            Some(local_hash) => match policy {
                ConfigUpdatePolicy::AlwaysApply => {
                    write_validated(&target, file)?;
                    record_applied(state, path, hash);
                    report.installed.push(path.clone());
                }
                ConfigUpdatePolicy::AlwaysKeep => {
                    state.pending.remove(path);
                    let record = state.config.entry(path.clone()).or_default();
                    record.declined = Some(hash);
                    record.policy_set_at = None;
                    state
                        .declined
                        .insert(path.clone(), PendingConfigReason::ModifiedLocally);
                }
                ConfigUpdatePolicy::Ask => {
                    if prev_applied.is_none()
                        && prev_declined.is_none()
                        && prev_written == Some(&local_hash)
                    {
                        write_validated(&target, file)?;
                        record_applied(state, path, hash);
                        report.installed.push(path.clone());
                    } else {
                        state
                            .pending
                            .insert(path.clone(), PendingConfigReason::ModifiedLocally);
                    }
                }
            },
        }

        if state.pending.contains_key(path)
            && let Some(record) = state.config.get_mut(path)
        {
            record.declined = None;
        }
    }

    report.pending = review_items(state).pending;
    Ok(report)
}

pub(super) fn preserve_pending_policy_boundaries(state: &mut AppliedState) {
    let Some(latest) = state.latest.as_ref() else {
        return;
    };

    for path in state.pending.keys() {
        let Some(record) = state.config.get_mut(path) else {
            continue;
        };
        if record.policy == ConfigUpdatePolicy::Ask || record.policy_set_at.is_some() {
            continue;
        }
        if let Some(entry) = latest.config.get(path) {
            record.policy_set_at = Some(entry.hash.clone());
        }
    }
}

pub(super) fn review_items(state: &AppliedState) -> ConfigReviewState {
    let published = |path: &ConfigPath| {
        state
            .latest
            .as_ref()
            .is_some_and(|latest| latest.config.contains_key(path))
    };

    let item = |(path, reason): (&ConfigPath, &PendingConfigReason)| ConfigReviewItem {
        path: path.clone(),
        reason: *reason,
    };

    let legacy_declined = |path: &ConfigPath| {
        state
            .latest
            .as_ref()
            .and_then(|latest| latest.config.get(path))
            .map(|entry| &entry.hash)
            == state
                .config
                .get(path)
                .and_then(|record| record.declined.as_ref())
    };

    let pending = state
        .pending
        .iter()
        .filter(|(path, _)| published(path) && !legacy_declined(path))
        .map(item)
        .collect();

    let mut declined: Vec<ConfigReviewItem> = state
        .declined
        .iter()
        .filter(|(path, _)| published(path))
        .map(item)
        .collect();

    for (path, reason) in state
        .pending
        .iter()
        .filter(|(path, _)| published(path) && legacy_declined(path))
    {
        if !state.declined.contains_key(path) {
            declined.push(ConfigReviewItem {
                path: path.clone(),
                reason: *reason,
            });
        }
    }

    declined.sort_by(|a, b| a.path.cmp(&b.path));

    let policies = state
        .latest
        .as_ref()
        .map(|latest| {
            latest
                .config
                .keys()
                .map(|path| ConfigPolicyEntry {
                    path: path.clone(),
                    policy: state
                        .config
                        .get(path)
                        .map(|record| record.policy)
                        .unwrap_or_default(),
                })
                .collect()
        })
        .unwrap_or_default();

    ConfigReviewState {
        pending,
        declined,
        policies,
    }
}

fn local_review_reason(profile_dir: &Path, path: &ConfigPath) -> Result<PendingConfigReason> {
    let target = checked_target(profile_dir, path)?;
    match fs::symlink_metadata(&target) {
        Ok(metadata) if metadata.is_file() => Ok(PendingConfigReason::ModifiedLocally),
        Ok(_) => bail!(
            "synced config path is not a regular file: {}",
            target.display()
        ),
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            Ok(PendingConfigReason::DeletedLocally)
        }
        Err(err) => {
            Err(err).with_context(|| format!("failed to inspect config path: {}", target.display()))
        }
    }
}

pub(super) fn current_review_items(
    profile_dir: &Path,
    state: &AppliedState,
) -> Result<ConfigReviewState> {
    let mut review = review_items(state);

    for item in review.pending.iter_mut().chain(review.declined.iter_mut()) {
        item.reason = local_review_reason(profile_dir, &item.path)?;
    }

    Ok(review)
}

pub(super) fn apply_selected(
    profile_dir: &Path,
    config: &BTreeMap<ConfigPath, archive::ValidatedConfigFile>,
    state: &mut AppliedState,
    files: &[ConfigPath],
    restore_deleted: &[ConfigPath],
    remember: bool,
) -> Result<Vec<ConfigPath>> {
    let mut seen = HashSet::new();
    for path in files {
        ensure!(
            seen.insert(path.clone()),
            "duplicate selected config path: {path}"
        );
    }

    let mut restored = HashSet::new();
    for path in restore_deleted {
        ensure!(
            restored.insert(path.clone()),
            "duplicate restore entry: {path}"
        );
        ensure!(
            seen.contains(path),
            "restore entry was not selected: {path}"
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

    let mut targets = Vec::with_capacity(files.len());
    for path in files {
        let reason = local_review_reason(profile_dir, path)?;
        ensure!(
            reason != PendingConfigReason::DeletedLocally || restored.contains(path),
            "config file was deleted locally; confirm restoring it: {path}"
        );
        targets.push(checked_target(profile_dir, path)?);
    }

    let mut written = Vec::with_capacity(files.len());
    for (path, target) in files.iter().zip(targets) {
        let file = &config[path];
        let hash = file.hash.clone();

        write_validated(&target, file)?;
        record_applied(state, path, hash);
        if remember {
            state.config.entry(path.clone()).or_default().policy = ConfigUpdatePolicy::AlwaysApply;
        }
        written.push(path.clone());
    }

    Ok(written)
}

pub(super) fn decline_selected(
    state: &mut AppliedState,
    files: &[ConfigPath],
    remember: bool,
) -> Result<()> {
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
            let reason = *state
                .pending
                .get(path)
                .ok_or_eyre("config file is not pending review: {path}")?;
            entries.push((path.clone(), advertised.hash.clone(), reason));
        }
    }

    for (path, hash, reason) in entries {
        state.pending.remove(&path);
        state.declined.insert(path.clone(), reason);

        let record = state.config.entry(path).or_default();
        record.declined = Some(hash);
        if remember {
            record.policy = ConfigUpdatePolicy::AlwaysKeep;
            record.policy_set_at = None;
        }
    }

    Ok(())
}

pub(super) fn set_policy(
    state: &mut AppliedState,
    path: &ConfigPath,
    policy: ConfigUpdatePolicy,
) -> Result<()> {
    let current = state
        .latest
        .as_ref()
        .and_then(|latest| latest.config.get(path))
        .map(|entry| entry.hash.clone())
        .ok_or_eyre(format!("config file is not published: {path}"))?;

    let record = state.config.entry(path.clone()).or_default();
    record.policy = policy;
    record.policy_set_at = match policy {
        ConfigUpdatePolicy::Ask => None,
        ConfigUpdatePolicy::AlwaysApply | ConfigUpdatePolicy::AlwaysKeep => Some(current),
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;
    use crate::profile::{
        export::{ModRevision, SyncFileEntry, SyncManifest},
        sync::{AppliedFile, ConfigUpdatePolicy},
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
        policy: ConfigUpdatePolicy,
    ) -> AppliedFile {
        AppliedFile {
            applied,
            written,
            declined,
            policy,
            policy_set_at: None,
        }
    }

    fn receive(
        dir: &Path,
        state: &mut AppliedState,
        entries: &[(&str, &[u8])],
    ) -> ConfigApplyReport {
        preserve_pending_policy_boundaries(state);
        state.latest = Some(latest(entries));
        let config = archive_map(entries);
        apply_available_config(dir, &config, state).unwrap()
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
        state.config.insert(
            p.clone(),
            applied_file(None, None, Some(hash(b"remote")), ConfigUpdatePolicy::Ask),
        );
        state
            .pending
            .insert(p.clone(), PendingConfigReason::ModifiedLocally);
        state.latest = Some(latest(&[("a.cfg", b"remote")]));

        let config = archive_map(&[("a.cfg", b"remote")]);
        let report = apply_available_config(dir.path(), &config, &mut state).unwrap();

        assert!(report.installed.is_empty());
        assert_eq!(fs::read(dir.path().join(p.as_path())).unwrap(), b"local");
        assert!(!state.pending.contains_key(&p));
        assert_eq!(state.declined[&p], PendingConfigReason::ModifiedLocally);

        let review = review_items(&state);
        assert!(review.pending.is_empty());
        assert_eq!(review.declined.len(), 1);
        assert_eq!(
            review.declined[0].reason,
            PendingConfigReason::ModifiedLocally
        );
    }

    #[test]
    fn player_edit_after_applied_revision_skips() {
        let dir = tempdir().unwrap();
        let p = path("a.cfg");
        write(dir.path(), &p, b"edited");

        let mut state = AppliedState::default();
        state.config.insert(
            p.clone(),
            applied_file(
                Some(hash(b"remote")),
                Some(hash(b"remote")),
                None,
                ConfigUpdatePolicy::Ask,
            ),
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
            applied_file(
                Some(hash(b"old")),
                Some(hash(b"old")),
                None,
                ConfigUpdatePolicy::Ask,
            ),
        );

        let config = archive_map(&[("a.cfg", b"remote")]);
        let report = apply_available_config(dir.path(), &config, &mut state).unwrap();

        assert!(report.installed.is_empty());
        assert!(!dir.path().join("a.cfg").exists());
        assert_eq!(state.pending[&p], PendingConfigReason::DeletedLocally);
    }

    #[test]
    fn installer_default_accepts_initial_baseline() {
        let dir = tempdir().unwrap();
        let p = path("a.cfg");
        write(dir.path(), &p, b"package-default");

        let mut state = AppliedState::default();
        state.config.insert(
            p.clone(),
            applied_file(
                None,
                Some(hash(b"package-default")),
                None,
                ConfigUpdatePolicy::Ask,
            ),
        );

        let report = receive(dir.path(), &mut state, &[("a.cfg", b"remote")]);

        assert_eq!(report.installed, vec![p.clone()]);
        assert_eq!(fs::read(dir.path().join(p.as_path())).unwrap(), b"remote");
        let record = &state.config[&p];
        assert_eq!(record.applied, Some(hash(b"remote")));
        assert_eq!(record.written, Some(hash(b"remote")));
        assert!(state.pending.is_empty());
        assert!(state.declined.is_empty());
    }

    #[test]
    fn ask_policy_prompts_after_previously_applied_untouched_file() {
        let dir = tempdir().unwrap();
        let p = path("a.cfg");
        write(dir.path(), &p, b"v1");

        let mut state = AppliedState::default();
        state.config.insert(
            p.clone(),
            applied_file(
                Some(hash(b"v1")),
                Some(hash(b"v1")),
                None,
                ConfigUpdatePolicy::Ask,
            ),
        );

        let report = receive(dir.path(), &mut state, &[("a.cfg", b"v2")]);

        assert!(report.installed.is_empty());
        assert_eq!(fs::read(dir.path().join(p.as_path())).unwrap(), b"v1");
        assert_eq!(state.pending[&p], PendingConfigReason::ModifiedLocally);
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
            applied_file(Some(hash(b"applied")), None, None, ConfigUpdatePolicy::Ask),
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
        state.config.insert(
            p.clone(),
            applied_file(None, None, Some(hash(b"remote")), ConfigUpdatePolicy::Ask),
        );
        state
            .pending
            .insert(p.clone(), PendingConfigReason::ModifiedLocally);

        let config = archive_map(&[("a.cfg", b"remote"), ("b.cfg", b"bee")]);

        assert!(
            apply_selected(
                dir.path(),
                &config,
                &mut state,
                &[p.clone(), p.clone()],
                &[],
                false
            )
            .is_err()
        );
        assert!(
            apply_selected(
                dir.path(),
                &config,
                &mut state,
                &[path("nope.cfg")],
                &[],
                false
            )
            .is_err()
        );

        let mut bad = vfile(b"remote");
        bad.hash = hash(b"other");
        let bad_config = BTreeMap::from([(p.clone(), bad), (path("b.cfg"), vfile(b"bee"))]);
        assert!(
            apply_selected(
                dir.path(),
                &bad_config,
                &mut state,
                &[p.clone()],
                &[],
                false
            )
            .is_err()
        );

        let partial = archive_map(&[("a.cfg", b"remote")]);
        assert!(
            apply_selected(
                dir.path(),
                &partial,
                &mut state,
                &[path("b.cfg")],
                &[],
                false
            )
            .is_err()
        );

        let mut no_latest = AppliedState::default();
        assert!(
            apply_selected(
                dir.path(),
                &config,
                &mut no_latest,
                &[p.clone()],
                &[],
                false
            )
            .is_err()
        );

        let written =
            apply_selected(dir.path(), &config, &mut state, &[p.clone()], &[], false).unwrap();
        assert_eq!(written, vec![p.clone()]);
        assert_eq!(fs::read(dir.path().join(p.as_path())).unwrap(), b"remote");
        let record = &state.config[&p];
        assert_eq!(record.applied, Some(hash(b"remote")));
        assert_eq!(record.written, Some(hash(b"remote")));
        assert!(record.declined.is_none());
        assert_eq!(record.policy, ConfigUpdatePolicy::Ask);
        assert!(!state.pending.contains_key(&p));
        assert!(state.declined.is_empty());
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

        assert!(decline_selected(&mut state, &[p.clone(), p.clone()], false).is_err());
        assert!(decline_selected(&mut state, &[q.clone()], false).is_err());
        assert!(decline_selected(&mut state, &[path("nope.cfg")], false).is_err());

        let mut no_latest = AppliedState::default();
        no_latest
            .pending
            .insert(p.clone(), PendingConfigReason::ModifiedLocally);
        assert!(decline_selected(&mut no_latest, &[p.clone()], false).is_err());

        decline_selected(&mut state, &[p.clone()], false).unwrap();
        assert_eq!(state.config[&p].declined, Some(hash(b"remote")));
        assert_eq!(state.config[&p].policy, ConfigUpdatePolicy::Ask);
        assert!(!state.pending.contains_key(&p));
        assert_eq!(state.declined[&p], PendingConfigReason::ModifiedLocally);

        let review = review_items(&state);
        assert!(review.pending.is_empty());
        assert_eq!(review.declined.len(), 1);
        assert_eq!(
            review.declined[0].reason,
            PendingConfigReason::ModifiedLocally
        );
    }

    #[test]
    fn new_revision_re_prompts_after_decline() {
        let dir = tempdir().unwrap();
        let p = path("a.cfg");
        write(dir.path(), &p, b"local");

        let mut state = AppliedState::default();
        state.latest = Some(latest(&[("a.cfg", b"v1")]));
        state.config.insert(
            p.clone(),
            applied_file(None, None, Some(hash(b"v1")), ConfigUpdatePolicy::Ask),
        );
        state
            .pending
            .insert(p.clone(), PendingConfigReason::ModifiedLocally);
        let review = review_items(&state);
        assert_eq!(review.pending.len(), 0);
        assert_eq!(review.declined.len(), 1);

        state.latest = Some(latest(&[("a.cfg", b"v2")]));
        let config = archive_map(&[("a.cfg", b"v2")]);
        apply_available_config(dir.path(), &config, &mut state).unwrap();

        assert_eq!(fs::read(dir.path().join(p.as_path())).unwrap(), b"local");
        assert_eq!(state.pending[&p], PendingConfigReason::ModifiedLocally);
        assert!(state.declined.is_empty());
        assert!(state.config[&p].declined.is_none());
        let review = review_items(&state);
        assert_eq!(review.pending.len(), 1);
        assert!(review.declined.is_empty());
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

    fn read_file(dir: &Path, p: &ConfigPath) -> Vec<u8> {
        fs::read(dir.join(p.as_path())).unwrap()
    }

    fn round_trip(state: &AppliedState) -> AppliedState {
        let json = serde_json::to_string(state).unwrap();
        serde_json::from_str(&json).unwrap()
    }

    #[test]
    fn ask_policy_prompts_after_accepting_a_then_b_and_c() {
        let dir = tempdir().unwrap();
        let p = path("a.cfg");
        write(dir.path(), &p, b"custom");

        let mut state = AppliedState::default();
        state.config.insert(
            p.clone(),
            applied_file(None, None, None, ConfigUpdatePolicy::Ask),
        );

        let report = receive(dir.path(), &mut state, &[("a.cfg", b"A")]);
        assert_eq!(report.pending.len(), 1);
        assert_eq!(state.pending[&p], PendingConfigReason::ModifiedLocally);
        assert_eq!(read_file(dir.path(), &p), b"custom");
        assert_eq!(state.latest.as_ref().unwrap().config[&p].hash, hash(b"A"));

        let config = archive_map(&[("a.cfg", b"A")]);
        apply_selected(dir.path(), &config, &mut state, &[p.clone()], &[], false).unwrap();
        assert_eq!(read_file(dir.path(), &p), b"A");

        let mut state = round_trip(&state);

        let report = receive(dir.path(), &mut state, &[("a.cfg", b"B")]);
        assert_eq!(state.latest.as_ref().unwrap().config[&p].hash, hash(b"B"));
        assert!(report.installed.is_empty());
        assert_eq!(read_file(dir.path(), &p), b"A");
        assert_eq!(state.pending[&p], PendingConfigReason::ModifiedLocally);
        assert!(state.declined.is_empty());
        assert_eq!(state.config[&p].policy, ConfigUpdatePolicy::Ask);

        let review = review_items(&state);
        assert_eq!(review.pending.len(), 1);
        assert!(review.declined.is_empty());

        let config = archive_map(&[("a.cfg", b"B")]);
        apply_selected(dir.path(), &config, &mut state, &[p.clone()], &[], false).unwrap();
        assert_eq!(read_file(dir.path(), &p), b"B");

        let report = receive(dir.path(), &mut state, &[("a.cfg", b"C")]);
        assert_eq!(state.latest.as_ref().unwrap().config[&p].hash, hash(b"C"));
        assert!(report.installed.is_empty());
        assert_eq!(read_file(dir.path(), &p), b"B");
        assert_eq!(state.pending[&p], PendingConfigReason::ModifiedLocally);
    }

    #[test]
    fn ask_policy_prompts_after_declining_a_then_b_and_c() {
        let dir = tempdir().unwrap();
        let p = path("a.cfg");
        write(dir.path(), &p, b"custom");

        let mut state = AppliedState::default();
        state.config.insert(
            p.clone(),
            applied_file(None, None, None, ConfigUpdatePolicy::Ask),
        );

        receive(dir.path(), &mut state, &[("a.cfg", b"A")]);
        assert_eq!(state.pending[&p], PendingConfigReason::ModifiedLocally);

        decline_selected(&mut state, &[p.clone()], false).unwrap();
        assert!(state.pending.is_empty());
        assert_eq!(state.declined[&p], PendingConfigReason::ModifiedLocally);
        assert_eq!(state.config[&p].declined, Some(hash(b"A")));
        assert_eq!(state.config[&p].policy, ConfigUpdatePolicy::Ask);

        let review = review_items(&state);
        assert_eq!(review.pending.len(), 0);
        assert_eq!(review.declined.len(), 1);

        let mut state = round_trip(&state);

        let report = receive(dir.path(), &mut state, &[("a.cfg", b"B")]);
        assert_eq!(state.latest.as_ref().unwrap().config[&p].hash, hash(b"B"));
        assert!(report.installed.is_empty());
        assert_eq!(read_file(dir.path(), &p), b"custom");
        assert_eq!(state.pending[&p], PendingConfigReason::ModifiedLocally);
        assert!(state.declined.is_empty());

        let review = review_items(&state);
        assert_eq!(review.pending.len(), 1);
        assert_eq!(review.declined.len(), 0);

        decline_selected(&mut state, &[p.clone()], false).unwrap();
        assert_eq!(state.config[&p].declined, Some(hash(b"B")));
        assert_eq!(state.config[&p].policy, ConfigUpdatePolicy::Ask);

        let report = receive(dir.path(), &mut state, &[("a.cfg", b"C")]);
        assert_eq!(state.latest.as_ref().unwrap().config[&p].hash, hash(b"C"));
        assert!(report.installed.is_empty());
        assert_eq!(read_file(dir.path(), &p), b"custom");
        assert_eq!(state.pending[&p], PendingConfigReason::ModifiedLocally);
        assert!(state.declined.is_empty());
    }

    #[test]
    fn always_apply_persists_and_applies_b_and_c() {
        let dir = tempdir().unwrap();
        let p = path("a.cfg");
        write(dir.path(), &p, b"custom");

        let mut state = AppliedState::default();
        state.config.insert(
            p.clone(),
            applied_file(None, None, None, ConfigUpdatePolicy::Ask),
        );

        receive(dir.path(), &mut state, &[("a.cfg", b"A")]);
        assert_eq!(state.pending[&p], PendingConfigReason::ModifiedLocally);

        let config = archive_map(&[("a.cfg", b"A")]);
        apply_selected(dir.path(), &config, &mut state, &[p.clone()], &[], true).unwrap();
        assert_eq!(read_file(dir.path(), &p), b"A");
        assert_eq!(state.config[&p].policy, ConfigUpdatePolicy::AlwaysApply);

        let mut state = round_trip(&state);
        assert_eq!(state.config[&p].policy, ConfigUpdatePolicy::AlwaysApply);

        let report = receive(dir.path(), &mut state, &[("a.cfg", b"B")]);
        assert_eq!(report.installed, vec![p.clone()]);
        assert_eq!(read_file(dir.path(), &p), b"B");
        let review = review_items(&state);
        assert!(review.pending.is_empty());
        assert!(review.declined.is_empty());

        let report = receive(dir.path(), &mut state, &[("a.cfg", b"C")]);
        assert_eq!(report.installed, vec![p.clone()]);
        assert_eq!(read_file(dir.path(), &p), b"C");
        let review = review_items(&state);
        assert!(review.pending.is_empty());
        assert!(review.declined.is_empty());
    }

    #[test]
    fn always_keep_persists_and_keeps_b_and_c() {
        let dir = tempdir().unwrap();
        let p = path("a.cfg");
        write(dir.path(), &p, b"custom");

        let mut state = AppliedState::default();
        state.config.insert(
            p.clone(),
            applied_file(None, None, None, ConfigUpdatePolicy::Ask),
        );

        receive(dir.path(), &mut state, &[("a.cfg", b"A")]);
        decline_selected(&mut state, &[p.clone()], true).unwrap();
        assert_eq!(state.config[&p].policy, ConfigUpdatePolicy::AlwaysKeep);
        assert_eq!(state.config[&p].declined, Some(hash(b"A")));

        let mut state = round_trip(&state);
        assert_eq!(state.config[&p].policy, ConfigUpdatePolicy::AlwaysKeep);

        let report = receive(dir.path(), &mut state, &[("a.cfg", b"B")]);
        assert!(report.installed.is_empty());
        assert_eq!(read_file(dir.path(), &p), b"custom");
        assert!(state.pending.is_empty());
        assert_eq!(state.config[&p].declined, Some(hash(b"B")));
        let review = review_items(&state);
        assert!(review.pending.is_empty());
        assert_eq!(review.declined.len(), 1);
        assert_eq!(review.declined[0].path, p);

        let report = receive(dir.path(), &mut state, &[("a.cfg", b"C")]);
        assert!(report.installed.is_empty());
        assert_eq!(read_file(dir.path(), &p), b"custom");
        assert!(state.pending.is_empty());
        assert_eq!(state.config[&p].declined, Some(hash(b"C")));
        let review = review_items(&state);
        assert!(review.pending.is_empty());
        assert_eq!(review.declined.len(), 1);

        let config = archive_map(&[("a.cfg", b"C")]);
        apply_selected(dir.path(), &config, &mut state, &[p.clone()], &[], false).unwrap();
        assert_eq!(read_file(dir.path(), &p), b"C");
        assert!(state.declined.is_empty());
        assert_eq!(state.config[&p].policy, ConfigUpdatePolicy::AlwaysKeep);
    }

    #[test]
    fn always_apply_still_prompts_before_restoring_deleted_file() {
        let dir = tempdir().unwrap();
        let p = path("a.cfg");
        write(dir.path(), &p, b"A");

        let mut state = AppliedState::default();
        state.config.insert(
            p.clone(),
            applied_file(
                Some(hash(b"A")),
                Some(hash(b"A")),
                None,
                ConfigUpdatePolicy::AlwaysApply,
            ),
        );
        fs::remove_file(dir.path().join(p.as_path())).unwrap();

        let report = receive(dir.path(), &mut state, &[("a.cfg", b"B")]);
        assert_eq!(state.latest.as_ref().unwrap().config[&p].hash, hash(b"B"));
        assert!(report.installed.is_empty());
        assert!(!dir.path().join(p.as_path()).exists());
        assert_eq!(state.pending[&p], PendingConfigReason::DeletedLocally);
    }

    #[test]
    fn legacy_applied_state_defaults_to_ask_policy() {
        let p = path("a.cfg");
        let applied = hash(b"old");
        let json = format!(
            r#"{{"config":{{"a.cfg":{{"applied":"{}","written":null,"declined":null}}}},"pending":{{"a.cfg":"modifiedLocally"}}}}"#,
            applied.as_str()
        );

        let state: AppliedState = serde_json::from_str(&json).unwrap();
        assert!(state.declined.is_empty());
        assert_eq!(state.config[&p].policy, ConfigUpdatePolicy::Ask);
        assert_eq!(state.config[&p].policy_set_at, None);
        assert_eq!(state.config[&p].applied, Some(applied.clone()));
        assert_eq!(state.pending[&p], PendingConfigReason::ModifiedLocally);

        let state = round_trip(&state);
        assert_eq!(state.config[&p].policy, ConfigUpdatePolicy::Ask);
        assert_eq!(state.config[&p].applied, Some(applied));
        assert_eq!(state.pending[&p], PendingConfigReason::ModifiedLocally);
    }

    #[test]
    fn policy_can_be_changed_later_and_persists() {
        let p = path("a.cfg");

        let mut state = AppliedState::default();
        state.latest = Some(latest(&[("a.cfg", b"remote")]));

        set_policy(&mut state, &p, ConfigUpdatePolicy::AlwaysApply).unwrap();

        let mut state = round_trip(&state);
        assert_eq!(state.config[&p].policy, ConfigUpdatePolicy::AlwaysApply);

        set_policy(&mut state, &p, ConfigUpdatePolicy::Ask).unwrap();

        let mut state = round_trip(&state);
        assert_eq!(state.config[&p].policy, ConfigUpdatePolicy::Ask);

        assert!(
            set_policy(
                &mut state,
                &path("unpublished.cfg"),
                ConfigUpdatePolicy::AlwaysKeep
            )
            .is_err()
        );
    }

    #[test]
    fn legacy_declined_state_is_reported_only_as_declined_without_pull() {
        let remote = hash(b"A");
        let mods = ModRevision::from_hash(blake3::hash(b"rev"));

        let json = format!(
            r#"{{"latest":{{"version":1,"modsRevision":"{mods}","config":{{"a.cfg":{{"hash":"{remote}"}}}}}},"config":{{"a.cfg":{{"applied":null,"written":null,"declined":"{remote}"}}}},"pending":{{"a.cfg":"modifiedLocally"}}}}"#,
            mods = mods.as_str(),
            remote = remote.as_str(),
        );

        let state: AppliedState = serde_json::from_str(&json).unwrap();
        assert!(state.declined.is_empty());

        let review = review_items(&state);
        assert_eq!(review.pending.len(), 0);
        assert_eq!(
            review.declined,
            vec![ConfigReviewItem {
                path: path("a.cfg"),
                reason: PendingConfigReason::ModifiedLocally,
            }]
        );
        assert_eq!(state.config[&path("a.cfg")].policy, ConfigUpdatePolicy::Ask);
    }

    #[test]
    fn declined_reason_tracks_local_deletion_on_repeat_pull() {
        let dir = tempdir().unwrap();
        let p = path("a.cfg");
        write(dir.path(), &p, b"custom");

        let mut state = AppliedState::default();
        state.config.insert(
            p.clone(),
            applied_file(None, None, None, ConfigUpdatePolicy::Ask),
        );

        receive(dir.path(), &mut state, &[("a.cfg", b"A")]);
        assert_eq!(state.pending[&p], PendingConfigReason::ModifiedLocally);

        decline_selected(&mut state, &[p.clone()], false).unwrap();
        assert_eq!(state.declined[&p], PendingConfigReason::ModifiedLocally);

        fs::remove_file(dir.path().join(p.as_path())).unwrap();

        receive(dir.path(), &mut state, &[("a.cfg", b"A")]);

        assert_eq!(state.declined[&p], PendingConfigReason::DeletedLocally);
        let review = review_items(&state);
        assert_eq!(review.pending.len(), 0);
        assert_eq!(review.declined.len(), 1);
        assert_eq!(
            review.declined[0].reason,
            PendingConfigReason::DeletedLocally
        );
    }

    #[test]
    fn always_keep_new_revision_replaces_pending_with_declined() {
        let dir = tempdir().unwrap();
        let p = path("a.cfg");
        write(dir.path(), &p, b"custom");

        let mut state = AppliedState::default();
        state.config.insert(
            p.clone(),
            applied_file(None, None, None, ConfigUpdatePolicy::Ask),
        );

        receive(dir.path(), &mut state, &[("a.cfg", b"A")]);
        assert_eq!(state.pending[&p], PendingConfigReason::ModifiedLocally);

        set_policy(&mut state, &p, ConfigUpdatePolicy::AlwaysKeep).unwrap();

        let report = receive(dir.path(), &mut state, &[("a.cfg", b"B")]);
        assert_eq!(state.latest.as_ref().unwrap().config[&p].hash, hash(b"B"));
        assert!(report.installed.is_empty());
        assert_eq!(read_file(dir.path(), &p), b"custom");
        assert!(state.pending.is_empty());
        assert_eq!(state.declined[&p], PendingConfigReason::ModifiedLocally);
        assert_eq!(state.config[&p].declined, Some(hash(b"B")));

        let review = review_items(&state);
        assert_eq!(review.pending.len(), 0);
        assert_eq!(review.declined.len(), 1);
    }

    #[test]
    fn policy_screen_changes_start_after_current_revision() {
        let dir = tempdir().unwrap();
        let p = path("a.cfg");
        let q = path("b.cfg");
        write(dir.path(), &p, b"custom-a");
        write(dir.path(), &q, b"custom-b");

        let mut state = AppliedState::default();
        state.config.insert(
            p.clone(),
            applied_file(None, None, None, ConfigUpdatePolicy::Ask),
        );
        state.config.insert(
            q.clone(),
            applied_file(None, None, None, ConfigUpdatePolicy::Ask),
        );

        receive(dir.path(), &mut state, &[("a.cfg", b"A"), ("b.cfg", b"A")]);
        assert_eq!(state.pending[&p], PendingConfigReason::ModifiedLocally);
        assert_eq!(state.pending[&q], PendingConfigReason::ModifiedLocally);

        set_policy(&mut state, &p, ConfigUpdatePolicy::AlwaysApply).unwrap();
        set_policy(&mut state, &q, ConfigUpdatePolicy::AlwaysKeep).unwrap();

        let report = receive(dir.path(), &mut state, &[("a.cfg", b"A"), ("b.cfg", b"A")]);
        assert!(report.installed.is_empty());
        assert_eq!(read_file(dir.path(), &p), b"custom-a");
        assert_eq!(read_file(dir.path(), &q), b"custom-b");
        assert_eq!(state.pending[&p], PendingConfigReason::ModifiedLocally);
        assert_eq!(state.pending[&q], PendingConfigReason::ModifiedLocally);
        assert!(state.declined.is_empty());

        let report = receive(dir.path(), &mut state, &[("a.cfg", b"B"), ("b.cfg", b"B")]);
        assert_eq!(report.installed, vec![p.clone()]);
        assert_eq!(state.latest.as_ref().unwrap().config[&p].hash, hash(b"B"));
        assert_eq!(state.latest.as_ref().unwrap().config[&q].hash, hash(b"B"));
        assert_eq!(read_file(dir.path(), &p), b"B");
        assert_eq!(read_file(dir.path(), &q), b"custom-b");
        assert!(state.pending.is_empty());
        assert_eq!(state.declined[&q], PendingConfigReason::ModifiedLocally);
        assert_eq!(state.config[&q].declined, Some(hash(b"B")));

        let review = review_items(&state);
        assert_eq!(review.pending.len(), 0);
        assert_eq!(review.declined.len(), 1);
        assert_eq!(review.declined[0].path, q);
    }

    #[test]
    fn current_review_items_reflects_filesystem_changes_without_pull() {
        let dir = tempdir().unwrap();
        let p = path("a.cfg");
        write(dir.path(), &p, b"custom");

        let mut state = AppliedState::default();
        state.config.insert(
            p.clone(),
            applied_file(None, None, None, ConfigUpdatePolicy::Ask),
        );

        receive(dir.path(), &mut state, &[("a.cfg", b"A")]);
        decline_selected(&mut state, &[p.clone()], false).unwrap();

        let review = current_review_items(dir.path(), &state).unwrap();
        assert_eq!(review.declined.len(), 1);
        assert_eq!(
            review.declined[0].reason,
            PendingConfigReason::ModifiedLocally
        );

        fs::remove_file(dir.path().join(p.as_path())).unwrap();

        let review = current_review_items(dir.path(), &state).unwrap();
        assert_eq!(review.declined.len(), 1);
        assert_eq!(
            review.declined[0].reason,
            PendingConfigReason::DeletedLocally
        );
    }

    #[test]
    fn apply_selected_requires_exact_deleted_file_confirmation() {
        let dir = tempdir().unwrap();
        let p = path("a.cfg");
        let q = path("b.cfg");
        write(dir.path(), &q, b"local-b");

        let mut state = AppliedState::default();
        state.latest = Some(latest(&[("a.cfg", b"A"), ("b.cfg", b"B")]));
        state
            .pending
            .insert(p.clone(), PendingConfigReason::DeletedLocally);
        state
            .pending
            .insert(q.clone(), PendingConfigReason::ModifiedLocally);

        let config = archive_map(&[("a.cfg", b"A"), ("b.cfg", b"B")]);

        assert!(apply_selected(dir.path(), &config, &mut state, &[p.clone()], &[], false).is_err());
        assert!(!dir.path().join(p.as_path()).exists());

        assert!(
            apply_selected(
                dir.path(),
                &config,
                &mut state,
                &[q.clone()],
                &[p.clone()],
                false
            )
            .is_err()
        );
        assert_eq!(read_file(dir.path(), &q), b"local-b");

        let written = apply_selected(
            dir.path(),
            &config,
            &mut state,
            &[p.clone()],
            &[p.clone()],
            false,
        )
        .unwrap();
        assert_eq!(written, vec![p.clone()]);
        assert_eq!(read_file(dir.path(), &p), b"A");
    }

    #[test]
    fn same_applied_revision_does_not_restore_deleted_file() {
        let dir = tempdir().unwrap();
        let p = path("a.cfg");
        write(dir.path(), &p, b"A");

        let mut state = AppliedState::default();
        state.config.insert(
            p.clone(),
            applied_file(
                Some(hash(b"A")),
                Some(hash(b"A")),
                None,
                ConfigUpdatePolicy::Ask,
            ),
        );

        fs::remove_file(dir.path().join(p.as_path())).unwrap();

        let report = receive(dir.path(), &mut state, &[("a.cfg", b"A")]);
        assert!(report.installed.is_empty());
        assert!(!dir.path().join(p.as_path()).exists());
        assert!(state.pending.is_empty());
        assert!(state.declined.is_empty());
    }

    #[test]
    fn declined_to_always_apply_keeps_current_and_applies_next() {
        let dir = tempdir().unwrap();
        let p = path("a.cfg");
        write(dir.path(), &p, b"custom");

        let mut state = AppliedState::default();
        state.config.insert(
            p.clone(),
            applied_file(None, None, None, ConfigUpdatePolicy::Ask),
        );

        receive(dir.path(), &mut state, &[("a.cfg", b"A")]);
        decline_selected(&mut state, &[p.clone()], false).unwrap();
        assert_eq!(state.declined[&p], PendingConfigReason::ModifiedLocally);

        set_policy(&mut state, &p, ConfigUpdatePolicy::AlwaysApply).unwrap();

        receive(dir.path(), &mut state, &[("a.cfg", b"A")]);
        assert_eq!(read_file(dir.path(), &p), b"custom");
        assert!(state.pending.is_empty());
        assert_eq!(state.declined[&p], PendingConfigReason::ModifiedLocally);

        let report = receive(dir.path(), &mut state, &[("a.cfg", b"B")]);
        assert_eq!(report.installed, vec![p.clone()]);
        assert_eq!(read_file(dir.path(), &p), b"B");
        assert!(state.pending.is_empty());
        assert!(state.declined.is_empty());
    }

    #[test]
    fn declined_to_ask_keeps_current_and_prompts_on_next() {
        let dir = tempdir().unwrap();
        let p = path("a.cfg");
        write(dir.path(), &p, b"custom");

        let mut state = AppliedState::default();
        state.config.insert(
            p.clone(),
            applied_file(None, None, None, ConfigUpdatePolicy::Ask),
        );

        receive(dir.path(), &mut state, &[("a.cfg", b"A")]);
        decline_selected(&mut state, &[p.clone()], false).unwrap();
        assert_eq!(state.declined[&p], PendingConfigReason::ModifiedLocally);

        set_policy(&mut state, &p, ConfigUpdatePolicy::Ask).unwrap();

        receive(dir.path(), &mut state, &[("a.cfg", b"A")]);
        assert_eq!(read_file(dir.path(), &p), b"custom");
        assert!(state.pending.is_empty());
        assert_eq!(state.declined[&p], PendingConfigReason::ModifiedLocally);

        let report = receive(dir.path(), &mut state, &[("a.cfg", b"B")]);
        assert!(report.installed.is_empty());
        assert_eq!(read_file(dir.path(), &p), b"custom");
        assert_eq!(state.pending[&p], PendingConfigReason::ModifiedLocally);
        assert!(state.declined.is_empty());
    }

    #[test]
    fn policy_set_at_round_trips_and_defaults_to_none() {
        let p = path("a.cfg");

        let mut state = AppliedState::default();
        state.latest = Some(latest(&[("a.cfg", b"A")]));

        set_policy(&mut state, &p, ConfigUpdatePolicy::AlwaysApply).unwrap();
        assert_eq!(state.config[&p].policy_set_at, Some(hash(b"A")));

        let mut state = round_trip(&state);
        assert_eq!(state.config[&p].policy_set_at, Some(hash(b"A")));

        set_policy(&mut state, &p, ConfigUpdatePolicy::Ask).unwrap();
        assert_eq!(state.config[&p].policy_set_at, None);

        let state = round_trip(&state);
        assert_eq!(state.config[&p].policy_set_at, None);
    }

    #[test]
    fn legacy_pending_automatic_policies_start_after_current_revision() {
        let dir = tempdir().unwrap();
        let p = path("a.cfg");
        let q = path("b.cfg");
        write(dir.path(), &p, b"custom-a");
        write(dir.path(), &q, b"custom-b");

        let mut state = AppliedState::default();
        state.latest = Some(latest(&[("a.cfg", b"A"), ("b.cfg", b"A")]));
        state.config.insert(
            p.clone(),
            applied_file(None, None, None, ConfigUpdatePolicy::AlwaysApply),
        );
        state.config.insert(
            q.clone(),
            applied_file(None, None, None, ConfigUpdatePolicy::AlwaysKeep),
        );
        state
            .pending
            .insert(p.clone(), PendingConfigReason::ModifiedLocally);
        state
            .pending
            .insert(q.clone(), PendingConfigReason::ModifiedLocally);
        assert_eq!(state.config[&p].policy_set_at, None);
        assert_eq!(state.config[&q].policy_set_at, None);

        let mut state = round_trip(&state);
        assert_eq!(state.config[&p].policy_set_at, None);
        assert_eq!(state.config[&q].policy_set_at, None);

        let report = receive(dir.path(), &mut state, &[("a.cfg", b"A"), ("b.cfg", b"A")]);
        assert!(report.installed.is_empty());
        assert_eq!(read_file(dir.path(), &p), b"custom-a");
        assert_eq!(read_file(dir.path(), &q), b"custom-b");
        assert_eq!(state.pending[&p], PendingConfigReason::ModifiedLocally);
        assert_eq!(state.pending[&q], PendingConfigReason::ModifiedLocally);
        assert!(state.declined.is_empty());

        let report = receive(dir.path(), &mut state, &[("a.cfg", b"B"), ("b.cfg", b"B")]);
        assert_eq!(report.installed, vec![p.clone()]);
        assert_eq!(read_file(dir.path(), &p), b"B");
        assert_eq!(read_file(dir.path(), &q), b"custom-b");
        assert!(state.pending.is_empty());
        assert_eq!(state.declined[&q], PendingConfigReason::ModifiedLocally);
        assert_eq!(state.config[&q].declined, Some(hash(b"B")));
    }
}
