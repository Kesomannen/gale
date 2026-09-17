use std::{
    collections::{BTreeMap, HashSet},
    fs,
    io::Cursor,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};
use eyre::{Context, OptionExt, Result, bail, ensure};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use super::{
    PublishedState, SyncProfileData, archive, auth, download_profile_bytes, get_profile_meta,
    upload_profile_file,
};
use crate::{
    profile::export::{
        self, ConfigPath, ContentHash, ProfileManifest, SyncFileEntry, SyncManifest,
        manifest_revision,
    },
    state::ManagerExt,
};

const SNAPSHOT_DIR: &str = "_state/sync";
const STAGING_DIR: &str = "_state/sync-staging";

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PublishMode {
    Mods,
    Config { files: Vec<ConfigPath> },
    Both { files: Vec<ConfigPath> },
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncConfigFileInfo {
    pub path: ConfigPath,
    pub size: u64,
    pub status: SyncConfigFileStatus,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SyncConfigFileStatus {
    New,
    Modified,
    Published,
}

fn snapshot_dir(profile_dir: &Path) -> PathBuf {
    profile_dir.join(SNAPSHOT_DIR)
}

fn staging_dir(profile_dir: &Path) -> PathBuf {
    profile_dir.join(STAGING_DIR)
}

fn published_state(archive: &archive::ValidatedSyncArchive) -> Result<PublishedState> {
    let mut manifest = archive.manifest.clone();
    let mods_revision = match &archive.format {
        archive::SyncArchiveFormat::Selective(sync) => sync.mods_revision.clone(),
        archive::SyncArchiveFormat::Legacy => manifest_revision(&manifest)?,
    };
    manifest.sync = None;

    let config = archive
        .config
        .iter()
        .map(|(path, file)| (path.clone(), file.hash.clone()))
        .collect();

    Ok(PublishedState {
        manifest,
        mods_revision,
        config,
    })
}

fn snapshot_consistent(published: &PublishedState, profile_dir: &Path) -> Result<bool> {
    if manifest_revision(&published.manifest)? != published.mods_revision {
        return Ok(false);
    }

    for (path, hash) in &published.config {
        let file = snapshot_dir(profile_dir).join(path.as_path());
        if !file.is_file() {
            return Ok(false);
        }

        let bytes = fs::read(&file)
            .with_context(|| format!("failed to read snapshot file: {}", file.display()))?;
        if ContentHash::from_hash(blake3::hash(&bytes)) != *hash {
            return Ok(false);
        }
    }

    Ok(true)
}

fn needs_reconcile(
    data: &SyncProfileData,
    remote_updated_at: DateTime<Utc>,
    profile_dir: &Path,
) -> Result<bool> {
    let Some(published) = &data.published else {
        return Ok(true);
    };

    if remote_updated_at != data.synced_at {
        return Ok(true);
    }

    Ok(!snapshot_consistent(published, profile_dir)?)
}

pub(super) fn adopt_publication(
    profile_dir: &Path,
    validated: &archive::ValidatedSyncArchive,
) -> Result<PublishedState> {
    let state = published_state(validated)?;
    let config: BTreeMap<_, _> = validated
        .config
        .iter()
        .map(|(path, file)| (path.clone(), file.bytes.clone()))
        .collect();

    stage_snapshot(&config, profile_dir)?;
    promote_snapshot(profile_dir)?;

    Ok(state)
}

fn stage_snapshot(config: &BTreeMap<ConfigPath, Vec<u8>>, profile_dir: &Path) -> Result<()> {
    let staging = staging_dir(profile_dir);

    if staging.exists() {
        fs::remove_dir_all(&staging).with_context(|| {
            format!("failed to remove stale staging dir: {}", staging.display())
        })?;
    }

    fs::create_dir_all(&staging)
        .with_context(|| format!("failed to create staging dir: {}", staging.display()))?;

    for (path, bytes) in config {
        let target = staging.join(path.as_path());
        fs::create_dir_all(target.parent().unwrap())
            .with_context(|| format!("failed to create staging dir: {}", target.display()))?;
        fs::write(&target, bytes)
            .with_context(|| format!("failed to write staged file: {}", target.display()))?;
    }

    Ok(())
}

fn promote_snapshot(profile_dir: &Path) -> Result<()> {
    let staging = staging_dir(profile_dir);
    let snapshot = snapshot_dir(profile_dir);

    if snapshot.exists() {
        fs::remove_dir_all(&snapshot)
            .with_context(|| format!("failed to remove old snapshot: {}", snapshot.display()))?;
    }

    fs::rename(&staging, &snapshot)
        .with_context(|| format!("failed to promote staging dir: {}", staging.display()))?;

    Ok(())
}

fn remove_staging(profile_dir: &Path) {
    let staging = staging_dir(profile_dir);
    if staging.exists() {
        let _ = fs::remove_dir_all(staging);
    }
}

fn load_snapshot_bytes(
    published: &PublishedState,
    profile_dir: &Path,
) -> Result<BTreeMap<ConfigPath, Vec<u8>>> {
    published
        .config
        .iter()
        .map(|(path, hash)| {
            let file = snapshot_dir(profile_dir).join(path.as_path());
            let bytes = fs::read(&file).with_context(|| {
                format!("failed to read published config file: {}", file.display())
            })?;
            ensure!(
                ContentHash::from_hash(blake3::hash(&bytes)) == *hash,
                "published config file does not match its recorded hash: {path}"
            );
            Ok((path.clone(), bytes))
        })
        .collect()
}

fn merge_publication(
    mode: PublishMode,
    published_manifest: ProfileManifest,
    published_bytes: BTreeMap<ConfigPath, Vec<u8>>,
    live_manifest: ProfileManifest,
    live_config: &BTreeMap<ConfigPath, Vec<u8>>,
) -> Result<(ProfileManifest, BTreeMap<ConfigPath, Vec<u8>>)> {
    let (manifest, selection) = match mode {
        PublishMode::Mods => (live_manifest, None),
        PublishMode::Config { files } => {
            ensure!(!files.is_empty(), "no config files selected");
            (published_manifest, Some(files))
        }
        PublishMode::Both { files } => (live_manifest, Some(files)),
    };

    let mut config = published_bytes;

    if let Some(files) = selection {
        let mut seen = HashSet::new();
        for path in files {
            ensure!(
                seen.insert(path.clone()),
                "duplicate config selection: {path}"
            );
            let Some(bytes) = live_config.get(&path) else {
                bail!("selected config file is missing locally: {path}");
            };
            config.insert(path, bytes.clone());
        }
    }

    Ok((manifest, config))
}

fn build_publication(
    name: &str,
    mut manifest: ProfileManifest,
    config: &BTreeMap<ConfigPath, Vec<u8>>,
) -> Result<(Vec<u8>, PublishedState)> {
    manifest.name = name.to_owned();
    manifest.sync = None;

    let mut archive_manifest = manifest.clone();
    archive_manifest.sync = Some(SyncManifest {
        version: 1,
        mods_revision: manifest_revision(&manifest)?,
        config: config
            .iter()
            .map(|(path, bytes)| {
                (
                    path.clone(),
                    SyncFileEntry {
                        hash: ContentHash::from_hash(blake3::hash(bytes)),
                    },
                )
            })
            .collect(),
    });

    let mut writer = Cursor::new(Vec::new());
    export::write_archive(&archive_manifest, config, &mut writer)
        .context("failed to write sync archive")?;
    let bytes = writer.into_inner();

    let validated = archive::validate(&bytes).context("generated archive failed validation")?;
    let state = published_state(&validated)?;

    Ok((bytes, state))
}

pub(super) async fn reconcile_published(app: &AppHandle, profile_id: i64) -> Result<()> {
    let (sync_id, profile_dir, data) = {
        let manager = app.lock_manager();
        let (_, profile) = manager.profile_by_id(profile_id)?;
        let data = profile.sync.clone().ok_or_eyre("profile is not synced")?;
        (data.id.clone(), profile.path.clone(), data)
    };

    let metadata = get_profile_meta(&sync_id, app)
        .await?
        .ok_or_eyre("synced profile does not exist on the server")?;

    if !needs_reconcile(&data, metadata.updated_at, &profile_dir)? {
        let mut manager = app.lock_manager();
        let (_, profile) = manager.profile_by_id_mut(profile_id)?;
        let Some(sync) = profile.sync.as_mut() else {
            return Ok(());
        };
        ensure!(
            sync.id == sync_id,
            "synced profile changed during reconcile"
        );

        if sync.updated_at != metadata.updated_at || sync.missing {
            sync.owner = metadata.owner;
            sync.updated_at = metadata.updated_at;
            sync.missing = false;
            profile.save(app, true)?;
        }

        return Ok(());
    }

    let bytes = download_profile_bytes(&sync_id, app).await?;
    let validated = archive::validate(&bytes).context("remote sync archive failed validation")?;
    let state = adopt_publication(&profile_dir, &validated)?;

    {
        let mut manager = app.lock_manager();
        let (_, profile) = manager.profile_by_id_mut(profile_id)?;
        let Some(sync) = profile.sync.as_mut() else {
            bail!("profile is no longer synced");
        };
        ensure!(
            sync.id == sync_id,
            "synced profile changed during reconcile"
        );

        sync.published = Some(state);
        sync.owner = metadata.owner;
        sync.synced_at = metadata.updated_at;
        sync.updated_at = metadata.updated_at;
        sync.missing = false;
        profile.save(app, true)?;
    }

    Ok(())
}

pub(super) async fn publish_profile(
    app: &AppHandle,
    profile_id: i64,
    mode: PublishMode,
) -> Result<()> {
    reconcile_published(app, profile_id).await?;

    let (profile_dir, profile_name, sync_id, published, live_manifest, live_config) = {
        let manager = app.lock_manager();
        let (game, profile) = manager.profile_by_id(profile_id)?;

        let sync = profile.sync.as_ref().ok_or_eyre("profile is not synced")?;
        let published = sync
            .published
            .clone()
            .ok_or_eyre("profile has no published state")?;

        (
            profile.path.clone(),
            profile.name.clone(),
            sync.id.clone(),
            published,
            export::build_manifest(profile, game),
            export::collect_config_files(&profile.path, game.mod_loader.mod_config_dirs())?,
        )
    };

    let published_bytes = load_snapshot_bytes(&published, &profile_dir)?;

    let (manifest, config) = merge_publication(
        mode,
        published.manifest,
        published_bytes,
        live_manifest,
        &live_config,
    )?;

    let (bytes, state) = build_publication(&profile_name, manifest, &config)?;

    stage_snapshot(&config, &profile_dir)?;

    let response =
        match upload_profile_file(app, bytes, Method::PUT, format!("/profile/{sync_id}")).await {
            Ok(response) => response,
            Err(err) => {
                remove_staging(&profile_dir);
                return Err(err);
            }
        };

    promote_snapshot(&profile_dir)?;

    let mut manager = app.lock_manager();
    let (_, profile) = manager.profile_by_id_mut(profile_id)?;
    let Some(sync) = profile.sync.as_mut() else {
        bail!("profile is no longer synced");
    };
    ensure!(sync.id == sync_id, "synced profile changed during publish");

    sync.published = Some(state);
    sync.synced_at = response.updated_at;
    sync.updated_at = response.updated_at;
    profile.save(app, true)?;

    Ok(())
}

pub(super) async fn create_profile(app: &AppHandle) -> Result<String> {
    let Some(user) = auth::user_info(app) else {
        bail!("not logged in");
    };

    let (profile_id, profile_dir, profile_name, manifest, config) = {
        let manager = app.lock_manager();
        let game = manager.active_game().game;
        let profile = manager.active_profile();

        (
            profile.id,
            profile.path.clone(),
            profile.name.clone(),
            export::build_manifest(profile, game),
            export::collect_config_files(&profile.path, game.mod_loader.mod_config_dirs())?,
        )
    };

    let (bytes, state) = build_publication(&profile_name, manifest, &config)?;

    stage_snapshot(&config, &profile_dir)?;

    let response = match upload_profile_file(app, bytes, Method::POST, "/profile").await {
        Ok(response) => response,
        Err(err) => {
            remove_staging(&profile_dir);
            return Err(err);
        }
    };

    promote_snapshot(&profile_dir)?;

    {
        let mut manager = app.lock_manager();
        let (_, profile) = manager.profile_by_id_mut(profile_id)?;

        profile.sync = Some(SyncProfileData {
            id: response.id.clone(),
            owner: user,
            synced_at: response.updated_at,
            updated_at: response.updated_at,
            missing: false,
            published: Some(state),
            applied: None,
        });
        profile.save(app, true)?;
    }

    Ok(response.id)
}

pub(super) async fn list_config_files(
    app: &AppHandle,
    profile_id: i64,
) -> Result<Vec<SyncConfigFileInfo>> {
    reconcile_published(app, profile_id).await?;

    let (live_config, published_config) = {
        let manager = app.lock_manager();
        let (game, profile) = manager.profile_by_id(profile_id)?;

        (
            export::collect_config_files(&profile.path, game.mod_loader.mod_config_dirs())?,
            profile
                .sync
                .as_ref()
                .and_then(|sync| sync.published.as_ref())
                .map(|published| published.config.clone())
                .unwrap_or_default(),
        )
    };

    live_config
        .into_iter()
        .map(|(path, bytes)| {
            let status = match published_config.get(&path) {
                Some(hash) if ContentHash::from_hash(blake3::hash(&bytes)) == *hash => {
                    SyncConfigFileStatus::Published
                }
                Some(_) => SyncConfigFileStatus::Modified,
                None => SyncConfigFileStatus::New,
            };
            Ok(SyncConfigFileInfo {
                path,
                size: bytes.len() as u64,
                status,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;
    use crate::{
        profile::export::{R2Mod, R2Version},
        thunderstore::{Backend, PackageIdent},
    };

    fn config_path(path: &str) -> ConfigPath {
        ConfigPath::try_from(path.to_owned()).unwrap()
    }

    fn base_manifest() -> ProfileManifest {
        ProfileManifest {
            name: "Test".to_owned(),
            mods: vec![R2Mod {
                ident: PackageIdent::from(("Author", "Mod")),
                version: R2Version {
                    major: 1,
                    minor: 0,
                    patch: 0,
                },
                enabled: true,
                source: Backend::Thunderstore,
            }],
            game: Some("risk-of-rain-2".to_owned()),
            ignored_version_updates: Vec::new(),
            ignored_package_updates: Vec::new(),
            sync: None,
        }
    }

    fn published_state_with(config: &[(&str, &[u8])]) -> PublishedState {
        let manifest = base_manifest();
        PublishedState {
            mods_revision: manifest_revision(&manifest).unwrap(),
            manifest,
            config: config
                .iter()
                .map(|&(path, bytes)| {
                    (
                        config_path(path),
                        ContentHash::from_hash(blake3::hash(bytes)),
                    )
                })
                .collect(),
        }
    }

    fn write_file(path: PathBuf, contents: &[u8]) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    #[test]
    fn published_state_from_selective_archive() {
        let mut manifest = base_manifest();
        let path = config_path("BepInEx/config/example.cfg");
        let bytes = b"data".to_vec();
        let hash = ContentHash::from_hash(blake3::hash(&bytes));

        manifest.sync = Some(SyncManifest {
            version: 1,
            mods_revision: manifest_revision(&manifest).unwrap(),
            config: BTreeMap::from([(path.clone(), SyncFileEntry { hash: hash.clone() })]),
        });

        let config = BTreeMap::from([(path.clone(), bytes)]);
        let mut writer = Cursor::new(Vec::new());
        export::write_archive(&manifest, &config, &mut writer).unwrap();

        let validated = archive::validate(writer.get_ref()).unwrap();
        let state = published_state(&validated).unwrap();

        assert!(state.manifest.sync.is_none());
        assert_eq!(
            state.mods_revision,
            manifest_revision(&state.manifest).unwrap()
        );
        assert_eq!(state.config[&path], hash);
    }

    #[test]
    fn published_state_from_legacy_archive() {
        let manifest = base_manifest();
        let path = config_path("BepInEx/config/example.cfg");
        let bytes = b"data".to_vec();

        let config = BTreeMap::from([(path.clone(), bytes.clone())]);
        let mut writer = Cursor::new(Vec::new());
        export::write_archive(&manifest, &config, &mut writer).unwrap();

        let validated = archive::validate(writer.get_ref()).unwrap();
        let state = published_state(&validated).unwrap();

        assert!(state.manifest.sync.is_none());
        assert_eq!(
            state.mods_revision,
            manifest_revision(&state.manifest).unwrap()
        );
        assert_eq!(
            state.config[&path],
            ContentHash::from_hash(blake3::hash(&bytes))
        );
    }

    #[test]
    fn snapshot_consistency() {
        let dir = tempdir().unwrap();
        let path = config_path("BepInEx/config/example.cfg");
        let file = dir
            .path()
            .join(SNAPSHOT_DIR)
            .join(path.as_path())
            .to_owned();
        let state = published_state_with(&[("BepInEx/config/example.cfg", b"data")]);

        assert!(!snapshot_consistent(&state, dir.path()).unwrap());

        write_file(file.clone(), b"data");
        assert!(snapshot_consistent(&state, dir.path()).unwrap());

        write_file(file.clone(), b"changed");
        assert!(!snapshot_consistent(&state, dir.path()).unwrap());

        write_file(file, b"data");
        let mut tampered = state.clone();
        tampered.mods_revision =
            crate::profile::export::ModRevision::from_hash(blake3::hash(b"wrong"));
        assert!(!snapshot_consistent(&tampered, dir.path()).unwrap());
    }

    #[test]
    fn stage_and_promote_snapshot() {
        let dir = tempdir().unwrap();
        let staging = dir.path().join(STAGING_DIR);
        let snapshot = dir.path().join(SNAPSHOT_DIR);

        fs::create_dir_all(&staging).unwrap();
        fs::write(staging.join("stale.cfg"), b"stale").unwrap();

        fs::create_dir_all(&snapshot).unwrap();
        fs::write(snapshot.join("old.cfg"), b"old").unwrap();

        let config = BTreeMap::from([(config_path("a/b.cfg"), b"new".to_vec())]);
        stage_snapshot(&config, dir.path()).unwrap();

        assert!(!staging.join("stale.cfg").exists());
        assert_eq!(fs::read(staging.join("a/b.cfg")).unwrap(), b"new");

        promote_snapshot(dir.path()).unwrap();

        assert!(!staging.exists());
        assert!(!snapshot.join("old.cfg").exists());
        assert_eq!(fs::read(snapshot.join("a/b.cfg")).unwrap(), b"new");
    }

    #[test]
    fn merge_modes() {
        let published_manifest = base_manifest();
        let live_manifest = ProfileManifest {
            name: "Live".to_owned(),
            ..base_manifest()
        };

        let old_path = config_path("old.cfg");
        let sel_path = config_path("sel.cfg");
        let new_path = config_path("new.cfg");

        let published_bytes = BTreeMap::from([
            (old_path.clone(), b"old".to_vec()),
            (sel_path.clone(), b"old-sel".to_vec()),
        ]);
        let live_config = BTreeMap::from([
            (sel_path.clone(), b"live-sel".to_vec()),
            (new_path.clone(), b"new".to_vec()),
        ]);

        let (manifest, config) = merge_publication(
            PublishMode::Mods,
            published_manifest.clone(),
            published_bytes.clone(),
            live_manifest.clone(),
            &live_config,
        )
        .unwrap();
        assert_eq!(
            serde_json::to_string(&manifest).unwrap(),
            serde_json::to_string(&live_manifest).unwrap()
        );
        assert_eq!(config, published_bytes);

        let (manifest, config) = merge_publication(
            PublishMode::Config {
                files: vec![sel_path.clone()],
            },
            published_manifest.clone(),
            published_bytes.clone(),
            live_manifest.clone(),
            &live_config,
        )
        .unwrap();
        assert_eq!(
            serde_json::to_string(&manifest).unwrap(),
            serde_json::to_string(&published_manifest).unwrap()
        );
        assert_eq!(config[&sel_path], b"live-sel");
        assert_eq!(config[&old_path], b"old");
        assert!(!config.contains_key(&new_path));

        let (manifest, config) = merge_publication(
            PublishMode::Both {
                files: vec![sel_path.clone()],
            },
            published_manifest.clone(),
            published_bytes.clone(),
            live_manifest.clone(),
            &live_config,
        )
        .unwrap();
        assert_eq!(
            serde_json::to_string(&manifest).unwrap(),
            serde_json::to_string(&live_manifest).unwrap()
        );
        assert_eq!(config[&sel_path], b"live-sel");
        assert_eq!(config[&old_path], b"old");

        let (_, config) = merge_publication(
            PublishMode::Both { files: vec![] },
            published_manifest.clone(),
            published_bytes.clone(),
            live_manifest.clone(),
            &live_config,
        )
        .unwrap();
        assert_eq!(config, published_bytes);

        assert!(
            merge_publication(
                PublishMode::Config { files: vec![] },
                published_manifest.clone(),
                published_bytes.clone(),
                live_manifest.clone(),
                &live_config,
            )
            .is_err()
        );
        assert!(
            merge_publication(
                PublishMode::Config {
                    files: vec![sel_path.clone(), sel_path.clone()],
                },
                published_manifest.clone(),
                published_bytes.clone(),
                live_manifest.clone(),
                &live_config,
            )
            .is_err()
        );
        assert!(
            merge_publication(
                PublishMode::Config {
                    files: vec![config_path("missing.cfg")],
                },
                published_manifest,
                published_bytes,
                live_manifest,
                &live_config,
            )
            .is_err()
        );
    }

    #[test]
    fn build_publication_validates() {
        let manifest = base_manifest();
        let path = config_path("BepInEx/config/example.cfg");
        let config = BTreeMap::from([(path.clone(), b"live".to_vec())]);

        let (bytes, state) = build_publication("Owner Profile", manifest, &config).unwrap();

        let validated = archive::validate(&bytes).unwrap();
        assert!(matches!(
            validated.format,
            archive::SyncArchiveFormat::Selective(_)
        ));
        assert_eq!(validated.config[&path].bytes, b"live");
        assert_eq!(validated.config[&path].hash, state.config[&path]);

        assert!(state.manifest.sync.is_none());
        assert_eq!(state.manifest.name, "Owner Profile");
        assert_eq!(
            state.mods_revision,
            manifest_revision(&state.manifest).unwrap()
        );
    }

    #[test]
    fn reconcile_decision() {
        let dir = tempdir().unwrap();
        let now = Utc::now();

        let mut data = SyncProfileData {
            id: "id".to_owned(),
            owner: auth::User {
                discord_id: "1".to_owned(),
                name: "user".to_owned(),
                display_name: "User".to_owned(),
                avatar: None,
            },
            synced_at: now,
            updated_at: now,
            missing: false,
            published: None,
            applied: None,
        };

        assert!(needs_reconcile(&data, now, dir.path()).unwrap());

        data.published = Some(published_state_with(&[("a.cfg", b"data")]));
        assert!(needs_reconcile(&data, now, dir.path()).unwrap());

        write_file(dir.path().join(SNAPSHOT_DIR).join("a.cfg"), b"data");
        assert!(!needs_reconcile(&data, now, dir.path()).unwrap());

        assert!(needs_reconcile(&data, now + chrono::Duration::seconds(1), dir.path()).unwrap());
    }
}
