use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fs::File,
    io::{self, BufReader},
    path::{Component, Path, PathBuf},
    thread,
    time::Duration,
};

use eyre::{Context, Result, bail, ensure};
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tracing::{info, warn};
use walkdir::{DirEntry, WalkDir};

use super::{
    config::RemoteServerSettings,
    manifest::{self, DeploymentManifest, ManifestEntry},
    remote::{self, ConnectionAttempt, RemoteConnection},
};
use crate::{
    game::{Game, mod_loader::ServerDeployment},
    profile::Profile,
    state::ManagerExt,
    thunderstore::Thunderstore,
};

const MAX_MANIFEST_BYTES: u64 = 8 * 1024 * 1024;
const TRANSFER_ATTEMPTS: usize = 3;
const RETRY_DELAY: Duration = Duration::from_millis(500);

#[derive(Debug, Clone, Serialize)]
#[serde(
    tag = "status",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum DeploymentResult {
    HostKeyUntrusted {
        fingerprint: String,
    },
    CertificateUntrusted,
    Deployed {
        uploaded_files: usize,
        uploaded_bytes: u64,
        removed_files: usize,
        unchanged_files: usize,
        skipped_client_only_mods: Vec<String>,
        cleanup_warnings: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(
    tag = "status",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum DeploymentPreviewResult {
    HostKeyUntrusted {
        fingerprint: String,
    },
    CertificateUntrusted,
    Preview {
        upload_files: Vec<String>,
        upload_file_sizes: BTreeMap<String, u64>,
        remove_files: Vec<String>,
        preserved_files: usize,
        unchanged_files: usize,
        skipped_client_only_mods: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentProgress {
    pub completed: usize,
    pub total: usize,
    pub path: String,
    pub operation: DeploymentOperation,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DeploymentOperation {
    Upload,
    Remove,
}

#[derive(Default)]
pub struct ClientOnlyMods {
    names: Vec<String>,
    paths: HashSet<PathBuf>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentSelection {
    pub upload_files: Vec<String>,
    pub remove_files: Vec<String>,
}

struct LocalFile {
    path: PathBuf,
    manifest: ManifestEntry,
}

struct LocalDeployment {
    files: BTreeMap<PathBuf, LocalFile>,
}

struct DeploymentPlan {
    current: DeploymentManifest,
    tracked_removals: HashMap<PathBuf, ManifestEntry>,
    uploads: Vec<PathBuf>,
    upload_bytes: u64,
    removals: Vec<PathBuf>,
    directory_removals: Vec<PathBuf>,
    preserved_files: Vec<PathBuf>,
    unchanged_files: usize,
}

struct DeploymentPolicy {
    loader_root: PathBuf,
    managed_directories: Vec<PathBuf>,
    preserve_untracked_directories: Vec<PathBuf>,
    manifest_directory: PathBuf,
    excluded_files: GlobSet,
}

struct ServerFileFilter<'a> {
    policy: &'a DeploymentPolicy,
    client_only: &'a ClientOnlyMods,
    preserve_loader: bool,
}

#[derive(Clone)]
struct RemoteLayout {
    base: PathBuf,
    loader_root: PathBuf,
    manifest_directory: PathBuf,
    loader_at_root: bool,
}

struct DeploymentSession<'a> {
    connection: &'a mut RemoteConnection,
    settings: &'a RemoteServerSettings,
    password: &'a str,
    policy: DeploymentPolicy,
    layout: RemoteLayout,
    server_has_loader: bool,
    ensured_directories: HashSet<PathBuf>,
}

impl ClientOnlyMods {
    pub fn from_profile(profile: &Profile, thunderstore: &Thunderstore) -> Self {
        let Some(server) = &profile.game.dedicated_server else {
            return Self::default();
        };

        let mut result = Self::default();

        for (profile_mod, enabled) in profile.thunderstore_mods() {
            if !enabled {
                continue;
            }

            let Ok(package) = profile_mod.id.borrow(thunderstore).map(|item| item.package) else {
                continue;
            };
            let client_only = server.client_only_categories.iter().any(|expected| {
                package
                    .categories
                    .iter()
                    .any(|category| category.eq_ignore_ascii_case(expected))
            }) && !server.server_categories.iter().any(|expected| {
                package
                    .categories
                    .iter()
                    .any(|category| category.eq_ignore_ascii_case(expected))
            });

            if !client_only {
                continue;
            }

            let name = profile_mod.ident.full_name().to_owned();
            let installer = profile.game.mod_loader.installer_for(&name);
            match installer.installed_paths(&name, profile) {
                Ok(paths) => result.paths.extend(paths.into_iter().filter_map(|path| {
                    path.strip_prefix(&profile.path).ok().map(Path::to_path_buf)
                })),
                Err(error) => warn!(%error, package = %name, "failed to read installed mod files"),
            }

            result.names.push(name);
        }

        result.names.sort();
        result
    }

    fn excludes(&self, path: &Path) -> bool {
        self.paths.iter().any(|owned| path.starts_with(owned))
    }
}

impl DeploymentPolicy {
    fn for_game(game: Game) -> Result<Self> {
        let deployment = game.mod_loader.server_deployment().ok_or_else(|| {
            eyre::eyre!(
                "remote deployment is unsupported for {}",
                game.mod_loader.as_str()
            )
        })?;

        Self::from_config(deployment)
    }

    fn from_config(config: ServerDeployment) -> Result<Self> {
        let mut excluded_files = GlobSetBuilder::new();
        for pattern in config.excluded_globs {
            excluded_files.add(
                GlobBuilder::new(pattern)
                    .literal_separator(true)
                    .case_insensitive(true)
                    .build()?,
            );
        }

        Ok(Self {
            loader_root: config.loader_root,
            managed_directories: config.managed_directories,
            preserve_untracked_directories: config.preserve_untracked_directories,
            manifest_directory: config.manifest_directory,
            excluded_files: excluded_files.build()?,
        })
    }

    fn manages(&self, path: &Path) -> bool {
        self.managed_directories
            .iter()
            .any(|directory| path.starts_with(directory))
    }

    fn preserves_untracked(&self, path: &Path) -> bool {
        self.preserve_untracked_directories
            .iter()
            .any(|directory| path.starts_with(directory))
    }

    fn preserve_host_loader(&self, previous: &DeploymentManifest, server_has_loader: bool) -> bool {
        server_has_loader && previous.files.keys().all(|path| self.manages(path))
    }
}

impl RemoteLayout {
    fn path(&self, relative: &Path) -> Result<PathBuf> {
        ensure!(
            !relative.as_os_str().is_empty() && crate::util::fs::is_enclosed(relative),
            "remote path escaped the server directory"
        );

        let relative = if self.loader_at_root && relative.starts_with(&self.loader_root) {
            relative.strip_prefix(&self.loader_root)?
        } else {
            relative
        };

        Ok(self.base.join(relative))
    }

    fn manifest_path(&self) -> Result<PathBuf> {
        if self.loader_at_root {
            join_name(
                &self.path(&self.manifest_directory)?,
                Path::new(manifest::FILE_NAME),
            )
        } else {
            join_name(&self.base, Path::new(manifest::FILE_NAME))
        }
    }
}

impl<'a> DeploymentSession<'a> {
    fn new(
        connection: &'a mut RemoteConnection,
        settings: &'a RemoteServerSettings,
        password: &'a str,
        game: Game,
    ) -> Result<Self> {
        connection
            .check_directory(&settings.server_directory)
            .with_context(|| {
                format!(
                    "remote server directory '{}' could not be accessed",
                    settings.server_directory.display()
                )
            })?;

        let policy = DeploymentPolicy::for_game(game)?;
        let standard_loader = settings.server_directory.join(&policy.loader_root);
        let standard_layout = connection
            .directory_exists(&standard_loader)
            .context("failed to check for a server-managed mod loader")?;
        let mut loader_at_root = false;

        if !standard_layout {
            for directory in &policy.managed_directories {
                let relative = directory.strip_prefix(&policy.loader_root)?;
                let path = settings.server_directory.join(relative);

                if connection
                    .directory_exists(&path)
                    .context("failed to check for a restricted mod-loader layout")?
                {
                    loader_at_root = true;
                    break;
                }
            }
        }

        Ok(Self {
            connection,
            settings,
            password,
            layout: RemoteLayout {
                base: settings.server_directory.clone(),
                loader_root: policy.loader_root.clone(),
                manifest_directory: policy.manifest_directory.clone(),
                loader_at_root,
            },
            policy,
            server_has_loader: standard_layout || loader_at_root,
            ensured_directories: HashSet::new(),
        })
    }

    fn prepare(
        &mut self,
        profile_dir: &Path,
        client_only: &ClientOnlyMods,
        selection: Option<&DeploymentSelection>,
    ) -> Result<(LocalDeployment, DeploymentPlan)> {
        let previous = self.read_manifest()?;
        ensure!(
            previous.version == manifest::VERSION,
            "unsupported remote Gale manifest version {}",
            previous.version
        );

        let filter = ServerFileFilter::new(
            &self.policy,
            client_only,
            self.policy
                .preserve_host_loader(&previous, self.server_has_loader),
        );
        let local = collect_local_files(profile_dir, &filter)?;
        let (remote_files, remote_directories) = self.collect_remote_state(&previous)?;
        let plan = build_plan(
            &local,
            previous,
            remote_files,
            remote_directories,
            &self.policy,
            selection,
        );

        info!(
            uploads = plan.uploads.len(),
            removals = plan.removals.len(),
            unchanged = plan.unchanged_files,
            upload_bytes = plan.upload_bytes,
            "prepared remote deployment"
        );

        Ok((local, plan))
    }

    fn collect_remote_state(
        &mut self,
        previous: &DeploymentManifest,
    ) -> Result<(BTreeSet<PathBuf>, BTreeSet<PathBuf>)> {
        let mut files = BTreeSet::new();
        let mut directories = BTreeSet::new();

        for directory in self.policy.managed_directories.clone() {
            self.collect_remote_directory(&directory, &mut files, &mut directories)?;
        }

        for path in previous
            .files
            .keys()
            .filter(|path| !self.policy.manages(path))
        {
            if self.connection.file_exists(&self.layout.path(path)?)? {
                files.insert(path.clone());
            }
        }

        Ok((files, directories))
    }

    fn collect_remote_directory(
        &mut self,
        directory: &Path,
        files: &mut BTreeSet<PathBuf>,
        directories: &mut BTreeSet<PathBuf>,
    ) -> Result<()> {
        let remote_directory = self.layout.path(directory)?;
        let manifest_path = join_name(
            &self.policy.manifest_directory,
            Path::new(manifest::FILE_NAME),
        )?;
        let temporary_manifest = append_suffix(&manifest_path, ".tmp");

        for entry in self
            .connection
            .list_directory_entries(&remote_directory)
            .with_context(|| {
                format!(
                    "failed to inspect remote directory {}",
                    remote_directory.display()
                )
            })?
        {
            let relative = join_name(directory, &entry.name)?;

            if entry.is_directory {
                directories.insert(relative.clone());
                self.collect_remote_directory(&relative, files, directories)?;
            } else if relative != manifest_path && relative != temporary_manifest {
                files.insert(relative);
            }
        }

        Ok(())
    }

    fn apply<F>(
        &mut self,
        local: &LocalDeployment,
        plan: &DeploymentPlan,
        mut report: F,
    ) -> Result<(usize, Vec<String>)>
    where
        F: FnMut(DeploymentProgress),
    {
        let total = plan.uploads.len() + plan.removals.len() + plan.directory_removals.len();
        let mut completed = 0;
        let mut removed_files = 0;
        let mut cleanup_warnings = Vec::new();
        let mut manifest = plan.current.clone();

        for relative in &plan.removals {
            match self.connection.remove_file(&self.layout.path(relative)?) {
                Ok(true) => removed_files += 1,
                Ok(false) => {}
                Err(error) => {
                    let warning = format!(
                        "Could not remove {}: {error}",
                        display_remote_path(relative)
                    );
                    warn!("{warning}");
                    cleanup_warnings.push(warning);

                    if let Some(entry) = plan.tracked_removals.get(relative) {
                        manifest.files.insert(relative.clone(), entry.clone());
                    }
                }
            }

            completed += 1;
            report(progress(
                completed,
                total,
                relative,
                DeploymentOperation::Remove,
            ));
        }

        for relative in &plan.directory_removals {
            if let Err(error) = self
                .connection
                .remove_directory(&self.layout.path(relative)?)
            {
                let warning = format!(
                    "Could not remove {}/: {error}",
                    display_remote_path(relative)
                );
                warn!("{warning}");
                cleanup_warnings.push(warning);
            }

            completed += 1;
            report(DeploymentProgress {
                completed,
                total,
                path: format!("{}/", display_remote_path(relative)),
                operation: DeploymentOperation::Remove,
            });
        }

        for relative in &plan.uploads {
            self.upload_with_retry(relative, &local.files[relative])?;

            completed += 1;
            report(progress(
                completed,
                total,
                relative,
                DeploymentOperation::Upload,
            ));
        }

        self.write_manifest_with_retry(&manifest)?;

        Ok((removed_files, cleanup_warnings))
    }

    fn upload_with_retry(&mut self, relative: &Path, file: &LocalFile) -> Result<()> {
        let target = self.layout.path(relative)?;
        let temporary = append_suffix(&target, ".gale-upload");
        let base = self.layout.base.clone();
        let settings = self.settings;
        let password = self.password;
        let ensured_directories = &mut self.ensured_directories;

        transfer_with_retry(
            self.connection,
            settings,
            password,
            relative,
            |connection| {
                upload_file(
                    connection,
                    &base,
                    &target,
                    &temporary,
                    relative,
                    file,
                    ensured_directories,
                )
            },
        )
    }

    fn write_manifest_with_retry(&mut self, manifest: &DeploymentManifest) -> Result<()> {
        let target = self.layout.manifest_path()?;
        let temporary = append_suffix(&target, ".tmp");
        let base = self.layout.base.clone();
        let settings = self.settings;
        let password = self.password;
        let ensured_directories = &mut self.ensured_directories;

        transfer_with_retry(
            self.connection,
            settings,
            password,
            Path::new(manifest::FILE_NAME),
            |connection| {
                ensure_remote_parents(connection, &base, &target, ensured_directories)?;
                write_remote_manifest(connection, &target, &temporary, manifest)
            },
        )
    }

    fn read_manifest(&mut self) -> Result<DeploymentManifest> {
        let path = self.layout.manifest_path()?;
        let bytes = match self.connection.read_file(&path)? {
            Some(bytes) => bytes,
            None => return Ok(DeploymentManifest::default()),
        };

        ensure!(
            bytes.len() as u64 <= MAX_MANIFEST_BYTES,
            "remote Gale manifest is unexpectedly large"
        );

        serde_json::from_slice(&bytes).context("remote Gale manifest is invalid")
    }
}

impl<'a> ServerFileFilter<'a> {
    fn new(
        policy: &'a DeploymentPolicy,
        client_only: &'a ClientOnlyMods,
        preserve_loader: bool,
    ) -> Self {
        Self {
            policy,
            client_only,
            preserve_loader,
        }
    }

    fn excludes(&self, relative: &Path) -> bool {
        if self.client_only.excludes(relative) || self.policy.excluded_files.is_match(relative) {
            return true;
        }

        if !self.preserve_loader {
            return false;
        }

        !self
            .policy
            .managed_directories
            .iter()
            .any(|directory| relative.starts_with(directory) || directory.starts_with(relative))
    }
}

pub fn active_profile(app: &AppHandle) -> (i64, PathBuf, Game, ClientOnlyMods) {
    let manager = app.lock_manager();
    let thunderstore = app.lock_thunderstore();
    let profile = manager.active_profile();

    (
        profile.id,
        profile.path.clone(),
        profile.game,
        ClientOnlyMods::from_profile(profile, &thunderstore),
    )
}

pub fn preview(
    profile_dir: &Path,
    game: Game,
    client_only: ClientOnlyMods,
    settings: &RemoteServerSettings,
    password: &str,
) -> Result<DeploymentPreviewResult> {
    let mut connection = match remote::connect(settings, password)? {
        ConnectionAttempt::Connected(connection) => connection,
        ConnectionAttempt::HostKeyUntrusted { fingerprint } => {
            return Ok(DeploymentPreviewResult::HostKeyUntrusted { fingerprint });
        }
        ConnectionAttempt::CertificateUntrusted => {
            return Ok(DeploymentPreviewResult::CertificateUntrusted);
        }
    };

    let mut session = DeploymentSession::new(&mut connection, settings, password, game)?;
    let (local, plan) = session.prepare(profile_dir, &client_only, None)?;

    Ok(DeploymentPreviewResult::Preview {
        upload_file_sizes: plan
            .uploads
            .iter()
            .map(|path| (display_remote_path(path), local.files[path].manifest.size))
            .collect(),
        upload_files: plan
            .uploads
            .iter()
            .map(|path| display_remote_path(path))
            .collect(),
        remove_files: plan
            .removals
            .iter()
            .map(|path| display_remote_path(path))
            .collect(),
        preserved_files: plan.preserved_files.len(),
        unchanged_files: plan.unchanged_files,
        skipped_client_only_mods: client_only.names,
    })
}

pub fn deploy<F>(
    profile_dir: &Path,
    game: Game,
    client_only: ClientOnlyMods,
    settings: &RemoteServerSettings,
    password: &str,
    selection: &DeploymentSelection,
    report: F,
) -> Result<DeploymentResult>
where
    F: FnMut(DeploymentProgress),
{
    let mut connection = match remote::connect(settings, password)? {
        ConnectionAttempt::Connected(connection) => connection,
        ConnectionAttempt::HostKeyUntrusted { fingerprint } => {
            return Ok(DeploymentResult::HostKeyUntrusted { fingerprint });
        }
        ConnectionAttempt::CertificateUntrusted => {
            return Ok(DeploymentResult::CertificateUntrusted);
        }
    };

    let mut session = DeploymentSession::new(&mut connection, settings, password, game)?;
    let (local, plan) = session.prepare(profile_dir, &client_only, Some(selection))?;
    let (removed_files, cleanup_warnings) = session.apply(&local, &plan, report)?;

    Ok(DeploymentResult::Deployed {
        uploaded_files: plan.uploads.len(),
        uploaded_bytes: plan.upload_bytes,
        removed_files,
        unchanged_files: plan.unchanged_files,
        skipped_client_only_mods: client_only.names,
        cleanup_warnings,
    })
}

fn collect_local_files(
    profile_dir: &Path,
    filter: &ServerFileFilter<'_>,
) -> Result<LocalDeployment> {
    let mut files = BTreeMap::new();

    for entry in WalkDir::new(profile_dir)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| !is_excluded(entry, profile_dir, filter))
    {
        let entry = entry.context("failed to enumerate profile files")?;

        if !entry.file_type().is_file() {
            continue;
        }

        let relative = entry
            .path()
            .strip_prefix(profile_dir)
            .context("profile file escaped its root")?;
        let relative = manifest::normalize_relative_path(relative)?;
        let metadata = entry
            .metadata()
            .context("failed to read profile file metadata")?;
        let hash = hash_file(entry.path())?;

        files.insert(
            relative,
            LocalFile {
                path: entry.into_path(),
                manifest: ManifestEntry {
                    hash,
                    size: metadata.len(),
                },
            },
        );
    }

    Ok(LocalDeployment { files })
}

fn is_excluded(entry: &DirEntry, profile_dir: &Path, filter: &ServerFileFilter<'_>) -> bool {
    let Ok(relative) = entry.path().strip_prefix(profile_dir) else {
        return true;
    };

    if relative.as_os_str().is_empty() {
        return false;
    }

    if is_disabled(relative) || filter.excludes(relative) {
        return true;
    }

    if entry.depth() == 1
        && matches!(
            entry.file_name().to_str(),
            Some("profile.json" | "mods.yml" | "snapshots" | "_state")
        )
    {
        return true;
    }

    entry.file_name().to_str() == Some(manifest::FILE_NAME)
}

fn is_disabled(path: &Path) -> bool {
    path.extension().and_then(|extension| extension.to_str()) == Some("old")
}

fn hash_file(path: &Path) -> Result<String> {
    let mut reader = BufReader::new(
        File::open(path).with_context(|| format!("failed to open {}", path.display()))?,
    );
    let mut hasher = blake3::Hasher::new();
    io::copy(&mut reader, &mut hasher)?;

    Ok(hasher.finalize().to_hex().to_string())
}

fn build_plan(
    local: &LocalDeployment,
    previous: DeploymentManifest,
    remote_files: BTreeSet<PathBuf>,
    remote_directories: BTreeSet<PathBuf>,
    policy: &DeploymentPolicy,
    selection: Option<&DeploymentSelection>,
) -> DeploymentPlan {
    let mut current = DeploymentManifest {
        version: manifest::VERSION,
        files: local
            .files
            .iter()
            .map(|(path, file)| (path.clone(), file.manifest.clone()))
            .collect(),
    };
    let mut uploads = local
        .files
        .iter()
        .filter(|(path, file)| {
            previous.files.get(*path) != Some(&file.manifest) || !remote_files.contains(*path)
        })
        .map(|(path, _)| path.clone())
        .collect::<Vec<_>>();
    let (mut removals, preserved_files): (Vec<_>, Vec<_>) = remote_files
        .into_iter()
        .filter(|path| !current.files.contains_key(path))
        .partition(|path| {
            previous.files.contains_key(path)
                || !policy.preserves_untracked(path)
                || is_transfer_artifact(path)
        });
    let tracked_removals = removals
        .iter()
        .filter_map(|path| {
            previous
                .files
                .get(path)
                .map(|entry| (path.clone(), entry.clone()))
        })
        .collect();
    let mut directory_removals = remote_directories
        .into_iter()
        .filter(|directory| policy.manages(directory))
        .filter(|directory| !current.files.keys().any(|path| path.starts_with(directory)))
        .filter(|directory| {
            !preserved_files
                .iter()
                .any(|path| path.starts_with(directory))
        })
        .collect::<Vec<_>>();
    directory_removals.sort_by_key(|path| std::cmp::Reverse(path.components().count()));
    if let Some(selection) = selection {
        let selected_uploads = selection
            .upload_files
            .iter()
            .map(String::as_str)
            .collect::<HashSet<_>>();
        for path in uploads
            .iter()
            .filter(|path| !selected_uploads.contains(display_remote_path(path).as_str()))
        {
            match previous.files.get(path) {
                Some(entry) => current.files.insert(path.clone(), entry.clone()),
                None => current.files.remove(path),
            };
        }
        uploads.retain(|path| selected_uploads.contains(display_remote_path(path).as_str()));

        let selected_removals = selection
            .remove_files
            .iter()
            .map(String::as_str)
            .collect::<HashSet<_>>();
        let skipped_removals = removals
            .iter()
            .filter(|path| !selected_removals.contains(display_remote_path(path).as_str()))
            .cloned()
            .collect::<Vec<_>>();
        for path in &skipped_removals {
            if let Some(entry) = previous.files.get(path) {
                current.files.insert(path.clone(), entry.clone());
            }
        }
        removals.retain(|path| selected_removals.contains(display_remote_path(path).as_str()));
        directory_removals.retain(|path| {
            !skipped_removals
                .iter()
                .any(|skipped| skipped.starts_with(path))
        });
    }
    let upload_bytes = uploads
        .iter()
        .map(|path| local.files[path].manifest.size)
        .sum();
    let unchanged_files = local.files.len() - uploads.len();

    DeploymentPlan {
        current,
        tracked_removals,
        uploads,
        upload_bytes,
        removals,
        directory_removals,
        preserved_files,
        unchanged_files,
    }
}

fn is_transfer_artifact(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".gale-upload") || name.ends_with(".gale-backup"))
}

fn progress(
    completed: usize,
    total: usize,
    path: &Path,
    operation: DeploymentOperation,
) -> DeploymentProgress {
    DeploymentProgress {
        completed,
        total,
        path: display_remote_path(path),
        operation,
    }
}

fn upload_file(
    connection: &mut RemoteConnection,
    base: &Path,
    target: &Path,
    temporary: &Path,
    relative: &Path,
    file: &LocalFile,
    ensured_directories: &mut HashSet<PathBuf>,
) -> Result<()> {
    ensure_remote_parents(connection, base, target, ensured_directories)?;

    let bytes = std::fs::read(&file.path)
        .with_context(|| format!("failed to open {}", file.path.display()))?;
    let uploaded_hash = blake3::hash(&bytes).to_hex().to_string();

    ensure!(
        uploaded_hash == file.manifest.hash,
        "profile file changed during deployment: {}",
        relative.display()
    );

    if !connection.supports_atomic_replace() {
        connection.write_file(target, &bytes).with_context(|| {
            format!(
                "failed to upload remote file {}",
                display_remote_path(relative)
            )
        })?;
        return Ok(());
    }

    connection.write_file(temporary, &bytes).with_context(|| {
        format!(
            "failed to create remote temporary file for {}",
            display_remote_path(relative)
        )
    })?;
    replace_remote_file(connection, temporary, target).with_context(|| {
        format!(
            "failed to replace remote file {}",
            display_remote_path(relative)
        )
    })
}

fn ensure_remote_parents(
    connection: &mut RemoteConnection,
    base: &Path,
    target: &Path,
    ensured_directories: &mut HashSet<PathBuf>,
) -> Result<()> {
    let mut directories = Vec::new();
    let mut current = target.parent();

    while let Some(directory) = current {
        if directory == base {
            break;
        }

        ensure!(
            directory.starts_with(base),
            "remote path escaped the server directory"
        );
        directories.push(directory.to_path_buf());
        current = directory.parent();
    }

    for directory in directories.into_iter().rev() {
        if ensured_directories.contains(&directory) {
            continue;
        }

        connection.ensure_directory(&directory).with_context(|| {
            format!("failed to create remote directory {}", directory.display())
        })?;
        ensured_directories.insert(directory);
    }

    Ok(())
}

fn transfer_with_retry<T>(
    connection: &mut RemoteConnection,
    settings: &RemoteServerSettings,
    password: &str,
    path: &Path,
    mut transfer: impl FnMut(&mut RemoteConnection) -> Result<T>,
) -> Result<T> {
    let mut attempt = 1;

    loop {
        match transfer(connection) {
            Ok(result) => return Ok(result),
            Err(error) if attempt == TRANSFER_ATTEMPTS => return Err(error),
            Err(error) => {
                warn!(%error, path = %path.display(), attempt, "transfer failed; reconnecting");
                thread::sleep(RETRY_DELAY * attempt as u32);
                *connection = reconnect(settings, password)?;
                attempt += 1;
            }
        }
    }
}

fn reconnect(settings: &RemoteServerSettings, password: &str) -> Result<RemoteConnection> {
    match remote::connect(settings, password)? {
        ConnectionAttempt::Connected(connection) => Ok(connection),
        ConnectionAttempt::HostKeyUntrusted { .. } => {
            bail!("remote host key became untrusted while reconnecting")
        }
        ConnectionAttempt::CertificateUntrusted => {
            bail!("FTPS certificate became untrusted while reconnecting")
        }
    }
}

fn write_remote_manifest(
    connection: &mut RemoteConnection,
    target: &Path,
    temporary: &Path,
    manifest: &DeploymentManifest,
) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(manifest)?;

    if !connection.supports_atomic_replace() {
        connection
            .write_file(target, &bytes)
            .context("failed to write remote Gale manifest")?;
        return Ok(());
    }

    connection
        .write_file(temporary, &bytes)
        .context("failed to write remote Gale manifest")?;
    replace_remote_file(connection, temporary, target)
        .context("failed to replace remote Gale manifest")
}

fn replace_remote_file(
    connection: &mut RemoteConnection,
    temporary: &Path,
    target: &Path,
) -> Result<()> {
    if connection.rename_file(temporary, target, true).is_ok() {
        return Ok(());
    }

    let target_exists = connection.file_exists(target)?;
    let backup = append_suffix(target, ".gale-backup");

    if target_exists {
        if connection.file_exists(&backup)? {
            connection
                .remove_file(&backup)
                .context("failed to clear stale remote backup")?;
        }
        if let Err(error) = connection.rename_file(target, &backup, false) {
            let _ = connection.remove_file(temporary);
            return Err(error).context("failed to back up existing remote file");
        }
    }

    if let Err(error) = connection.rename_file(temporary, target, false) {
        if target_exists {
            let _ = connection.rename_file(&backup, target, false);
        }
        let _ = connection.remove_file(temporary);
        return Err(error).context("failed to move uploaded file into place");
    }

    if target_exists {
        let _ = connection.remove_file(&backup);
    }

    Ok(())
}

fn join_name(base: &Path, name: &Path) -> Result<PathBuf> {
    let mut components = name.components();
    ensure!(
        matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none(),
        "remote directory returned an invalid entry name"
    );

    Ok(base.join(name))
}

fn append_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut path = path.as_os_str().to_owned();
    path.push(suffix);
    path.into()
}

