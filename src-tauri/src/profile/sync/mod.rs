use std::{
    borrow::Cow,
    collections::BTreeMap,
    env,
    fmt::Display,
    path::{Component, Path},
    sync::LazyLock,
};

use chrono::{DateTime, Utc};
use eyre::{Context, OptionExt, Result, bail, ensure, eyre};
use reqwest::{Method, StatusCode};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tracing::warn;

use super::export::{
    ConfigPath, ContentHash, ModRevision, ProfileManifest, R2Mod, SyncFileEntry, SyncManifest,
    manifest_revision,
};
use crate::{
    profile::{
        Profile,
        import::{ImportOptions, ImportTarget},
        install::InstallOptions,
    },
    state::ManagerExt,
};

mod apply;
pub(super) mod archive;
pub mod auth;
pub mod commands;
mod publish;
pub mod socket;

static API_URL: LazyLock<Cow<'static, str>> = LazyLock::new(|| match env::var("GALE_SYNC_URL") {
    Ok(var) => var.into(),
    Err(_) => "https://gale.kesomannen.com/api".into(),
});

async fn request(
    method: Method,
    path: impl Display,
    app: &AppHandle,
) -> reqwest_middleware::RequestBuilder {
    let url = format!("{}{path}", *API_URL);

    let mut req = app.http().request(method, url);
    if let Some(token) = auth::access_token(app).await {
        req = req.bearer_auth(token);
    }
    req
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateSyncProfileResponse {
    id: String,
    #[allow(unused)]
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SyncProfileMetadata {
    id: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    owner: auth::User,
    manifest: ProfileManifest,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SyncProfileData {
    id: String,
    owner: auth::User,
    synced_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    #[serde(default)]
    missing: bool,
    #[serde(default)]
    pub published: Option<PublishedState>,
    #[serde(default)]
    pub applied: Option<AppliedState>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PublishedState {
    pub manifest: ProfileManifest,
    pub mods_revision: ModRevision,
    pub config: BTreeMap<ConfigPath, ContentHash>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppliedState {
    pub mods_revision: Option<ModRevision>,
    pub latest: Option<SyncManifest>,
    pub config: BTreeMap<ConfigPath, AppliedFile>,
    pub pending: BTreeMap<ConfigPath, PendingConfigReason>,
    #[serde(default)]
    pub declined: BTreeMap<ConfigPath, PendingConfigReason>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppliedFile {
    pub applied: Option<ContentHash>,
    pub written: Option<ContentHash>,
    pub declined: Option<ContentHash>,
    #[serde(default)]
    pub policy: ConfigUpdatePolicy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_set_at: Option<ContentHash>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ConfigUpdatePolicy {
    #[default]
    Ask,
    AlwaysApply,
    AlwaysKeep,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PendingConfigReason {
    ModifiedLocally,
    DeletedLocally,
}

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PullReport {
    pub mods_updated: bool,
    pub config: apply::ConfigApplyReport,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct FullUserInfo {
    #[serde(flatten)]
    user: auth::User,
    profiles: Option<Vec<ListedSyncProfile>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListedSyncProfile {
    id: String,
    name: String,
    community: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<SyncProfileMetadata> for SyncProfileData {
    fn from(value: SyncProfileMetadata) -> Self {
        SyncProfileData {
            id: value.id,
            owner: value.owner,
            synced_at: value.updated_at,
            updated_at: value.updated_at,
            missing: false,
            published: None,
            applied: None,
        }
    }
}

async fn create_profile(app: &AppHandle) -> Result<String> {
    publish::create_profile(app).await
}

pub async fn push_profile(app: &AppHandle, profile_id: i64) -> Result<()> {
    let files = {
        let manager = app.lock_manager();
        let (game, profile) = manager.profile_by_id(profile_id)?;

        super::export::collect_config_files(&profile.path, game.mod_loader.mod_config_dirs())?
            .into_keys()
            .collect()
    };

    publish::publish_profile(app, profile_id, publish::PublishMode::Both { files }).await
}

async fn upload_profile_file(
    app: &AppHandle,
    bytes: Vec<u8>,
    method: Method,
    endpoint: impl Display,
) -> Result<CreateSyncProfileResponse> {
    let len = bytes.len();
    let res = request(method, endpoint, app)
        .await
        .body(bytes)
        .send()
        .await?;

    if res.status().is_success() {
        let response = res.json().await?;
        Ok(response)
    } else if res.status() == StatusCode::PAYLOAD_TOO_LARGE {
        bail!(
            "profile config is too large to upload: {}, please reduce the size by removing heavy and/or unneeded config files",
            humansize::format_size(len, humansize::BINARY)
        );
    } else {
        bail!("upload failed with status: {}", res.status());
    }
}

async fn disconnect_profile(delete: bool, app: &AppHandle) -> Result<()> {
    let (id, is_owner) = {
        let mut manager = app.lock_manager();
        let profile = manager.active_profile_mut();

        let (id, owner_discord_id) = profile
            .sync
            .as_ref()
            .map(|info| (info.id.clone(), &info.owner.discord_id))
            .ok_or_eyre("profile is not synced")?;

        let is_owner =
            auth::user_info(app).is_some_and(|user| user.discord_id == *owner_discord_id);

        (id, is_owner)
    };

    if is_owner && delete {
        delete_profile(&id, app).await?;
    }

    {
        let mut manager = app.lock_manager();
        let profile = manager.active_profile_mut();

        profile.sync = None;

        profile.save(app, true)?;
    }

    Ok(())
}

struct NormalizedArchive<'a> {
    manifest: ProfileManifest,
    config: Cow<'a, BTreeMap<ConfigPath, archive::ValidatedConfigFile>>,
    latest: SyncManifest,
    selective: bool,
}

fn normalize_legacy_path(path: &ConfigPath) -> Result<ConfigPath> {
    let mut components = path.as_path().components();
    let Some(Component::Normal(first)) = components.next() else {
        bail!("invalid config path: {path}");
    };

    if first.to_str() != Some("config") {
        return Ok(path.clone());
    }

    let mut mapped = String::from("BepInEx/config");
    for component in components {
        let Component::Normal(part) = component else {
            bail!("invalid config path: {path}");
        };
        let part = part.to_str().ok_or_eyre("config path is not valid UTF-8")?;
        mapped.push('/');
        mapped.push_str(part);
    }

    ConfigPath::try_from(mapped)
}

fn normalize_archive(archive: &archive::ValidatedSyncArchive) -> Result<NormalizedArchive<'_>> {
    let mut manifest = archive.manifest.clone();

    let (config, latest, selective) = match &archive.format {
        archive::SyncArchiveFormat::Selective(sync) => {
            (Cow::Borrowed(&archive.config), sync.clone(), true)
        }
        archive::SyncArchiveFormat::Legacy => {
            let mut config: BTreeMap<ConfigPath, archive::ValidatedConfigFile> = BTreeMap::new();
            for (path, file) in &archive.config {
                let normalized = normalize_legacy_path(path)?;
                ensure!(
                    config.keys().all(|existing| !existing
                        .as_str()
                        .eq_ignore_ascii_case(normalized.as_str())),
                    "config paths collide after legacy normalization: {normalized}"
                );
                config.insert(normalized, file.clone());
            }

            let latest = SyncManifest {
                version: 1,
                mods_revision: manifest_revision(&manifest)?,
                config: config
                    .iter()
                    .map(|(path, file)| {
                        (
                            path.clone(),
                            SyncFileEntry {
                                hash: file.hash.clone(),
                            },
                        )
                    })
                    .collect(),
            };

            (Cow::Owned(config), latest, false)
        }
    };

    manifest.sync = None;

    Ok(NormalizedArchive {
        manifest,
        config,
        latest,
        selective,
    })
}

fn preseed_migration(state: &mut AppliedState, latest: &SyncManifest, existed: bool) {
    if !existed {
        return;
    }

    for path in latest.config.keys() {
        state.config.entry(path.clone()).or_default();
    }
}

/// Whether the profile's installed thunderstore mods exactly match the
/// manifest, by (owner, name, version, enabled) tuples.
fn mod_set_matches(profile: &Profile, expected: &[R2Mod]) -> bool {
    let mut installed: Vec<_> = profile
        .thunderstore_mods()
        .map(|(ts_mod, enabled)| {
            let (owner, name, version) = ts_mod.ident.split();
            (
                owner.to_owned(),
                name.to_owned(),
                version.to_owned(),
                enabled,
            )
        })
        .collect();
    installed.sort();

    let mut expected: Vec<_> = expected
        .iter()
        .map(|r2_mod| {
            let ident = r2_mod.version_ident();
            (
                ident.owner().to_owned(),
                ident.name().to_owned(),
                ident.version().to_owned(),
                r2_mod.enabled,
            )
        })
        .collect();
    expected.sort();

    installed == expected
}

async fn apply_archive(
    archive: &archive::ValidatedSyncArchive,
    normalized: NormalizedArchive<'_>,
    metadata: SyncProfileMetadata,
    override_name: Option<String>,
    clone: bool,
    target: ImportTarget,
    app: &AppHandle,
) -> Result<PullReport> {
    let mut manifest = normalized.manifest;
    if let Some(name) = override_name {
        manifest.name = name;
    }
    let latest = normalized.latest;

    let profile_id = match &target {
        ImportTarget::Existing(id) => Some(*id),
        ImportTarget::Named { .. } => None,
    };

    let (prior_sync, existed, existing_dir, game) = {
        let manager = app.lock_manager();

        match &target {
            ImportTarget::Existing(id) => {
                let (game, profile) = manager.profile_by_id(*id)?;
                (profile.sync.clone(), true, Some(profile.path.clone()), game)
            }
            ImportTarget::Named { game } => {
                let resolved = manager
                    .games
                    .get(game)
                    .and_then(|game| game.find_profile_index(&manifest.name))
                    .map(|index| &manager.games[game].profiles[index]);

                match resolved {
                    Some(profile) => (
                        profile.sync.clone(),
                        true,
                        Some(profile.path.clone()),
                        *game,
                    ),
                    None => (None, false, None, *game),
                }
            }
        }
    };

    if clone && profile_id.is_none() && existed {
        ensure_clone_target(prior_sync.as_ref(), &metadata.id, &manifest.name)?;
    }

    let mut applied = prior_sync
        .as_ref()
        .and_then(|sync| sync.applied.clone())
        .unwrap_or_default();
    let applied_was_none = prior_sync
        .as_ref()
        .is_none_or(|sync| sync.applied.is_none());
    let prior_sync_id = prior_sync.as_ref().map(|sync| sync.id.clone());
    let was_owner = prior_sync
        .as_ref()
        .is_some_and(|sync| sync.published.is_some());

    let selective = normalized.selective;
    let needs_install = clone || applied.mods_revision.as_ref() != Some(&latest.mods_revision);

    let before = if needs_install && existed {
        apply::snapshot_config(
            existing_dir.as_ref().unwrap(),
            game.mod_loader.mod_config_dirs(),
        )?
    } else {
        BTreeMap::new()
    };

    let (imported, target_id, profile_dir, created, expected_mods) = if needs_install {
        super::import::resolve_manifest_sources(&mut manifest, &app.lock_thunderstore());
        let expected_mods = selective.then(|| manifest.mods.clone());

        let imported = super::import::import_manifest(
            manifest,
            target,
            ImportOptions::default().ignore_missing_mods(!selective),
            InstallOptions::default(),
            app,
        )
        .await
        .context("failed to import synced profile")?;

        let target_id = imported.id;
        let profile_dir = imported.path.clone();
        let created = imported.created;
        (
            Some(imported),
            target_id,
            profile_dir,
            created,
            expected_mods,
        )
    } else {
        (
            None,
            profile_id.unwrap(),
            existing_dir.unwrap(),
            false,
            None,
        )
    };

    let result = async {
        if let (Some(id), Some(imported)) = (profile_id, imported.as_ref()) {
            ensure!(imported.id == id, "synced profile changed during apply");
        }

        // reject rather than overwrite: a foreign install queued or running for
        // this profile would make the installed set diverge after verification
        ensure!(
            !app.install_queue().lock().has_any_for_profile(target_id),
            "another install is queued for this profile"
        );

        let mut report = PullReport::default();

        if let Some(imported) = &imported {
            let after =
                apply::snapshot_config(&imported.path, imported.game.mod_loader.mod_config_dirs())?;
            apply::record_installer_written(&before, &after, &mut applied);
            applied.mods_revision = Some(latest.mods_revision.clone());
            report.mods_updated = true;
        }

        if applied_was_none {
            preseed_migration(&mut applied, &latest, existed);
        }

        apply::preserve_pending_policy_boundaries(&mut applied);
        applied.latest = Some(latest);
        report.config =
            apply::apply_available_config(&profile_dir, &normalized.config, &mut applied)?;

        let published = if was_owner {
            Some(publish::adopt_publication(&profile_dir, archive)?)
        } else {
            None
        };

        {
            let mut manager = app.lock_manager();
            let (_, profile) = manager.profile_by_id_mut(target_id)?;

            let current_sync_id = profile.sync.as_ref().map(|sync| sync.id.clone());
            ensure!(
                current_sync_id == prior_sync_id,
                "profile sync target changed during apply"
            );

            if let Some(expected) = &expected_mods {
                ensure!(
                    mod_set_matches(profile, expected),
                    "installed mod set does not match the synced manifest"
                );
            }

            profile.sync = Some(SyncProfileData {
                id: metadata.id,
                owner: metadata.owner,
                synced_at: metadata.updated_at,
                updated_at: metadata.updated_at,
                missing: false,
                published,
                applied: Some(applied),
            });
            profile.save(app, true)?;
        }

        Ok::<_, eyre::Report>(report)
    }
    .await;

    let report = match result {
        Ok(report) => report,
        Err(err) => {
            if created {
                super::import::cleanup_failed_profile(target_id, app).unwrap_or_else(|err| {
                    warn!(
                        "failed to remove profile after failed or cancelled apply: {}",
                        err
                    );
                });
            }
            return Err(err);
        }
    };

    app.emit_buffered("sync_config_review_changed", &());

    if !report.config.pending.is_empty() {
        app.emit_buffered("sync_config_pending", &report.config.pending);
    }

    Ok(report)
}

async fn clone_profile(id: &str, override_name: Option<String>, app: &AppHandle) -> Result<()> {
    // capture the target game before any awaits so a mid-download switch
    // can't redirect the clone into another game
    let game = app.lock_manager().active_game().game;

    let metadata = read_profile(id, app).await?;
    let bytes = download_profile_bytes(id, app).await?;
    let validated = archive::validate(&bytes).context("sync archive failed validation")?;
    let normalized = normalize_archive(&validated)?;

    apply_archive(
        &validated,
        normalized,
        metadata,
        override_name,
        true,
        ImportTarget::Named { game },
        app,
    )
    .await?;

    Ok(())
}

pub async fn pull_profile(dry_run: bool, app: &AppHandle) -> Result<PullReport> {
    let (id, profile_id, name, synced_at) = {
        let mut manager = app.lock_manager();
        let profile = manager.active_profile_mut();

        match &profile.sync {
            Some(data) if data.missing => bail!("cannot pull from missing profile"),
            Some(data) => (
                data.id.clone(),
                profile.id,
                profile.name.clone(),
                data.synced_at,
            ),
            None => return Ok(PullReport::default()),
        }
    };

    let metadata = get_profile_meta(&id, app).await?;

    match metadata {
        Some(metadata) if !dry_run && metadata.updated_at > synced_at => {
            let bytes = download_profile_bytes(&id, app).await?;
            let validated = archive::validate(&bytes).context("sync archive failed validation")?;
            let normalized = normalize_archive(&validated)?;

            apply_archive(
                &validated,
                normalized,
                metadata,
                Some(name),
                false,
                ImportTarget::Existing(profile_id),
                app,
            )
            .await
        }
        metadata => {
            let mut manager = app.lock_manager();
            let (_, profile) = manager.profile_by_id_mut(profile_id)?;

            let Some(sync) = profile.sync.as_mut() else {
                return Ok(PullReport::default());
            };

            match metadata {
                Some(metadata) => {
                    *sync = SyncProfileData {
                        synced_at: sync.synced_at,
                        published: sync.published.clone(),
                        applied: sync.applied.clone(),
                        ..metadata.into()
                    };
                }
                None => sync.missing = true,
            }

            profile.save(app, true)?;

            Ok(PullReport::default())
        }
    }
}

fn pending_config_items(app: &AppHandle) -> Result<apply::ConfigReviewState> {
    let manager = app.lock_manager();
    let profile = manager.active_profile();

    let Some(applied) = profile.sync.as_ref().and_then(|sync| sync.applied.as_ref()) else {
        return Ok(apply::ConfigReviewState::default());
    };

    apply::current_review_items(&profile.path, applied)
}

fn decline_selected_config(files: &[ConfigPath], remember: bool, app: &AppHandle) -> Result<()> {
    let (profile_id, sync_id, mut applied) = {
        let manager = app.lock_manager();
        let profile = manager.active_profile();
        let sync = profile.sync.as_ref().ok_or_eyre("profile is not synced")?;
        let applied = sync.applied.clone().ok_or_eyre("no applied sync state")?;
        (profile.id, sync.id.clone(), applied)
    };

    apply::decline_selected(&mut applied, files, remember)?;

    let mut manager = app.lock_manager();
    let profile = manager.active_profile_mut();
    ensure!(
        profile.id == profile_id,
        "active profile changed during decline"
    );
    let Some(sync) = profile.sync.as_mut() else {
        bail!("profile is no longer synced");
    };
    ensure!(
        sync.id == sync_id,
        "profile sync target changed during decline"
    );
    sync.applied = Some(applied);
    profile.save(app, true)
}

fn set_config_policy(file: ConfigPath, policy: ConfigUpdatePolicy, app: &AppHandle) -> Result<()> {
    let (profile_id, sync_id, mut applied) = {
        let manager = app.lock_manager();
        let profile = manager.active_profile();
        let sync = profile.sync.as_ref().ok_or_eyre("profile is not synced")?;
        let applied = sync.applied.clone().ok_or_eyre("no applied sync state")?;
        (profile.id, sync.id.clone(), applied)
    };

    apply::set_policy(&mut applied, &file, policy)?;

    let mut manager = app.lock_manager();
    let profile = manager.active_profile_mut();
    ensure!(
        profile.id == profile_id,
        "active profile changed during policy update"
    );
    let Some(sync) = profile.sync.as_mut() else {
        bail!("profile is no longer synced");
    };
    ensure!(
        sync.id == sync_id,
        "profile sync target changed during policy update"
    );
    sync.applied = Some(applied);
    profile.save(app, true)
}

fn ensure_clone_target(
    existing: Option<&SyncProfileData>,
    remote_id: &str,
    name: &str,
) -> Result<()> {
    if existing.map(|sync| sync.id.as_str()) != Some(remote_id) {
        bail!(
            "a profile named '{name}' already exists and is not linked to sync profile {remote_id}"
        );
    }
    Ok(())
}

fn ensure_latest(applied: &AppliedState, latest: &SyncManifest) -> Result<()> {
    ensure!(
        applied.latest.as_ref() == Some(latest),
        "published sync revision changed; pull the latest update before applying config"
    );
    Ok(())
}

fn sync_apply_target<'a>(
    active_profile_id: i64,
    sync: &'a mut Option<SyncProfileData>,
    expected_profile_id: i64,
    expected_sync_id: &str,
) -> Result<&'a mut SyncProfileData> {
    ensure!(
        active_profile_id == expected_profile_id,
        "active profile changed during apply"
    );
    let Some(sync) = sync.as_mut() else {
        bail!("profile is no longer synced");
    };
    ensure!(
        sync.id == expected_sync_id,
        "profile sync target changed during apply"
    );
    Ok(sync)
}

fn apply_selected_and_record<F>(
    profile_dir: &Path,
    config: &BTreeMap<ConfigPath, archive::ValidatedConfigFile>,
    latest: &SyncManifest,
    sync: &mut SyncProfileData,
    files: &[ConfigPath],
    restore_deleted: &[ConfigPath],
    remember: bool,
    write: F,
) -> Result<Vec<ConfigPath>>
where
    F: FnMut(&Path, &archive::ValidatedConfigFile) -> Result<()>,
{
    let mut applied = sync.applied.clone().ok_or_eyre("no applied sync state")?;
    ensure_latest(&applied, latest)?;
    let result = apply::apply_selected_with_writer(
        profile_dir,
        config,
        &mut applied,
        files,
        restore_deleted,
        remember,
        write,
    );
    sync.applied = Some(applied);
    result
}

async fn apply_selected_config(
    files: Vec<ConfigPath>,
    remember: bool,
    restore_deleted: Vec<ConfigPath>,
    app: &AppHandle,
) -> Result<Vec<ConfigPath>> {
    let (profile_id, sync_id) = {
        let manager = app.lock_manager();
        let profile = manager.active_profile();
        let sync = profile.sync.as_ref().ok_or_eyre("profile is not synced")?;
        (profile.id, sync.id.clone())
    };

    let bytes = download_profile_bytes(&sync_id, app).await?;
    let validated = archive::validate(&bytes).context("sync archive failed validation")?;
    let normalized = normalize_archive(&validated)?;

    let mut manager = app.lock_manager();
    let profile = manager.active_profile_mut();
    let profile_dir = profile.path.clone();
    let sync = sync_apply_target(profile.id, &mut profile.sync, profile_id, &sync_id)?;

    let apply_result = apply_selected_and_record(
        &profile_dir,
        &normalized.config,
        &normalized.latest,
        sync,
        &files,
        &restore_deleted,
        remember,
        apply::write_validated,
    );

    profile.save(app, true)?;

    apply_result
}

pub(super) async fn download_profile_bytes(id: &str, app: &AppHandle) -> Result<Vec<u8>> {
    let bytes = request(Method::GET, format!("/profile/{id}"), app)
        .await
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    Ok(bytes.to_vec())
}

async fn delete_profile(id: &str, app: &AppHandle) -> Result<()> {
    request(Method::DELETE, format!("/profile/{id}"), app)
        .await
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

async fn get_profile_meta(id: &str, app: &AppHandle) -> Result<Option<SyncProfileMetadata>> {
    let res = request(Method::GET, format!("/profile/{id}/meta"), app)
        .await
        .send()
        .await?
        .error_for_status();

    match res {
        Ok(res) => {
            let res = res.json().await?;
            Ok(Some(res))
        }
        Err(err) if err.status() == Some(StatusCode::NOT_FOUND) => Ok(None),
        Err(err) => Err(eyre!(err)),
    }
}

pub async fn read_profile(id: &str, app: &AppHandle) -> Result<SyncProfileMetadata> {
    get_profile_meta(id, app)
        .await?
        .ok_or_eyre("profile not found")
}

async fn get_owned_profiles(app: &AppHandle) -> Result<Vec<ListedSyncProfile>> {
    let user: FullUserInfo = request(Method::GET, "/user/me", app)
        .await
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(user.profiles.unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::{
        profile::export::R2Mod,
        thunderstore::{Backend, ModId, PackageIdent},
    };

    fn config_path(path: &str) -> ConfigPath {
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

    fn base_manifest() -> ProfileManifest {
        ProfileManifest {
            name: "Test".to_owned(),
            mods: vec![R2Mod {
                ident: PackageIdent::from(("Author", "Mod")),
                version: semver::Version::new(1, 0, 0).into(),
                enabled: true,
                source: Backend::Thunderstore,
            }],
            game: Some("risk-of-rain-2".to_owned()),
            ignored_version_updates: Vec::new(),
            ignored_package_updates: Vec::new(),
            sync: None,
        }
    }

    fn validated_archive(
        manifest: ProfileManifest,
        format: archive::SyncArchiveFormat,
        entries: &[(&str, &[u8])],
    ) -> archive::ValidatedSyncArchive {
        archive::ValidatedSyncArchive {
            manifest,
            format,
            config: entries
                .iter()
                .map(|&(path, bytes)| (config_path(path), vfile(bytes)))
                .collect(),
        }
    }

    #[test]
    fn legacy_normalization_maps_config_dir() {
        let validated = validated_archive(
            base_manifest(),
            archive::SyncArchiveFormat::Legacy,
            &[("config/a.cfg", b"data"), ("other/file.cfg", b"keep")],
        );

        let normalized = normalize_archive(&validated).unwrap();

        assert!(normalized.manifest.sync.is_none());
        assert!(
            normalized
                .config
                .contains_key(&config_path("BepInEx/config/a.cfg"))
        );
        assert_eq!(
            normalized
                .config
                .get(&config_path("BepInEx/config/a.cfg"))
                .unwrap()
                .bytes,
            b"data"
        );
        assert!(
            normalized
                .config
                .contains_key(&config_path("other/file.cfg"))
        );

        assert_eq!(normalized.latest.version, 1);
        assert_eq!(
            normalized.latest.mods_revision,
            manifest_revision(&normalized.manifest).unwrap()
        );
        assert_eq!(
            normalized.latest.config[&config_path("BepInEx/config/a.cfg")].hash,
            hash(b"data")
        );
        assert_eq!(
            normalized.latest.config[&config_path("other/file.cfg")].hash,
            hash(b"keep")
        );
    }

    #[test]
    fn legacy_normalization_rejects_collisions() {
        for entries in [
            [
                ("config/a.cfg", b"x".as_slice()),
                ("BepInEx/config/a.cfg", b"y".as_slice()),
            ],
            [
                ("config/A.cfg", b"x".as_slice()),
                ("BepInEx/config/a.cfg", b"y".as_slice()),
            ],
        ] {
            let validated = validated_archive(
                base_manifest(),
                archive::SyncArchiveFormat::Legacy,
                &entries,
            );
            assert!(normalize_archive(&validated).is_err());
        }
    }

    #[test]
    fn selective_normalization_preserves_exact() {
        let mut manifest = base_manifest();
        manifest.sync = Some(SyncManifest {
            version: 1,
            mods_revision: manifest_revision(&manifest).unwrap(),
            config: BTreeMap::from([(
                config_path("config/x.cfg"),
                SyncFileEntry {
                    hash: hash(b"data"),
                },
            )]),
        });
        let sync = manifest.sync.clone().unwrap();

        let validated = archive::ValidatedSyncArchive {
            manifest,
            format: archive::SyncArchiveFormat::Selective(sync.clone()),
            config: BTreeMap::from([(config_path("config/x.cfg"), vfile(b"data"))]),
        };

        let normalized = normalize_archive(&validated).unwrap();

        assert!(normalized.manifest.sync.is_none());
        assert!(normalized.config.contains_key(&config_path("config/x.cfg")));
        assert!(
            !normalized
                .config
                .contains_key(&config_path("BepInEx/config/x.cfg"))
        );
        assert_eq!(normalized.latest, sync);
    }

    #[test]
    fn migration_preseed_only_for_existing() {
        let latest = SyncManifest {
            version: 1,
            mods_revision: ModRevision::from_hash(blake3::hash(b"rev")),
            config: BTreeMap::from([(
                config_path("a.cfg"),
                SyncFileEntry {
                    hash: hash(b"remote"),
                },
            )]),
        };

        let mut state = AppliedState::default();
        preseed_migration(&mut state, &latest, true);
        assert!(state.config.contains_key(&config_path("a.cfg")));

        let mut state = AppliedState::default();
        preseed_migration(&mut state, &latest, false);
        assert!(state.config.is_empty());
    }

    #[test]
    fn normalization_sets_selective_flag() {
        let legacy = validated_archive(base_manifest(), archive::SyncArchiveFormat::Legacy, &[]);
        assert!(!normalize_archive(&legacy).unwrap().selective);

        let mut manifest = base_manifest();
        manifest.sync = Some(SyncManifest {
            version: 1,
            mods_revision: manifest_revision(&manifest).unwrap(),
            config: BTreeMap::new(),
        });
        let sync = manifest.sync.clone().unwrap();
        let selective = archive::ValidatedSyncArchive {
            manifest,
            format: archive::SyncArchiveFormat::Selective(sync),
            config: BTreeMap::new(),
        };
        assert!(normalize_archive(&selective).unwrap().selective);
    }

    #[test]
    fn apply_requires_exact_latest_revision() {
        let latest = SyncManifest {
            version: 1,
            mods_revision: ModRevision::from_hash(blake3::hash(b"rev")),
            config: BTreeMap::from([
                (
                    config_path("a.cfg"),
                    SyncFileEntry {
                        hash: hash(b"remote"),
                    },
                ),
                (
                    config_path("unselected.cfg"),
                    SyncFileEntry {
                        hash: hash(b"other"),
                    },
                ),
            ]),
        };

        let mut applied = AppliedState::default();
        applied.latest = Some(latest.clone());
        assert!(ensure_latest(&applied, &latest).is_ok());

        let mut changed = latest.clone();
        changed.mods_revision = ModRevision::from_hash(blake3::hash(b"new-rev"));
        assert!(ensure_latest(&applied, &changed).is_err());

        let mut changed = latest.clone();
        changed
            .config
            .get_mut(&config_path("unselected.cfg"))
            .unwrap()
            .hash = hash(b"changed");
        assert!(ensure_latest(&applied, &changed).is_err());
    }

    #[test]
    fn clone_rejects_unrelated_existing_profile() {
        fn sync_data(id: &str) -> SyncProfileData {
            let now = Utc::now();
            SyncProfileData {
                id: id.to_owned(),
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
            }
        }

        assert!(ensure_clone_target(None, "remote-id", "Test").is_err());

        let unrelated = sync_data("other-id");
        assert!(ensure_clone_target(Some(&unrelated), "remote-id", "Test").is_err());

        let linked = sync_data("remote-id");
        assert!(ensure_clone_target(Some(&linked), "remote-id", "Test").is_ok());
    }

    fn build_selective_archive(
        mut manifest: ProfileManifest,
        config: BTreeMap<ConfigPath, Vec<u8>>,
    ) -> archive::ValidatedSyncArchive {
        manifest.sync = Some(SyncManifest {
            version: 1,
            mods_revision: manifest_revision(&manifest).unwrap(),
            config: config
                .iter()
                .map(|(path, bytes)| (path.clone(), SyncFileEntry { hash: hash(bytes) }))
                .collect(),
        });

        let mut bytes = Vec::new();
        super::super::export::write_archive(&manifest, &config, std::io::Cursor::new(&mut bytes))
            .unwrap();
        archive::validate(&bytes).unwrap()
    }

    #[test]
    #[ignore = "requires GALE_E2E_PROFILE with a populated Valheim profile"]
    fn deep_north_selective_sync_e2e() {
        let source = env::var("GALE_E2E_PROFILE")
            .expect("GALE_E2E_PROFILE must point to a populated Valheim profile");
        let source = Path::new(&source);
        let subscriber = tempfile::tempdir().unwrap();
        let subscriber = subscriber.path();

        let corpus =
            super::super::export::collect_config_files(source, &["BepInEx/config"]).unwrap();
        assert!(
            corpus.len() >= 2,
            "expected at least 2 config files in {source:?}"
        );

        fn publish(
            subscriber: &Path,
            state: &mut AppliedState,
            manifest: ProfileManifest,
            config: BTreeMap<ConfigPath, Vec<u8>>,
        ) -> apply::ConfigApplyReport {
            let validated = build_selective_archive(manifest, config);
            let normalized = normalize_archive(&validated).unwrap();
            state.latest = Some(normalized.latest);
            apply::apply_available_config(subscriber, &normalized.config, state).unwrap()
        }

        let mut state = AppliedState::default();

        let manifest = base_manifest();
        let report = publish(subscriber, &mut state, manifest.clone(), corpus.clone());
        for (path, bytes) in &corpus {
            assert!(report.installed.contains(path), "{path} not installed");
            assert_eq!(
                std::fs::read(subscriber.join(path.as_str())).unwrap(),
                *bytes
            );
        }

        let first = corpus.keys().next().unwrap().clone();
        std::fs::write(subscriber.join(first.as_str()), b"player-custom").unwrap();

        let mut mod_only = manifest.clone();
        mod_only.mods[0].enabled = false;
        publish(subscriber, &mut state, mod_only, corpus.clone());
        assert_eq!(
            std::fs::read(subscriber.join(first.as_str())).unwrap(),
            b"player-custom"
        );
        assert!(!state.pending.contains_key(&first));

        let mut owner_v2 = corpus.clone();
        owner_v2.insert(first.clone(), b"owner-v2".to_vec());
        publish(subscriber, &mut state, manifest.clone(), owner_v2.clone());
        assert_eq!(
            std::fs::read(subscriber.join(first.as_str())).unwrap(),
            b"player-custom"
        );
        assert_eq!(state.pending[&first], PendingConfigReason::ModifiedLocally);
        let review = apply::review_items(&state);
        assert!(review.pending.iter().any(|u| u.path == first));
        assert!(!review.declined.iter().any(|u| u.path == first));

        apply::decline_selected(&mut state, &[first.clone()], false).unwrap();
        publish(subscriber, &mut state, manifest.clone(), owner_v2.clone());
        assert_eq!(
            std::fs::read(subscriber.join(first.as_str())).unwrap(),
            b"player-custom"
        );
        assert!(
            apply::review_items(&state)
                .declined
                .iter()
                .any(|u| u.path == first)
        );

        let validated = build_selective_archive(manifest.clone(), owner_v2.clone());
        let normalized = normalize_archive(&validated).unwrap();
        apply::apply_selected(
            subscriber,
            &normalized.config,
            &mut state,
            &[first.clone()],
            &[],
            false,
        )
        .unwrap();
        assert_eq!(
            std::fs::read(subscriber.join(first.as_str())).unwrap(),
            b"owner-v2"
        );
        assert!(!state.pending.contains_key(&first));
        assert!(!state.declined.contains_key(&first));
        assert!(state.config[&first].declined.is_none());

        let second = corpus.keys().nth(1).unwrap().clone();
        std::fs::remove_file(subscriber.join(second.as_str())).unwrap();

        let mut deleted_v2 = owner_v2.clone();
        deleted_v2.insert(second.clone(), b"owner-v2-deleted".to_vec());
        publish(subscriber, &mut state, manifest.clone(), deleted_v2.clone());
        assert!(!subscriber.join(second.as_str()).exists());
        assert_eq!(state.pending[&second], PendingConfigReason::DeletedLocally);

        apply::decline_selected(&mut state, &[second.clone()], false).unwrap();
        publish(subscriber, &mut state, manifest.clone(), deleted_v2.clone());
        assert!(!subscriber.join(second.as_str()).exists());
        assert!(
            apply::review_items(&state)
                .declined
                .iter()
                .any(|u| u.path == second)
        );

        let validated = build_selective_archive(manifest.clone(), deleted_v2.clone());
        let normalized = normalize_archive(&validated).unwrap();
        apply::apply_selected(
            subscriber,
            &normalized.config,
            &mut state,
            &[second.clone()],
            &[second.clone()],
            false,
        )
        .unwrap();
        assert_eq!(
            std::fs::read(subscriber.join(second.as_str())).unwrap(),
            b"owner-v2-deleted"
        );

        let game_dirs: &[&str] = &["BepInEx/config"];
        let before = apply::snapshot_config(subscriber, game_dirs).unwrap();
        let new_mod = config_path("BepInEx/config/deep-north-e2e-new-mod.cfg");
        std::fs::create_dir_all(subscriber.join("BepInEx/config")).unwrap();
        std::fs::write(subscriber.join(new_mod.as_str()), b"package-default").unwrap();
        let after = apply::snapshot_config(subscriber, game_dirs).unwrap();
        apply::record_installer_written(&before, &after, &mut state);

        let mut with_new_mod = deleted_v2.clone();
        with_new_mod.insert(new_mod.clone(), b"owner-baseline".to_vec());
        publish(
            subscriber,
            &mut state,
            manifest.clone(),
            with_new_mod.clone(),
        );
        assert_eq!(
            std::fs::read(subscriber.join(new_mod.as_str())).unwrap(),
            b"owner-baseline"
        );
        assert_eq!(
            state.config[&new_mod].applied,
            Some(hash(b"owner-baseline"))
        );

        let extra = config_path("BepInEx/config/deep-north-e2e-extra.cfg");
        std::fs::write(subscriber.join(extra.as_str()), b"extra-local").unwrap();
        publish(
            subscriber,
            &mut state,
            manifest.clone(),
            with_new_mod.clone(),
        );
        assert_eq!(
            std::fs::read(subscriber.join(extra.as_str())).unwrap(),
            b"extra-local"
        );

        let mut removed = with_new_mod.clone();
        let removed_path = removed.keys().next().unwrap().clone();
        removed.remove(&removed_path);
        let removed_target = subscriber.join(removed_path.as_str());
        assert!(removed_target.exists());
        publish(subscriber, &mut state, manifest.clone(), removed);
        assert!(removed_target.exists());
    }

    fn latest(entries: &[(&str, &[u8])]) -> SyncManifest {
        SyncManifest {
            version: 1,
            mods_revision: ModRevision::from_hash(blake3::hash(b"rev")),
            config: entries
                .iter()
                .map(|&(p, b)| (config_path(p), SyncFileEntry { hash: hash(b) }))
                .collect(),
        }
    }

    fn sync_data(id: &str, applied: AppliedState) -> SyncProfileData {
        SyncProfileData {
            id: id.to_owned(),
            owner: auth::User {
                discord_id: "1".to_owned(),
                name: "u".to_owned(),
                display_name: "u".to_owned(),
                avatar: None,
            },
            synced_at: Utc::now(),
            updated_at: Utc::now(),
            missing: false,
            published: None,
            applied: Some(applied),
        }
    }

    #[test]
    fn apply_selected_gates_writes_on_profile_identity() {
        let dir = tempfile::tempdir().unwrap();
        let p = config_path("a.cfg");
        std::fs::write(dir.path().join(p.as_str()), b"local").unwrap();

        let latest_manifest = latest(&[("a.cfg", b"A")]);
        let mut applied = AppliedState {
            latest: Some(latest_manifest.clone()),
            ..AppliedState::default()
        };
        applied
            .pending
            .insert(p.clone(), PendingConfigReason::ModifiedLocally);
        let mut sync_slot = Some(sync_data("sync-id", applied));
        let config = BTreeMap::from([(p.clone(), vfile(b"A"))]);

        let result = sync_apply_target(2, &mut sync_slot, 1, "sync-id").and_then(|sync| {
            apply_selected_and_record(
                dir.path(),
                &config,
                &latest_manifest,
                sync,
                &[p.clone()],
                &[],
                false,
                apply::write_validated,
            )
        });

        assert!(result.is_err());
        assert_eq!(
            std::fs::read(dir.path().join(p.as_str())).unwrap(),
            b"local"
        );
        assert!(
            sync_slot
                .as_ref()
                .unwrap()
                .applied
                .as_ref()
                .unwrap()
                .pending
                .contains_key(&p)
        );
    }

    #[test]
    fn apply_selected_and_record_persists_partial_state_on_write_failure() {
        let dir = tempfile::tempdir().unwrap();
        let p = config_path("a.cfg");
        let q = config_path("b.cfg");
        std::fs::write(dir.path().join(p.as_str()), b"local-a").unwrap();
        std::fs::write(dir.path().join(q.as_str()), b"local-b").unwrap();

        let latest_manifest = latest(&[("a.cfg", b"A"), ("b.cfg", b"B")]);
        let mut applied = AppliedState {
            latest: Some(latest_manifest.clone()),
            ..AppliedState::default()
        };
        applied
            .pending
            .insert(p.clone(), PendingConfigReason::ModifiedLocally);
        applied
            .pending
            .insert(q.clone(), PendingConfigReason::ModifiedLocally);
        let mut sync = sync_data("sync-id", applied);
        let config = BTreeMap::from([(p.clone(), vfile(b"A")), (q.clone(), vfile(b"B"))]);

        let mut calls = 0;
        let result = apply_selected_and_record(
            dir.path(),
            &config,
            &latest_manifest,
            &mut sync,
            &[p.clone(), q.clone()],
            &[],
            false,
            |target, file| {
                calls += 1;
                if calls == 2 {
                    Err(eyre::eyre!("controlled second-write failure"))
                } else {
                    apply::write_validated(target, file)
                }
            },
        );

        assert!(result.is_err());
        assert_eq!(std::fs::read(dir.path().join(p.as_str())).unwrap(), b"A");
        assert_eq!(
            std::fs::read(dir.path().join(q.as_str())).unwrap(),
            b"local-b"
        );

        let applied = sync.applied.as_ref().unwrap();
        assert_eq!(applied.config[&p].applied, Some(hash(b"A")));
        assert_eq!(applied.config[&p].written, Some(hash(b"A")));
        assert!(!applied.pending.contains_key(&p));
        assert_eq!(applied.pending[&q], PendingConfigReason::ModifiedLocally);

        let mut sync: SyncProfileData =
            serde_json::from_str(&serde_json::to_string(&sync).unwrap()).unwrap();
        assert_eq!(
            sync.applied.as_ref().unwrap().pending[&q],
            PendingConfigReason::ModifiedLocally
        );

        let written = apply_selected_and_record(
            dir.path(),
            &config,
            &latest_manifest,
            &mut sync,
            &[p.clone(), q.clone()],
            &[],
            false,
            apply::write_validated,
        )
        .unwrap();
        assert_eq!(written, vec![p.clone(), q.clone()]);
        assert_eq!(std::fs::read(dir.path().join(p.as_str())).unwrap(), b"A");
        assert_eq!(std::fs::read(dir.path().join(q.as_str())).unwrap(), b"B");
        let applied = sync.applied.as_ref().unwrap();
        assert_eq!(applied.config[&p].applied, Some(hash(b"A")));
        assert_eq!(applied.config[&q].applied, Some(hash(b"B")));
        assert!(applied.pending.is_empty());
    }

    fn mod_id(seed: u128) -> ModId {
        ModId {
            package_uuid: uuid::Uuid::from_u128(seed),
            version_uuid: uuid::Uuid::from_u128(seed + 0x1000),
            backend: Backend::Thunderstore,
        }
    }

    fn ts_mod(ident: &str, id: ModId, enabled: bool) -> crate::profile::ProfileMod {
        let mut profile_mod = crate::profile::ProfileMod::new(
            crate::profile::ProfileModKind::Thunderstore(crate::profile::ThunderstoreMod {
                ident: ident.parse().unwrap(),
                id,
            }),
        );
        profile_mod.enabled = enabled;
        profile_mod
    }

    fn profile_with_mods(mods: Vec<crate::profile::ProfileMod>) -> Profile {
        Profile {
            id: 0,
            name: "Test".to_owned(),
            path: Path::new("").to_owned(),
            mods,
            game: crate::game::from_slug("among-us").unwrap(),
            ignored_version_updates: Default::default(),
            ignored_package_updates: Default::default(),
            config_cache: Default::default(),
            linked_config: Default::default(),
            modpack: None,
            sync: None,
            custom_args: String::new(),
            missing: false,
        }
    }

    #[test]
    fn mod_set_matches_requires_exact_set() {
        let id = mod_id(1);
        let expected = vec![R2Mod {
            ident: PackageIdent::from(("Author", "Mod")),
            version: semver::Version::new(1, 0, 0).into(),
            enabled: true,
            source: Backend::Thunderstore,
        }];

        let profile = profile_with_mods(vec![ts_mod("Author-Mod-1.0.0", id.clone(), true)]);
        assert!(mod_set_matches(&profile, &expected));

        let profile = profile_with_mods(vec![ts_mod("Author-Mod-1.0.1", id.clone(), true)]);
        assert!(!mod_set_matches(&profile, &expected));

        let profile = profile_with_mods(vec![ts_mod("Author-Mod-1.0.0", id.clone(), false)]);
        assert!(!mod_set_matches(&profile, &expected));

        let profile = profile_with_mods(vec![
            ts_mod("Author-Mod-1.0.0", id.clone(), true),
            ts_mod("Other-Pkg-1.0.0", mod_id(2), true),
        ]);
        assert!(!mod_set_matches(&profile, &expected));

        assert!(!mod_set_matches(&profile_with_mods(vec![]), &expected));
    }
}
