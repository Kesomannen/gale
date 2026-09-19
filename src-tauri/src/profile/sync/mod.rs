use std::{
    borrow::Cow,
    collections::{BTreeMap, HashSet},
    env,
    fmt::Display,
    path::{Path, PathBuf},
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
    pub missing: bool,
    #[serde(default)]
    pub published: Option<PublishedState>,
    #[serde(default)]
    pub applied: Option<AppliedState>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PublishedState {
    /// The remote `updated_at` revision this publication snapshot represents.
    /// `None` for records written before revision tracking — treated as stale.
    #[serde(default)]
    pub revision: Option<DateTime<Utc>>,
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReviewChangedEvent {
    profile_id: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PendingConfigEvent {
    profile_id: i64,
    pending: Vec<apply::ConfigReviewItem>,
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

async fn disconnect_profile(delete: bool, profile_id: i64, app: &AppHandle) -> Result<()> {
    let (id, is_owner) = {
        let manager = app.lock_manager();
        let (_, profile) = manager.profile_by_id(profile_id)?;

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
        let (_, profile) = manager.profile_by_id_mut(profile_id)?;

        ensure!(
            profile.sync.as_ref().map(|sync| sync.id.as_str()) == Some(id.as_str()),
            "profile sync target changed during disconnect"
        );
        profile.sync = None;

        profile.save(app, true)?;
    }

    Ok(())
}

pub(super) struct NormalizedArchive<'a> {
    manifest: ProfileManifest,
    config: Cow<'a, BTreeMap<ConfigPath, archive::ValidatedConfigFile>>,
    latest: SyncManifest,
    selective: bool,
}

/// Legacy archives keep BepInEx configs under `config/`; canonical paths nest
/// them under `BepInEx/config/`.
fn normalize_legacy_path(path: &ConfigPath) -> Result<ConfigPath> {
    match path.as_str().strip_prefix("config") {
        Some(rest) if rest.is_empty() || rest.starts_with('/') => {
            ConfigPath::try_from(format!("BepInEx/config{rest}"))
        }
        _ => Ok(path.clone()),
    }
}

fn normalize_archive(archive: &archive::ValidatedSyncArchive) -> Result<NormalizedArchive<'_>> {
    let mut manifest = archive.manifest.clone();

    let (config, latest, selective) = match &archive.format {
        archive::SyncArchiveFormat::Selective(sync) => {
            (Cow::Borrowed(&archive.config), sync.clone(), true)
        }
        archive::SyncArchiveFormat::Legacy => {
            let mut config: BTreeMap<ConfigPath, archive::ValidatedConfigFile> = BTreeMap::new();
            let mut seen = HashSet::new();
            for (path, file) in &archive.config {
                let normalized = normalize_legacy_path(path)?;
                ensure!(
                    seen.insert(normalized.as_str().to_ascii_lowercase()),
                    "config paths collide after legacy normalization: {normalized}"
                );
                config.insert(normalized, file.clone());
            }

            let latest = SyncManifest {
                version: 1,
                mods_revision: archive.mods_revision()?,
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

fn preseed_migration(state: &mut AppliedState, latest: &SyncManifest) {
    for path in latest.config.keys() {
        state.config.entry(path.clone()).or_default();
    }
}

/// Config snapshots taken around the mod install: `before` the update and
/// `after` it, so installer-written files can be told apart from user edits.
type InstallSnapshots<'a> = (
    &'a BTreeMap<ConfigPath, ContentHash>,
    &'a BTreeMap<ConfigPath, ContentHash>,
);

/// Applies the config side of a sync archive to `applied`, mutating the state
/// and the profile's config files together.
///
/// `install` carries the config snapshots taken around a mod install.
/// `migrate_existing` seeds empty records for a profile that predates
/// selective sync, so its first pull doesn't ask about untouched files.
///
/// Partial progress stays in `applied` when a file fails, so the caller must
/// persist it even on error.
fn apply_to_state(
    applied: &mut AppliedState,
    profile_dir: &Path,
    config: &BTreeMap<ConfigPath, archive::ValidatedConfigFile>,
    latest: SyncManifest,
    install: Option<InstallSnapshots<'_>>,
    migrate_existing: bool,
) -> Result<apply::ConfigApplyReport> {
    if let Some((before, after)) = install {
        apply::record_installer_written(before, after, applied);
        applied.mods_revision = Some(latest.mods_revision.clone());
    }

    if migrate_existing {
        preseed_migration(applied, &latest);
    }

    apply::preserve_pending_policy_boundaries(applied);
    applied.latest = Some(latest);
    apply::apply_available_config(profile_dir, config, applied)
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

/// The local destination a sync archive resolves to under the manager lock:
/// the profile, its previous sync link and its directory, if one exists.
struct ResolvedTarget {
    /// Set only for [`ImportTarget::Existing`]; named targets re-resolve by
    /// name inside the import.
    profile_id: Option<i64>,
    prior_sync: Option<SyncProfileData>,
    dir: Option<PathBuf>,
    game: crate::game::Game,
}

fn resolve_target(
    target: &ImportTarget,
    manifest_name: &str,
    app: &AppHandle,
) -> Result<ResolvedTarget> {
    let manager = app.lock_manager();

    match target {
        ImportTarget::Existing(id) => {
            let (game, profile) = manager.profile_by_id(*id)?;
            Ok(ResolvedTarget {
                profile_id: Some(profile.id),
                prior_sync: profile.sync.clone(),
                dir: Some(profile.path.clone()),
                game,
            })
        }
        ImportTarget::Named { game } => {
            let resolved = manager
                .games
                .get(game)
                .and_then(|game| game.find_profile_index(manifest_name))
                .map(|index| &manager.games[game].profiles[index]);

            Ok(ResolvedTarget {
                profile_id: None,
                prior_sync: resolved.and_then(|profile| profile.sync.clone()),
                dir: resolved.map(|profile| profile.path.clone()),
                game: *game,
            })
        }
    }
}

async fn apply_archive(
    normalized: NormalizedArchive<'_>,
    metadata: SyncProfileMetadata,
    override_name: Option<String>,
    clone: bool,
    target: ImportTarget,
    app: &AppHandle,
) -> Result<apply::ConfigApplyReport> {
    let mut manifest = normalized.manifest.clone();
    if let Some(name) = override_name {
        manifest.name = name;
    }
    let latest = normalized.latest.clone();

    let resolved = resolve_target(&target, &manifest.name, app)?;

    if clone && resolved.profile_id.is_none() && resolved.dir.is_some() {
        ensure_clone_target(resolved.prior_sync.as_ref(), &metadata.id, &manifest.name)?;
    }

    let prior_sync_id = resolved.prior_sync.as_ref().map(|sync| sync.id.clone());
    let was_owner = resolved
        .prior_sync
        .as_ref()
        .is_some_and(|sync| sync.published.is_some());

    let needs_install = clone
        || resolved
            .prior_sync
            .as_ref()
            .and_then(|sync| sync.applied.as_ref())
            .and_then(|applied| applied.mods_revision.as_ref())
            != Some(&latest.mods_revision);

    let before = if needs_install && resolved.dir.is_some() {
        apply::snapshot_config(
            resolved.dir.as_ref().unwrap(),
            resolved.game.mod_loader.mod_config_dirs(),
        )?
    } else {
        BTreeMap::new()
    };

    let mut install = if needs_install {
        super::import::resolve_manifest_sources(&mut manifest, &app.lock_thunderstore());
        let expected_mods = normalized.selective.then(|| manifest.mods.clone());

        let imported = super::import::import_manifest(
            manifest,
            target,
            ImportOptions::default().ignore_missing_mods(!normalized.selective),
            InstallOptions::default(),
            app,
        )
        .await
        .context("failed to import synced profile")?;

        Some((imported, expected_mods))
    } else {
        None
    };

    let target_id = install
        .as_ref()
        .map(|(imported, _)| imported.id)
        .or(resolved.profile_id)
        .ok_or_eyre("sync target resolved to no profile")?;
    let profile_dir = install
        .as_ref()
        .map(|(imported, _)| imported.path.clone())
        .or_else(|| resolved.dir.clone())
        .ok_or_eyre("sync target resolved to no directory")?;

    // Serialize the owner's snapshot adoption against local publishes. The
    // guard is taken before queue/manager locks so the order matches
    // publish_profile's publication -> manager ordering.
    let _publication = if was_owner {
        Some(publish::publication_guard().await)
    } else {
        None
    };

    // keep the backed-up originals until the new state is committed and
    // verified; restored below if the commit fails
    let mut pending_revert = install
        .as_mut()
        .and_then(|(imported, _)| imported.revert.take());

    let result = (|| -> Result<apply::ConfigApplyReport> {
        if let (Some(id), Some((imported, _))) = (resolved.profile_id, install.as_ref()) {
            ensure!(imported.id == id, "synced profile changed during apply");
        }

        let after = install
            .as_ref()
            .map(|(imported, _)| {
                apply::snapshot_config(&imported.path, imported.game.mod_loader.mod_config_dirs())
            })
            .transpose()?;
        let install_snapshots = after.as_ref().map(|after| (&before, after));

        let published = if was_owner {
            Some(publish::adopt_publication(
                &profile_dir,
                &normalized,
                Some(metadata.updated_at),
            )?)
        } else {
            None
        };

        // hold the queue lock through the commit so no foreign install for
        // this profile can be queued or start between the checks and the
        // state write (queue -> manager is the established lock order)
        let queue = app.install_queue();
        let queue = queue.lock();
        ensure!(
            !queue.has_any_for_profile(target_id),
            "another install is queued for this profile"
        );

        let mut manager = app.lock_manager();
        let (_, profile) = manager.profile_by_id_mut(target_id)?;

        let current_sync_id = profile.sync.as_ref().map(|sync| sync.id.clone());
        ensure!(
            current_sync_id == prior_sync_id,
            "profile sync target changed during apply"
        );

        if let Some((_, Some(expected))) = &install {
            ensure!(
                mod_set_matches(profile, expected),
                "installed mod set does not match the synced manifest"
            );
        }

        // clone fresh inside the lock so decisions made while the pull was
        // running (declines, policies) aren't overwritten
        let migrate_existing = resolved.dir.is_some()
            && profile
                .sync
                .as_ref()
                .is_none_or(|sync| sync.applied.is_none());
        let mut applied = profile
            .sync
            .as_ref()
            .and_then(|sync| sync.applied.clone())
            .unwrap_or_default();

        match apply_to_state(
            &mut applied,
            &profile_dir,
            &normalized.config,
            latest,
            install_snapshots,
            migrate_existing,
        ) {
            Ok(report) => {
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
                Ok(report)
            }
            Err(err) => {
                // persist partial progress and the adopted publication so a
                // retry resumes from accurate state
                let sync = profile
                    .sync
                    .get_or_insert_with(|| SyncProfileData::from(metadata.clone()));
                sync.applied = Some(applied);
                if published.is_some() {
                    sync.published = published;
                }
                profile.save(app, true)?;
                Err(err)
            }
        }
    })();

    // the install succeeded but the commit didn't; put the original mods back.
    // if restoration itself fails the backups stay in the revert dir
    let result = match result {
        Err(err) => match pending_revert.take() {
            Some(revert) => match super::import::restore_imported_profile(target_id, revert, app) {
                Ok(()) => Err(err),
                Err(restore_err) => Err(err.wrap_err(format!(
                    "failed to restore the previous mod set; \
                     backed-up files are preserved in {}: {restore_err:#}",
                    super::import::revert_dir(&profile_dir).display()
                ))),
            },
            None => Err(err),
        },
        Ok(report) => Ok(report),
    };

    // review state may have changed even when the apply itself failed
    app.emit_buffered(
        "sync_config_review_changed",
        &ReviewChangedEvent {
            profile_id: target_id,
        },
    );

    let report = match result {
        Ok(report) => report,
        Err(err) => {
            if install
                .as_ref()
                .is_some_and(|(imported, _)| imported.created)
            {
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

    // the pulled state is committed and verified; the backed-up originals
    // are no longer needed
    super::import::clear_revert_dir(&profile_dir);

    if !report.pending.is_empty() {
        app.emit_buffered(
            "sync_config_pending",
            &PendingConfigEvent {
                profile_id: target_id,
                pending: report.pending.clone(),
            },
        );
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

pub async fn pull_profile(
    dry_run: bool,
    profile_id: i64,
    app: &AppHandle,
) -> Result<apply::ConfigApplyReport> {
    let (id, name, synced_at) = {
        let manager = app.lock_manager();
        let (_, profile) = manager.profile_by_id(profile_id)?;

        match &profile.sync {
            Some(data) if data.missing => bail!("cannot pull from missing profile"),
            Some(data) => (data.id.clone(), profile.name.clone(), data.synced_at),
            None => return Ok(apply::ConfigApplyReport::default()),
        }
    };

    let metadata = get_profile_meta(&id, app).await?;

    match metadata {
        Some(metadata) if !dry_run && metadata.updated_at > synced_at => {
            let bytes = download_profile_bytes(&id, app).await?;
            let validated = archive::validate(&bytes).context("sync archive failed validation")?;
            let normalized = normalize_archive(&validated)?;

            apply_archive(
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
                return Ok(apply::ConfigApplyReport::default());
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

            Ok(apply::ConfigApplyReport::default())
        }
    }
}

fn pending_config_items(profile_id: i64, app: &AppHandle) -> Result<apply::ConfigReviewState> {
    let mut manager = app.lock_manager();
    let (_, profile) = manager.profile_by_id_mut(profile_id)?;

    let Some(applied) = profile.sync.as_ref().and_then(|sync| sync.applied.as_ref()) else {
        return Ok(apply::ConfigReviewState::default());
    };

    // a file hand-edited to match the published revision is no longer pending
    let mut applied = applied.clone();
    if apply::reconcile_review_state(&profile.path, &mut applied)? {
        profile.sync.as_mut().unwrap().applied = Some(applied.clone());
        profile.save(app, true)?;
    }

    apply::current_review_items(&profile.path, &applied)
}

/// Clones the profile's applied sync state, runs `update` on the clone, and
/// writes it back only if the profile is still linked to the same sync id.
fn update_applied(
    profile_id: i64,
    app: &AppHandle,
    update: impl FnOnce(&mut AppliedState) -> Result<()>,
) -> Result<()> {
    let (sync_id, mut applied) = {
        let manager = app.lock_manager();
        let (_, profile) = manager.profile_by_id(profile_id)?;
        let sync = profile.sync.as_ref().ok_or_eyre("profile is not synced")?;
        let applied = sync.applied.clone().ok_or_eyre("no applied sync state")?;
        (sync.id.clone(), applied)
    };

    update(&mut applied)?;

    let mut manager = app.lock_manager();
    let (_, profile) = manager.profile_by_id_mut(profile_id)?;
    let sync = sync_apply_target(&mut profile.sync, &sync_id)?;
    sync.applied = Some(applied);
    profile.save(app, true)
}

fn decline_selected_config(
    files: &[ConfigPath],
    remember: bool,
    profile_id: i64,
    app: &AppHandle,
) -> Result<()> {
    update_applied(profile_id, app, |applied| {
        apply::decline_selected(applied, files, remember)
    })
}

fn set_config_policy(
    file: ConfigPath,
    policy: ConfigUpdatePolicy,
    profile_id: i64,
    app: &AppHandle,
) -> Result<()> {
    update_applied(profile_id, app, |applied| {
        apply::set_policy(applied, &file, policy)
    })
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

fn sync_apply_target<'a>(
    sync: &'a mut Option<SyncProfileData>,
    expected_sync_id: &str,
) -> Result<&'a mut SyncProfileData> {
    let Some(sync) = sync.as_mut() else {
        bail!("profile is no longer synced");
    };
    ensure!(
        sync.id == expected_sync_id,
        "profile sync target changed during apply"
    );
    Ok(sync)
}

/// Runs `apply::apply_selected` on a clone of the sync's applied state and
/// writes the clone back even when the apply fails part-way, so progress and
/// decisions made before the failure survive.
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
    ensure!(
        applied.latest.as_ref() == Some(latest),
        "published sync revision changed; pull the latest update before applying config"
    );
    let result = apply::apply_selected_with(
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
    profile_id: i64,
    app: &AppHandle,
) -> Result<Vec<ConfigPath>> {
    let sync_id = {
        let manager = app.lock_manager();
        let (_, profile) = manager.profile_by_id(profile_id)?;
        let sync = profile.sync.as_ref().ok_or_eyre("profile is not synced")?;
        sync.id.clone()
    };

    let bytes = download_profile_bytes(&sync_id, app).await?;
    let validated = archive::validate(&bytes).context("sync archive failed validation")?;
    let normalized = normalize_archive(&validated)?;

    let mut manager = app.lock_manager();
    let (_, profile) = manager.profile_by_id_mut(profile_id)?;
    let profile_dir = profile.path.clone();
    let sync = sync_apply_target(&mut profile.sync, &sync_id)?;

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

    // review state may have changed even when the apply itself failed
    app.emit_buffered(
        "sync_config_review_changed",
        &ReviewChangedEvent { profile_id },
    );

    apply_result
}

/// Sync archives are a manifest plus text config files; anything larger is
/// malformed or hostile.
const MAX_DOWNLOAD_BYTES: usize = 16 * 1024 * 1024;

pub(super) async fn download_profile_bytes(id: &str, app: &AppHandle) -> Result<Vec<u8>> {
    let mut response = request(Method::GET, format!("/profile/{id}"), app)
        .await
        .send()
        .await?
        .error_for_status()?;

    if let Some(len) = response.content_length() {
        ensure!(
            len <= MAX_DOWNLOAD_BYTES as u64,
            "sync archive exceeds the download size limit"
        );
    }

    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        ensure!(
            bytes.len() + chunk.len() <= MAX_DOWNLOAD_BYTES,
            "sync archive exceeds the download size limit"
        );
        bytes.extend_from_slice(&chunk);
    }

    Ok(bytes)
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
        profile::export::{R2Mod, manifest_revision},
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
    fn migration_preseed_creates_empty_records() {
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
        preseed_migration(&mut state, &latest);
        assert!(state.config.contains_key(&config_path("a.cfg")));
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
        let dir = tempfile::tempdir().unwrap();
        let latest = latest(&[("a.cfg", b"A"), ("unselected.cfg", b"other")]);
        let config = BTreeMap::from([
            (config_path("a.cfg"), vfile(b"A")),
            (config_path("unselected.cfg"), vfile(b"other")),
        ]);

        let applied = AppliedState {
            latest: Some(latest.clone()),
            ..AppliedState::default()
        };
        let mut sync = sync_data("sync-id", applied);

        let apply = |latest: &SyncManifest, sync: &mut SyncProfileData| {
            apply_selected_and_record(
                dir.path(),
                &config,
                latest,
                sync,
                &[],
                &[],
                false,
                apply::write_validated,
            )
        };

        assert!(apply(&latest, &mut sync).is_ok());

        let mut changed = latest.clone();
        changed.mods_revision = ModRevision::from_hash(blake3::hash(b"new-rev"));
        assert!(apply(&changed, &mut sync).is_err());

        let mut changed = latest.clone();
        changed
            .config
            .get_mut(&config_path("unselected.cfg"))
            .unwrap()
            .hash = hash(b"changed");
        assert!(apply(&changed, &mut sync).is_err());
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

        let result = sync_apply_target(&mut sync_slot, "other-sync").and_then(|sync| {
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
            server_settings: None,
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

    #[test]
    fn apply_to_state_persists_partial_progress() {
        let dir = tempfile::tempdir().unwrap();
        let p = config_path("a.cfg");
        let q = config_path("b.cfg");

        // obstruct b.cfg's target so its write fails after a.cfg succeeded
        std::fs::create_dir_all(dir.path().join("b.cfg")).unwrap();

        let latest_manifest = latest(&[("a.cfg", b"A"), ("b.cfg", b"B")]);
        let config = BTreeMap::from([(p.clone(), vfile(b"A")), (q.clone(), vfile(b"B"))]);

        let mut applied = AppliedState::default();
        let result = apply_to_state(
            &mut applied,
            dir.path(),
            &config,
            latest_manifest.clone(),
            None,
            false,
        );

        assert!(result.is_err());
        // a.cfg was written and recorded before b.cfg failed
        assert_eq!(std::fs::read(dir.path().join("a.cfg")).unwrap(), b"A");
        assert_eq!(applied.config[&p].applied, Some(hash(b"A")));
        assert_eq!(applied.latest, Some(latest_manifest.clone()));

        // retrying after removing the obstruction applies b.cfg
        std::fs::remove_dir(dir.path().join("b.cfg")).unwrap();
        let report = apply_to_state(
            &mut applied,
            dir.path(),
            &config,
            latest_manifest,
            None,
            false,
        )
        .unwrap();

        assert_eq!(std::fs::read(dir.path().join("b.cfg")).unwrap(), b"B");
        assert_eq!(applied.config[&q].applied, Some(hash(b"B")));
        assert_eq!(report.installed, vec![q]);
    }
}