fn display_remote_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{BTreeMap, BTreeSet},
        fs,
        path::{Path, PathBuf},
    };

    use super::{
        ClientOnlyMods, DeploymentOperation, DeploymentPolicy, DeploymentPreviewResult,
        DeploymentProgress, DeploymentResult, DeploymentSelection, RemoteLayout, ServerFileFilter,
        build_plan, collect_local_files,
    };
    use crate::{
        game::mod_loader::ServerDeployment,
        profile::server::manifest::{DeploymentManifest, ManifestEntry},
    };

    fn policy() -> DeploymentPolicy {
        DeploymentPolicy::from_config(ServerDeployment {
            loader_root: PathBuf::from("BepInEx"),
            managed_directories: vec![
                PathBuf::from("BepInEx/plugins"),
                PathBuf::from("BepInEx/patchers"),
                PathBuf::from("BepInEx/config"),
            ],
            preserve_untracked_directories: vec![PathBuf::from("BepInEx/config")],
            manifest_directory: PathBuf::from("BepInEx/config"),
            excluded_globs: &[
                "BepInEx/{cache,DumpedAssemblies,interop}/**",
                "BepInEx/LogOutput.log",
                "BepInEx/plugins/*/{README.md,CHANGELOG.md,icon.png}",
            ],
        })
        .unwrap()
    }

    fn path(value: &str) -> PathBuf {
        PathBuf::from(value)
    }

    fn local(directory: &Path) -> super::LocalDeployment {
        let policy = policy();
        collect_local_files(
            directory,
            &ServerFileFilter::new(&policy, &ClientOnlyMods::default(), false),
        )
        .unwrap()
    }

    #[test]
    fn serializes_frontend_field_names() {
        let value = serde_json::to_value(DeploymentResult::Deployed {
            uploaded_files: 3,
            uploaded_bytes: 42,
            removed_files: 1,
            unchanged_files: 2,
            skipped_client_only_mods: vec!["Author-ClientOnly".to_owned()],
            cleanup_warnings: vec!["Could not remove locked.dll: permission denied".to_owned()],
        })
        .unwrap();

        assert_eq!(value["uploadedFiles"], 3);
        assert_eq!(value["uploadedBytes"], 42);
        assert_eq!(value["removedFiles"], 1);
        assert_eq!(value["unchangedFiles"], 2);
        assert_eq!(value["skippedClientOnlyMods"][0], "Author-ClientOnly");
        assert_eq!(value["cleanupWarnings"].as_array().unwrap().len(), 1);

        let preview = serde_json::to_value(DeploymentPreviewResult::Preview {
            upload_files: vec!["BepInEx/plugins/Test.dll".to_owned()],
            upload_file_sizes: [("BepInEx/plugins/Test.dll".to_owned(), 42)]
                .into_iter()
                .collect(),
            remove_files: vec!["BepInEx/plugins/Old.dll".to_owned()],
            preserved_files: 1,
            unchanged_files: 2,
            skipped_client_only_mods: vec!["Author-ClientOnly".to_owned()],
        })
        .unwrap();
        assert_eq!(preview["status"], "preview");
        assert_eq!(preview["uploadFileSizes"]["BepInEx/plugins/Test.dll"], 42);
        assert_eq!(preview["removeFiles"][0], "BepInEx/plugins/Old.dll");
        assert_eq!(preview["preservedFiles"], 1);

        let progress = serde_json::to_value(DeploymentProgress {
            completed: 1,
            total: 3,
            path: "BepInEx/plugins/Test.dll".to_owned(),
            operation: DeploymentOperation::Upload,
        })
        .unwrap();
        assert_eq!(progress["operation"], "upload");
    }

    #[test]
    fn maps_restricted_loader_roots() {
        let standard = RemoteLayout {
            base: path("/"),
            loader_root: path("BepInEx"),
            manifest_directory: path("BepInEx/config"),
            loader_at_root: false,
        };
        let restricted = RemoteLayout {
            loader_at_root: true,
            ..standard.clone()
        };

        assert_eq!(
            standard.path(&path("BepInEx/plugins/Example.dll")).unwrap(),
            path("/BepInEx/plugins/Example.dll")
        );
        assert_eq!(
            restricted
                .path(&path("BepInEx/plugins/Example.dll"))
                .unwrap(),
            path("/plugins/Example.dll")
        );
        assert_eq!(
            restricted.manifest_path().unwrap(),
            path("/config/.gale-server-manifest.json")
        );
    }

    #[test]
    fn disabled_files_become_remote_deletions() {
        let directory = tempfile::tempdir().unwrap();
        let plugin = directory.path().join("BepInEx/plugins/Example");
        fs::create_dir_all(&plugin).unwrap();
        fs::write(plugin.join("Enabled.dll"), b"enabled").unwrap();
        fs::write(plugin.join("Disabled.dll.old"), b"disabled").unwrap();
        let local = local(directory.path());
        let remote = [path("BepInEx/plugins/Example/Disabled.dll")]
            .into_iter()
            .collect();
        let plan = build_plan(
            &local,
            DeploymentManifest::default(),
            remote,
            BTreeSet::new(),
            &policy(),
            None,
        );

        assert!(
            local
                .files
                .contains_key(&path("BepInEx/plugins/Example/Enabled.dll"))
        );
        assert_eq!(
            plan.removals,
            [path("BepInEx/plugins/Example/Disabled.dll")]
        );
    }

    #[test]
    fn ignores_generated_files_and_disposable_metadata() {
        let directory = tempfile::tempdir().unwrap();
        let bepinex = directory.path().join("BepInEx");
        let plugin = bepinex.join("plugins/Example");
        fs::create_dir_all(bepinex.join("cache")).unwrap();
        fs::create_dir_all(bepinex.join("DumpedAssemblies")).unwrap();
        fs::create_dir_all(&plugin).unwrap();
        fs::write(bepinex.join("cache/generated.dat"), b"cache").unwrap();
        fs::write(bepinex.join("DumpedAssemblies/Game.dll"), b"dump").unwrap();
        fs::write(bepinex.join("LogOutput.log"), b"log").unwrap();
        fs::write(plugin.join("README.md"), b"readme").unwrap();
        fs::write(plugin.join("icon.png"), b"icon").unwrap();
        fs::write(plugin.join("manifest.json"), b"manifest").unwrap();
        fs::write(plugin.join("RuntimeAsset.json"), b"asset").unwrap();
        let local = local(directory.path());

        assert_eq!(local.files.len(), 2);
        assert!(
            local
                .files
                .contains_key(&path("BepInEx/plugins/Example/manifest.json"))
        );
        assert!(
            local
                .files
                .contains_key(&path("BepInEx/plugins/Example/RuntimeAsset.json"))
        );
    }

    #[test]
    fn excludes_client_only_files_from_installer_ownership() {
        let directory = tempfile::tempdir().unwrap();
        let client = directory.path().join("BepInEx/plugins/Author-ClientOnly");
        let shared = directory.path().join("BepInEx/plugins/Author-Shared");
        let patcher = directory.path().join("BepInEx/patchers/ClientPatcher.dll");
        fs::create_dir_all(&client).unwrap();
        fs::create_dir_all(&shared).unwrap();
        fs::create_dir_all(patcher.parent().unwrap()).unwrap();
        fs::write(client.join("Client.dll"), b"client").unwrap();
        fs::write(shared.join("Shared.dll"), b"shared").unwrap();
        fs::write(&patcher, b"patcher").unwrap();
        let client_only = ClientOnlyMods {
            names: vec!["Author-ClientOnly".to_owned()],
            paths: [
                PathBuf::from("BepInEx/patchers/ClientPatcher.dll"),
                PathBuf::from("BepInEx/plugins/Author-ClientOnly"),
            ]
            .into_iter()
            .collect(),
        };
        let policy = policy();
        let local = collect_local_files(
            directory.path(),
            &ServerFileFilter::new(&policy, &client_only, false),
        )
        .unwrap();

        assert_eq!(local.files.len(), 1);
        assert!(
            local
                .files
                .contains_key(&path("BepInEx/plugins/Author-Shared/Shared.dll"))
        );
    }

    #[test]
    fn preserves_host_managed_loader_files() {
        let directory = tempfile::tempdir().unwrap();
        fs::create_dir_all(directory.path().join("BepInEx/plugins/Example")).unwrap();
        fs::write(directory.path().join("start_game_bepinex.sh"), b"start").unwrap();
        fs::write(directory.path().join(".doorstop_version"), b"doorstop").unwrap();
        fs::write(
            directory.path().join("BepInEx/plugins/Example/Example.dll"),
            b"plugin",
        )
        .unwrap();
        let policy = policy();
        let local = collect_local_files(
            directory.path(),
            &ServerFileFilter::new(&policy, &ClientOnlyMods::default(), true),
        )
        .unwrap();

        assert_eq!(local.files.len(), 1);
        assert!(
            local
                .files
                .contains_key(&path("BepInEx/plugins/Example/Example.dll"))
        );
    }

    #[test]
    fn managed_directories_remove_only_remote_extras() {
        let directory = tempfile::tempdir().unwrap();
        let plugin = directory.path().join("BepInEx/plugins/Example");
        fs::create_dir_all(&plugin).unwrap();
        fs::write(plugin.join("Keep.dll"), b"plugin").unwrap();
        let local = local(directory.path());
        let remote = [
            path("BepInEx/plugins/Example/Keep.dll"),
            path("BepInEx/plugins/Example/Delete.dll"),
        ]
        .into_iter()
        .collect();
        let plan = build_plan(
            &local,
            DeploymentManifest::default(),
            remote,
            BTreeSet::new(),
            &policy(),
            None,
        );

        assert_eq!(plan.removals, [path("BepInEx/plugins/Example/Delete.dll")]);
    }

    #[test]
    fn applies_only_selected_changes_to_the_manifest() {
        let directory = tempfile::tempdir().unwrap();
        let plugin = directory.path().join("BepInEx/plugins/Example");
        fs::create_dir_all(&plugin).unwrap();
        fs::write(plugin.join("New.dll"), b"new").unwrap();
        fs::write(plugin.join("Changed.dll"), b"changed").unwrap();
        let local = local(directory.path());
        let changed = path("BepInEx/plugins/Example/Changed.dll");
        let removed = path("BepInEx/plugins/Removed/Removed.dll");
        let old_entry = ManifestEntry {
            hash: "old".to_owned(),
            size: 3,
        };
        let previous = DeploymentManifest {
            files: BTreeMap::from([
                (changed.clone(), old_entry.clone()),
                (removed.clone(), old_entry.clone()),
            ]),
            ..DeploymentManifest::default()
        };
        let plan = build_plan(
            &local,
            previous,
            [changed.clone(), removed.clone()].into_iter().collect(),
            [path("BepInEx/plugins/Removed")].into_iter().collect(),
            &policy(),
            Some(&DeploymentSelection {
                upload_files: vec!["BepInEx/plugins/Example/New.dll".to_owned()],
                remove_files: Vec::new(),
            }),
        );

        assert_eq!(plan.uploads, [path("BepInEx/plugins/Example/New.dll")]);
        assert!(plan.removals.is_empty());
        assert!(plan.directory_removals.is_empty());
        assert_eq!(plan.current.files[&changed], old_entry);
        assert!(plan.current.files.contains_key(&removed));
    }

    #[test]
    fn removes_empty_directories_for_selected_files() {
        let directory = tempfile::tempdir().unwrap();
        let local = local(directory.path());
        let removed = path("BepInEx/plugins/Removed/Removed.dll");
        let previous = DeploymentManifest {
            files: BTreeMap::from([(
                removed.clone(),
                ManifestEntry {
                    hash: "hash".to_owned(),
                    size: 1,
                },
            )]),
            ..DeploymentManifest::default()
        };
        let plan = build_plan(
            &local,
            previous,
            [removed].into_iter().collect(),
            [path("BepInEx/plugins/Removed")].into_iter().collect(),
            &policy(),
            Some(&DeploymentSelection {
                upload_files: Vec::new(),
                remove_files: vec!["BepInEx/plugins/Removed/Removed.dll".to_owned()],
            }),
        );

        assert_eq!(plan.directory_removals, [path("BepInEx/plugins/Removed")]);
    }

    #[test]
    fn preserves_untracked_server_config_files() {
        let directory = tempfile::tempdir().unwrap();
        let local = local(directory.path());
        let generated = path("BepInEx/config/AntiCheat/whitelist.yml");
        let tracked = path("BepInEx/config/Removed.cfg");
        let artifact = path("BepInEx/config/Failed.cfg.gale-upload");
        let plugin = path("BepInEx/plugins/Removed.dll");
        let previous = DeploymentManifest {
            files: BTreeMap::from([(
                tracked.clone(),
                ManifestEntry {
                    hash: "hash".to_owned(),
                    size: 1,
                },
            )]),
            ..DeploymentManifest::default()
        };
        let remote_files = [
            generated.clone(),
            tracked.clone(),
            artifact.clone(),
            plugin.clone(),
        ]
        .into_iter()
        .collect();
        let remote_directories = [path("BepInEx/config/AntiCheat")].into_iter().collect();
        let plan = build_plan(
            &local,
            previous,
            remote_files,
            remote_directories,
            &policy(),
            None,
        );

        assert_eq!(plan.preserved_files, [generated]);
        assert_eq!(plan.removals, [artifact, tracked, plugin]);
        assert!(plan.directory_removals.is_empty());
    }

    #[test]
    fn reuploads_manifest_file_missing_from_server() {
        let directory = tempfile::tempdir().unwrap();
        let plugin = directory.path().join("BepInEx/plugins/Example");
        fs::create_dir_all(&plugin).unwrap();
        fs::write(plugin.join("Example.dll"), b"plugin").unwrap();
        let local = local(directory.path());
        let previous = DeploymentManifest {
            files: local
                .files
                .iter()
                .map(|(path, file)| (path.clone(), file.manifest.clone()))
                .collect(),
            ..DeploymentManifest::default()
        };
        let plan = build_plan(
            &local,
            previous,
            BTreeSet::new(),
            BTreeSet::new(),
            &policy(),
            None,
        );

        assert_eq!(plan.uploads, [path("BepInEx/plugins/Example/Example.dll")]);
        assert_eq!(plan.unchanged_files, 0);
    }

    #[test]
    fn keeps_loader_files_on_the_second_deployment() {
        let directory = tempfile::tempdir().unwrap();
        fs::create_dir_all(directory.path().join("BepInEx/plugins/Example")).unwrap();
        fs::create_dir_all(directory.path().join("BepInEx/core")).unwrap();
        fs::write(directory.path().join("BepInEx/core/core.dll"), b"core").unwrap();
        fs::write(
            directory.path().join("BepInEx/plugins/Example/Example.dll"),
            b"plugin",
        )
        .unwrap();
        let policy = policy();
        let local = collect_local_files(
            directory.path(),
            &ServerFileFilter::new(&policy, &ClientOnlyMods::default(), false),
        )
        .unwrap();
        let first = build_plan(
            &local,
            DeploymentManifest::default(),
            BTreeSet::new(),
            BTreeSet::new(),
            &policy,
            None,
        );
        let remote = first.current.files.keys().cloned().collect();
        let preserve_loader = policy.preserve_host_loader(&first.current, true);
        let local = collect_local_files(
            directory.path(),
            &ServerFileFilter::new(&policy, &ClientOnlyMods::default(), preserve_loader),
        )
        .unwrap();
        let second = build_plan(
            &local,
            first.current,
            remote,
            BTreeSet::new(),
            &policy,
            None,
        );

        assert!(!preserve_loader);
        assert!(second.uploads.is_empty());
        assert!(second.removals.is_empty());
        assert!(
            second
                .current
                .files
                .contains_key(&path("BepInEx/core/core.dll"))
        );
    }

    #[test]
    fn removes_empty_remote_mod_directories_deepest_first() {
        let directory = tempfile::tempdir().unwrap();
        let local = local(directory.path());
        let remote_files = [path("BepInEx/plugins/HearthBelow/lib/Old.dll")]
            .into_iter()
            .collect();
        let remote_directories = [
            path("BepInEx/plugins/HearthBelow"),
            path("BepInEx/plugins/HearthBelow/lib"),
        ]
        .into_iter()
        .collect();
        let plan = build_plan(
            &local,
            DeploymentManifest::default(),
            remote_files,
            remote_directories,
            &policy(),
            None,
        );

        assert_eq!(
            plan.directory_removals,
            [
                path("BepInEx/plugins/HearthBelow/lib"),
                path("BepInEx/plugins/HearthBelow")
            ]
        );
    }
}
