use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use eyre::{Context, OptionExt, Result, bail, ensure};
use serde::Serialize;
use tracing::{info, warn};
use walkdir::{DirEntry, WalkDir};

use super::{
    manifest::{self, DeploymentManifest, ManifestEntry},
    paths::{DeployPath, DeployPathBuf, RemotePath, RemotePathBuf},
    remote::{ConnectionAttempt, RemoteConnection},
    settings::RemoteServerSettings,
    spec::DeploymentSpec,
};
use crate::util;

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
        fingerprint: Option<String>,
        uploaded_files: usize,
        uploaded_bytes: u64,
        removed_files: usize,
        unchanged_files: usize,
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
        fingerprint: Option<String>,
        upload_files: Vec<String>,
        upload_bytes: u64,
        remove_files: Vec<String>,
        unchanged_files: usize,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentProgress {
    pub completed: usize,
    pub total: usize,
    pub path: DeployPathBuf,
    pub operation: DeploymentOperation,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DeploymentOperation {
    Upload,
    Remove,
}

/// How the remote server lays out the profile relative to the configured
/// server directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RemoteLayout {
    /// The remote directory maps directly onto the profile root.
    Standard,
    /// The host is restricted to the mod loader's directory: it exposes
    /// `BepInEx/` contents at the remote root, so the mirror-root prefix is
    /// stripped when mapping deploy paths.
    MirrorRoot,
}

/// Maps deploy paths to absolute remote paths for one connection setup.
#[derive(Clone)]
struct RemoteMapper<'a> {
    spec: &'a DeploymentSpec,
    base: RemotePathBuf,
    layout: RemoteLayout,
}

/// An open remote session: the connection plus everything needed to map
/// deploy paths, bundled so it doesn't have to be passed around separately.
struct Session<'a> {
    connection: RemoteConnection,
    mapper: RemoteMapper<'a>,
    /// Whether the host manages the mod loader itself (restricted hosts, or
    /// hosts with a pre-installed loader Gale did not deploy). When true only
    /// mirror directories are written.
    host_managed: bool,
    previous: DeploymentManifest,
}

enum OpenSession<'a> {
    Connected(Session<'a>),
    HostKeyUntrusted { fingerprint: String },
    CertificateUntrusted,
}

struct LocalFile {
    path: PathBuf,
    manifest: ManifestEntry,
}

/// The local profile as the deployment should see it: every deployable file
/// and its manifest entry.
struct LocalDeployment {
    files: BTreeMap<DeployPathBuf, LocalFile>,
}

struct DeploymentPlan {
    current: DeploymentManifest,
    uploads: Vec<DeployPathBuf>,
    upload_bytes: u64,
    removals: Vec<DeployPathBuf>,
    directory_removals: Vec<DeployPathBuf>,
    unchanged_files: usize,
}

pub fn preview(
    profile_dir: &Path,
    spec: &DeploymentSpec,
    settings: &RemoteServerSettings,
    password: &str,
) -> Result<DeploymentPreviewResult> {
    let mut session = match open_session(spec, settings, password)? {
        OpenSession::Connected(session) => session,
        OpenSession::HostKeyUntrusted { fingerprint } => {
            return Ok(DeploymentPreviewResult::HostKeyUntrusted { fingerprint });
        }
        OpenSession::CertificateUntrusted => {
            return Ok(DeploymentPreviewResult::CertificateUntrusted);
        }
    };

    let (_, plan) = session.prepare(profile_dir)?;

    Ok(DeploymentPreviewResult::Preview {
        fingerprint: session.connection.fingerprint,
        upload_files: plan.uploads.iter().map(ToString::to_string).collect(),
        upload_bytes: plan.upload_bytes,
        remove_files: plan
            .removals
            .iter()
            .map(ToString::to_string)
            .chain(
                plan.directory_removals
                    .iter()
                    .map(|path| format!("{path}/")),
            )
            .collect(),
        unchanged_files: plan.unchanged_files,
    })
}

pub fn deploy<F>(
    profile_dir: &Path,
    spec: &DeploymentSpec,
    settings: &RemoteServerSettings,
    password: &str,
    mut report: F,
) -> Result<DeploymentResult>
where
    F: FnMut(DeploymentProgress),
{
    let mut session = match open_session(spec, settings, password)? {
        OpenSession::Connected(session) => session,
        OpenSession::HostKeyUntrusted { fingerprint } => {
            return Ok(DeploymentResult::HostKeyUntrusted { fingerprint });
        }
        OpenSession::CertificateUntrusted => {
            return Ok(DeploymentResult::CertificateUntrusted);
        }
    };

    let (local, plan) = session.prepare(profile_dir)?;
    let total = plan.uploads.len() + plan.removals.len() + plan.directory_removals.len();
    let mut completed = 0;

    // Removal failures are non-fatal: leaving a stale file behind only means
    // the server keeps an unused mod. Report them to the user afterwards.
    let mut removed_files = 0;
    let mut cleanup_warnings = Vec::new();

    for path in &plan.removals {
        let remote_path = session.mapper.remote_path(path);
        match session.connection.remove_file(&remote_path) {
            Ok(true) => removed_files += 1,
            Ok(false) => {}
            Err(error) => {
                let warning = format!("Could not remove {path}: {error}");
                warn!("{warning}");
                cleanup_warnings.push(warning);
            }
        }
        completed += 1;
        report(DeploymentProgress {
            completed,
            total,
            path: path.clone(),
            operation: DeploymentOperation::Remove,
        });
    }

    for directory in &plan.directory_removals {
        let remote_path = session.mapper.remote_path(directory);
        if let Err(error) = session.connection.remove_directory(&remote_path) {
            let warning = format!("Could not remove {directory}/: {error}");
            warn!("{warning}");
            cleanup_warnings.push(warning);
        }
        completed += 1;
        report(DeploymentProgress {
            completed,
            total,
            path: directory.clone(),
            operation: DeploymentOperation::Remove,
        });
    }

    let mapper = session.mapper.clone();
    let mut ensured_directories = BTreeSet::new();
    for path in &plan.uploads {
        let file = &local.files[path];
        with_retry(
            &mut session.connection,
            |connection| {
                upload_file(connection, &mapper, path, file, &mut ensured_directories).inspect_err(
                    // A retry may reconnect, so re-ensure the directories.
                    |_| ensured_directories.clear(),
                )
            },
            || reconnect(settings, password),
        )?;
        completed += 1;
        report(DeploymentProgress {
            completed,
            total,
            path: path.clone(),
            operation: DeploymentOperation::Upload,
        });
    }

    with_retry(
        &mut session.connection,
        |connection| write_manifest(connection, &mapper, &plan.current),
        || reconnect(settings, password),
    )?;

    Ok(DeploymentResult::Deployed {
        fingerprint: session.connection.fingerprint,
        uploaded_files: plan.uploads.len(),
        uploaded_bytes: plan.upload_bytes,
        removed_files,
        unchanged_files: plan.unchanged_files,
        cleanup_warnings,
    })
}

fn open_session<'a>(
    spec: &'a DeploymentSpec,
    settings: &RemoteServerSettings,
    password: &str,
) -> Result<OpenSession<'a>> {
    let connection = match RemoteConnection::connect(settings, password)? {
        ConnectionAttempt::Connected(connection) => connection,
        ConnectionAttempt::HostKeyUntrusted { fingerprint } => {
            return Ok(OpenSession::HostKeyUntrusted { fingerprint });
        }
        ConnectionAttempt::CertificateUntrusted => {
            return Ok(OpenSession::CertificateUntrusted);
        }
    };

    Session::new(connection, spec, settings).map(OpenSession::Connected)
}

impl<'a> RemoteMapper<'a> {
    /// Maps a deploy path to the absolute path on the remote server.
    fn remote_path(&self, deploy: &DeployPath) -> RemotePathBuf {
        match self.layout {
            RemoteLayout::Standard => self.base.join(deploy),
            RemoteLayout::MirrorRoot => self
                .spec
                .strip_mirror_root(deploy)
                .map(|relative| self.base.join(&relative))
                .unwrap_or_else(|| self.base.join(deploy)),
        }
    }
}

impl<'a> Session<'a> {
    fn new(
        mut connection: RemoteConnection,
        spec: &'a DeploymentSpec,
        settings: &RemoteServerSettings,
    ) -> Result<Self> {
        let base = settings.server_directory()?;
        connection.check_directory(&base).with_context(|| {
            format!(
                "remote server directory '{}' could not be accessed",
                settings.server_directory
            )
        })?;

        let layout = detect_layout(&mut connection, spec, &base)?;
        let mapper = RemoteMapper { spec, base, layout };

        let previous = read_manifest(&mut connection, &mapper)?;
        ensure!(
            previous.version == manifest::VERSION,
            "unsupported remote Gale manifest version {}",
            previous.version
        );

        let host_managed = detect_host_managed(&mut connection, &mapper, &previous)?;

        info!(
            ?layout,
            host_managed,
            manifest_files = previous.files.len(),
            "resolved remote server layout"
        );

        Ok(Self {
            connection,
            mapper,
            host_managed,
            previous,
        })
    }

    fn prepare(&mut self, profile_dir: &Path) -> Result<(LocalDeployment, DeploymentPlan)> {
        let local = collect_local_files(profile_dir, self.mapper.spec, self.host_managed)?;

        let (remote_mirror_files, remote_mirror_directories) = self.collect_mirror_files()?;
        let previous = std::mem::take(&mut self.previous);
        let plan = build_plan(
            &local,
            previous,
            remote_mirror_files,
            remote_mirror_directories,
            self.mapper.spec,
        )?;

        info!(
            uploads = plan.uploads.len(),
            removals = plan.removals.len(),
            unchanged = plan.unchanged_files,
            upload_bytes = plan.upload_bytes,
            "prepared remote deployment"
        );

        Ok((local, plan))
    }

    /// Lists every remote file and directory inside the mirror directories,
    /// as deploy paths.
    fn collect_mirror_files(
        &mut self,
    ) -> Result<(BTreeSet<DeployPathBuf>, BTreeSet<DeployPathBuf>)> {
        let mut files = BTreeSet::new();
        let mut directories = BTreeSet::new();
        let dirs = self.mapper.spec.mirror_dirs.clone();
        for dir in &dirs {
            self.collect_remote(dir, &mut files, &mut directories)?;
        }
        Ok((files, directories))
    }

    fn collect_remote(
        &mut self,
        dir: &DeployPath,
        files: &mut BTreeSet<DeployPathBuf>,
        directories: &mut BTreeSet<DeployPathBuf>,
    ) -> Result<()> {
        let remote_dir = self.mapper.remote_path(dir);
        let manifest_tmp = self.mapper.spec.manifest_path.with_suffix(".tmp")?;

        for entry in self
            .connection
            .list_directory_entries(&remote_dir)
            .with_context(|| format!("failed to inspect remote directory {remote_dir}"))?
        {
            let relative = dir.join(&entry.name).with_context(|| {
                format!("remote directory contains an unsafe name: {}", entry.name)
            })?;

            if entry.is_directory {
                directories.insert(relative.clone());
                self.collect_remote(&relative, files, directories)?;
            } else if relative != self.mapper.spec.manifest_path && relative != manifest_tmp {
                files.insert(relative);
            }
        }

        Ok(())
    }
}

fn detect_layout(
    connection: &mut RemoteConnection,
    spec: &DeploymentSpec,
    base: &RemotePath,
) -> Result<RemoteLayout> {
    // A standard server has the loader directory (e.g. `BepInEx`) inside the
    // configured directory. If it is missing but the first mirror dir sits at
    // the remote root (`plugins`, `config`, ...), the host only exposes the
    // loader's contents — a restricted layout.
    if connection
        .directory_exists(&base.join(&spec.mirror_root))
        .context("failed to check for a server-managed mod loader installation")?
    {
        return Ok(RemoteLayout::Standard);
    }

    let first_mirror = spec.mirror_dirs.first().expect("spec has mirror dirs");
    let at_root = spec
        .strip_mirror_root(first_mirror)
        .unwrap_or_else(|| first_mirror.to_owned_buf());

    if connection
        .directory_exists(&base.join(&at_root))
        .context("failed to check for a restricted server layout")?
    {
        return Ok(RemoteLayout::MirrorRoot);
    }

    Ok(RemoteLayout::Standard)
}

/// Whether the host manages the mod loader itself.
///
/// A restricted host always does — it only exposes the loader's contents. On
/// standard layouts the loader marker paths decide, unless a previous Gale
/// deployment recorded the loader payload: files Gale uploaded stay Gale's,
/// so their presence alone doesn't transfer ownership to the host.
fn detect_host_managed(
    connection: &mut RemoteConnection,
    mapper: &RemoteMapper,
    previous: &DeploymentManifest,
) -> Result<bool> {
    let owns_loader = mapper.spec.owns_loader(previous);
    let mut marker_exists = false;

    if mapper.layout == RemoteLayout::Standard {
        for marker in &mapper.spec.loader_markers {
            if connection
                .directory_exists(&mapper.remote_path(marker))
                .context("failed to check for a server-managed mod loader installation")?
            {
                marker_exists = true;
                break;
            }
        }
    }

    Ok(is_host_managed(mapper.layout, owns_loader, marker_exists))
}

fn is_host_managed(layout: RemoteLayout, owns_loader: bool, marker_exists: bool) -> bool {
    layout == RemoteLayout::MirrorRoot || (!owns_loader && marker_exists)
}

fn read_manifest(
    connection: &mut RemoteConnection,
    mapper: &RemoteMapper,
) -> Result<DeploymentManifest> {
    let remote = mapper.remote_path(&mapper.spec.manifest_path);
    let bytes = match connection.read_file(&remote)? {
        Some(bytes) => bytes,
        None => return Ok(DeploymentManifest::default()),
    };

    ensure!(
        bytes.len() as u64 <= MAX_MANIFEST_BYTES,
        "remote Gale manifest is unexpectedly large"
    );

    serde_json::from_slice(&bytes).context("remote Gale manifest is invalid")
}

/// Collects the deployable files of the local profile and their hashes.
fn collect_local_files(
    profile_dir: &Path,
    spec: &DeploymentSpec,
    host_managed: bool,
) -> Result<LocalDeployment> {
    let mut files = BTreeMap::new();

    for entry in WalkDir::new(profile_dir)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| !is_excluded(entry, profile_dir, spec, host_managed))
    {
        let entry = entry.context("failed to enumerate profile files")?;

        if !entry.file_type().is_file() {
            continue;
        }

        let relative = entry
            .path()
            .strip_prefix(profile_dir)
            .context("profile file escaped its root")?;

        // `.old` files are disabled and never uploaded. Their enabled
        // counterpart is absent from the manifest, so the mirror rules remove
        // any still-deployed copy on the server.
        if is_disabled(relative) {
            continue;
        }

        let relative = deploy_path(relative)?;
        let metadata = entry
            .metadata()
            .context("failed to read profile file metadata")?;
        let hash = util::fs::checksum(entry.path())
            .with_context(|| format!("failed to hash {}", entry.path().display()))?
            .to_hex()
            .to_string();

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

fn is_disabled(path: &Path) -> bool {
    path.extension().and_then(|extension| extension.to_str()) == Some("old")
}

fn is_excluded(
    entry: &DirEntry,
    profile_dir: &Path,
    spec: &DeploymentSpec,
    host_managed: bool,
) -> bool {
    let Ok(relative) = entry.path().strip_prefix(profile_dir) else {
        return false;
    };

    if relative.as_os_str().is_empty() {
        return false;
    }

    let Ok(path) = deploy_path(relative) else {
        // Unsafe names never get deployed — skip them entirely.
        return true;
    };

    if entry.file_type().is_dir() {
        !spec.descends(&path, host_managed)
    } else {
        !spec.deploys(&path, host_managed)
    }
}

/// Converts a local relative path into its canonical deploy-path form.
fn deploy_path(path: &Path) -> Result<DeployPathBuf> {
    let value = path
        .components()
        .map(|component| component.as_os_str().to_str())
        .collect::<Option<Vec<_>>>()
        .ok_or_eyre("profile contains a non-Unicode path")?
        .join("/");

    DeployPathBuf::new(value)
}

fn build_plan(
    local: &LocalDeployment,
    previous: DeploymentManifest,
    remote_mirror_files: BTreeSet<DeployPathBuf>,
    remote_mirror_directories: BTreeSet<DeployPathBuf>,
    spec: &DeploymentSpec,
) -> Result<DeploymentPlan> {
    let current = DeploymentManifest {
        version: manifest::VERSION,
        files: local
            .files
            .iter()
            .map(|(path, file)| (path.clone(), file.manifest.clone()))
            .collect(),
    };

    // A file is uploaded when it is new or changed, or when the remote mirror
    // copy went missing even though the manifest expects it — for example
    // after a host wiped the mirror dirs.
    let uploads = local
        .files
        .iter()
        .filter(|(path, file)| {
            previous.files.get(*path) != Some(&file.manifest)
                || (spec.is_mirrored(path) && !remote_mirror_files.contains(*path))
        })
        .map(|(path, _)| path.clone())
        .collect::<Vec<_>>();

    let upload_bytes = uploads
        .iter()
        .map(|path| local.files[path].manifest.size)
        .sum();
    let unchanged_files = local.files.len() - uploads.len();

    // Removal is strictly bounded to Gale-owned files: everything the remote
    // still holds inside the mirror dirs that is not deployed anymore, plus
    // anything a previous manifest recorded that is gone now. Host files
    // outside the mirrors are never touched.
    let removals = remote_mirror_files
        .into_iter()
        .chain(previous.files.into_keys())
        .filter(|path| !current.files.contains_key(path))
        .collect::<BTreeSet<_>>();

    let mut directory_removals = remote_mirror_directories
        .into_iter()
        .filter(|directory| {
            !current
                .files
                .keys()
                .any(|path| directory.is_ancestor_of(path))
        })
        .collect::<Vec<_>>();

    // Deepest first so parents are removed after their contents.
    directory_removals.sort_by_key(|path| std::cmp::Reverse(path.segments().count()));

    Ok(DeploymentPlan {
        current,
        uploads,
        upload_bytes,
        removals: removals.into_iter().collect(),
        directory_removals,
        unchanged_files,
    })
}

fn upload_file(
    connection: &mut RemoteConnection,
    mapper: &RemoteMapper,
    path: &DeployPath,
    file: &LocalFile,
    ensured_directories: &mut BTreeSet<RemotePathBuf>,
) -> Result<()> {
    ensure_remote_parents(connection, mapper, path, ensured_directories)?;

    let target = mapper.remote_path(path);
    let temporary = mapper.remote_path(&path.with_suffix(".gale-upload")?);
    let bytes = std::fs::read(&file.path)
        .with_context(|| format!("failed to open {}", file.path.display()))?;

    // The file may have changed between hashing and upload (e.g. the user
    // edited a config). Don't silently deploy a different file than planned.
    let uploaded_hash = blake3::hash(&bytes).to_hex().to_string();
    if uploaded_hash != file.manifest.hash {
        bail!("profile file changed during deployment: {path}");
    }

    if !connection.supports_atomic_replace() {
        connection
            .write_file(&target, &bytes)
            .with_context(|| format!("failed to upload remote file {path}"))?;
        return Ok(());
    }

    connection
        .write_file(&temporary, &bytes)
        .with_context(|| format!("failed to create remote temporary file for {path}"))?;

    replace_remote_file(connection, &temporary, &target)
        .with_context(|| format!("failed to replace remote file {path}"))?;

    Ok(())
}

/// Runs `work` with reconnect-and-retry semantics for transient transfer
/// failures. `reconnect` provides a fresh connection between attempts.
fn with_retry<T>(
    connection: &mut RemoteConnection,
    mut work: impl FnMut(&mut RemoteConnection) -> Result<T>,
    mut reconnect: impl FnMut() -> Result<RemoteConnection>,
) -> Result<T> {
    let mut attempt = 1;
    loop {
        match work(connection) {
            Ok(value) => return Ok(value),
            Err(error) if attempt == TRANSFER_ATTEMPTS => return Err(error),
            Err(error) => {
                warn!(attempt, %error, "transfer failed; reconnecting");
                thread::sleep(RETRY_DELAY * attempt as u32);
                *connection = reconnect()?;
                attempt += 1;
            }
        }
    }
}

fn reconnect(settings: &RemoteServerSettings, password: &str) -> Result<RemoteConnection> {
    match RemoteConnection::connect(settings, password)? {
        ConnectionAttempt::Connected(connection) => Ok(connection),
        ConnectionAttempt::HostKeyUntrusted { .. } => {
            bail!("remote host key became untrusted while reconnecting")
        }
        ConnectionAttempt::CertificateUntrusted => {
            bail!("FTPS certificate became untrusted while reconnecting")
        }
    }
}

fn ensure_remote_parents(
    connection: &mut RemoteConnection,
    mapper: &RemoteMapper,
    path: &DeployPath,
    ensured_directories: &mut BTreeSet<RemotePathBuf>,
) -> Result<()> {
    let mut ancestors = path.self_and_ancestors().collect::<Vec<_>>();
    ancestors.pop(); // the file itself

    for ancestor in ancestors {
        let remote = mapper.remote_path(&ancestor);
        if !ensured_directories.insert(remote.clone()) {
            continue;
        }
        connection
            .ensure_directory(&remote)
            .with_context(|| format!("failed to create remote directory {remote}"))?;
    }

    Ok(())
}

fn write_manifest(
    connection: &mut RemoteConnection,
    mapper: &RemoteMapper,
    deployment: &DeploymentManifest,
) -> Result<()> {
    // The manifest lives inside a mirror dir, but it may not exist yet on a
    // first deploy — ensure the parent before writing.
    let manifest_path = &mapper.spec.manifest_path;
    if let Some(parent) = manifest_path.parent() {
        let remote = mapper.remote_path(&parent);
        connection
            .ensure_directory(&remote)
            .context("failed to create remote manifest directory")?;
    }

    let target = mapper.remote_path(manifest_path);
    let temporary = target.with_suffix(".tmp");
    let bytes = serde_json::to_vec_pretty(deployment)?;

    if !connection.supports_atomic_replace() {
        connection
            .write_file(&target, &bytes)
            .context("failed to write remote Gale manifest")?;
        return Ok(());
    }

    connection
        .write_file(&temporary, &bytes)
        .context("failed to write remote Gale manifest")?;

    replace_remote_file(connection, &temporary, &target)
        .context("failed to replace remote Gale manifest")?;

    Ok(())
}

/// Moves an uploaded temporary file over its target as safely as the protocol
/// allows.
fn replace_remote_file(
    connection: &mut RemoteConnection,
    temporary: &RemotePath,
    target: &RemotePath,
) -> Result<()> {
    // SFTP servers usually support an atomic overwrite rename — that is the
    // whole job when it works.
    if connection.rename_file(temporary, target, true).is_ok() {
        return Ok(());
    }

    // Otherwise (FTP and older SFTP servers) move the existing file aside
    // first, so a failure mid-swap never leaves the deployment with neither
    // the old nor the new file.
    let backup = target.with_suffix(".gale-backup");
    let target_exists = connection.file_exists(target)?;

    if target_exists {
        if connection.file_exists(&backup)? {
            connection
                .remove_file(&backup)
                .context("failed to clear stale remote backup")?;
        }
        if let Err(err) = connection.rename_file(target, &backup, false) {
            let _ = connection.remove_file(temporary);
            return Err(err).context("failed to back up existing remote file");
        }
    }

    if let Err(err) = connection.rename_file(temporary, target, false) {
        if target_exists {
            let _ = connection.rename_file(&backup, target, false);
        }
        let _ = connection.remove_file(temporary);
        return Err(err).context("failed to move uploaded file into place");
    }

    if target_exists {
        let _ = connection.remove_file(&backup);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::game::mod_loader::{ModLoader, ModLoaderKind};

    fn spec() -> DeploymentSpec {
        DeploymentSpec::for_loader(&ModLoader {
            package_name: None,
            file_target: None,
            kind: ModLoaderKind::BepInEx {
                extra_subdirs: Vec::new(),
            },
        })
        .unwrap()
    }

    fn path(value: &str) -> DeployPathBuf {
        DeployPathBuf::new(value).unwrap()
    }

    fn local(dir: &Path, host_managed: bool) -> LocalDeployment {
        collect_local_files(dir, &spec(), host_managed).unwrap()
    }

    fn plan(
        local: &LocalDeployment,
        previous: DeploymentManifest,
        remote_files: &[&str],
        remote_dirs: &[&str],
    ) -> DeploymentPlan {
        build_plan(
            local,
            previous,
            remote_files.iter().map(|p| path(p)).collect(),
            remote_dirs.iter().map(|p| path(p)).collect(),
            &spec(),
        )
        .unwrap()
    }

    fn manifest_of(local: &LocalDeployment) -> DeploymentManifest {
        DeploymentManifest {
            version: manifest::VERSION,
            files: local
                .files
                .iter()
                .map(|(path, file)| (path.clone(), file.manifest.clone()))
                .collect(),
        }
    }

    #[test]
    fn serializes_frontend_field_names() {
        let value = serde_json::to_value(DeploymentResult::Deployed {
            fingerprint: Some("SHA256:test".to_owned()),
            uploaded_files: 3,
            uploaded_bytes: 42,
            removed_files: 1,
            unchanged_files: 2,
            cleanup_warnings: vec!["Could not remove locked.dll: permission denied".to_owned()],
        })
        .unwrap();

        assert_eq!(value["uploadedFiles"], 3);
        assert_eq!(value["uploadedBytes"], 42);
        assert_eq!(value["removedFiles"], 1);
        assert_eq!(value["unchangedFiles"], 2);
        assert_eq!(value["cleanupWarnings"].as_array().unwrap().len(), 1);

        let preview = serde_json::to_value(DeploymentPreviewResult::Preview {
            fingerprint: None,
            upload_files: vec!["BepInEx/plugins/Test.dll".to_owned()],
            upload_bytes: 42,
            remove_files: vec!["BepInEx/plugins/Old.dll".to_owned()],
            unchanged_files: 2,
        })
        .unwrap();
        assert_eq!(preview["status"], "preview");
        assert_eq!(preview["uploadBytes"], 42);
        assert_eq!(preview["removeFiles"][0], "BepInEx/plugins/Old.dll");

        let progress = serde_json::to_value(DeploymentProgress {
            completed: 1,
            total: 3,
            path: path("BepInEx/plugins/Test.dll"),
            operation: DeploymentOperation::Upload,
        })
        .unwrap();
        assert_eq!(progress["operation"], "upload");
        assert_eq!(progress["path"], "BepInEx/plugins/Test.dll");
    }

    #[test]
    fn first_deploy_uploads_loader_and_mods() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("BepInEx/core")).unwrap();
        fs::create_dir_all(dir.path().join("BepInEx/plugins/Example")).unwrap();
        fs::write(dir.path().join("BepInEx/core/bepinex.dll"), b"core").unwrap();
        fs::write(dir.path().join("doorstop_config.ini"), b"doorstop").unwrap();
        fs::write(
            dir.path().join("BepInEx/plugins/Example/Example.dll"),
            b"plugin",
        )
        .unwrap();

        let local = local(dir.path(), false);
        let plan = plan(&local, DeploymentManifest::default(), &[], &[]);

        assert_eq!(plan.removals, Vec::<DeployPathBuf>::new());
        assert!(plan.uploads.contains(&path("BepInEx/core/bepinex.dll")));
        assert!(plan.uploads.contains(&path("doorstop_config.ini")));
        assert!(
            plan.uploads
                .contains(&path("BepInEx/plugins/Example/Example.dll"))
        );
    }

    /// The review bug: when a fresh deployment uploads the loader, the next
    /// deployment must not treat those same files as host-managed and delete
    /// them. Ownership comes from the manifest, not from remote presence.
    #[test]
    fn gale_deployed_loader_is_not_removed_on_next_deploy() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("BepInEx/core")).unwrap();
        fs::create_dir_all(dir.path().join("BepInEx/plugins/Example")).unwrap();
        fs::write(dir.path().join("BepInEx/core/bepinex.dll"), b"core").unwrap();
        fs::write(dir.path().join("doorstop_config.ini"), b"doorstop").unwrap();
        fs::write(
            dir.path().join("BepInEx/plugins/Example/Example.dll"),
            b"plugin",
        )
        .unwrap();

        let spec = spec();

        // First deploy to a host without a loader: everything uploads and the
        // manifest records the loader payload as Gale-owned.
        let first = collect_local_files(dir.path(), &spec, false).unwrap();
        let previous = manifest_of(&first);
        assert!(spec.owns_loader(&previous));

        // Second deploy: the remote now has the loader files *we* uploaded,
        // so host_managed resolves to false and collection still includes
        // the core files — they are neither re-excluded nor removed.
        let host_managed =
            is_host_managed(RemoteLayout::Standard, spec.owns_loader(&previous), true);
        assert!(!host_managed);

        let second = collect_local_files(dir.path(), &spec, host_managed).unwrap();
        let plan = plan(
            &second,
            previous,
            &[
                "BepInEx/core/bepinex.dll",
                "doorstop_config.ini",
                "BepInEx/plugins/Example/Example.dll",
            ],
            &[],
        );

        assert!(plan.removals.is_empty());
        assert!(plan.uploads.is_empty());
    }

    #[test]
    fn restricted_layout_is_always_host_managed() {
        assert!(is_host_managed(RemoteLayout::MirrorRoot, false, false));
        assert!(is_host_managed(RemoteLayout::MirrorRoot, true, false));
        assert!(is_host_managed(RemoteLayout::Standard, false, true));
        assert!(!is_host_managed(RemoteLayout::Standard, true, true));
        assert!(!is_host_managed(RemoteLayout::Standard, false, false));
    }

    #[test]
    fn host_managed_layout_deploys_only_mirror_dirs() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("BepInEx/core")).unwrap();
        fs::create_dir_all(dir.path().join("BepInEx/plugins/Example")).unwrap();
        fs::write(dir.path().join("BepInEx/core/bepinex.dll"), b"core").unwrap();
        fs::write(dir.path().join("doorstop_config.ini"), b"doorstop").unwrap();
        fs::write(
            dir.path().join("BepInEx/plugins/Example/Example.dll"),
            b"plugin",
        )
        .unwrap();

        let local = local(dir.path(), true);

        assert!(
            local
                .files
                .contains_key(&path("BepInEx/plugins/Example/Example.dll"))
        );
        assert!(!local.files.contains_key(&path("BepInEx/core/bepinex.dll")));
        assert!(!local.files.contains_key(&path("doorstop_config.ini")));
    }

    #[test]
    fn removals_stay_within_gale_ownership() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("BepInEx/plugins/Example")).unwrap();
        fs::write(
            dir.path().join("BepInEx/plugins/Example/Keep.dll"),
            b"plugin",
        )
        .unwrap();

        let local = local(dir.path(), false);
        let remote_files = [
            "BepInEx/plugins/Example/Keep.dll",
            "BepInEx/plugins/Example/Delete.dll",
            "BepInEx/plugins/Other/Mod.dll",
        ];

        let plan = plan(&local, DeploymentManifest::default(), &remote_files, &[]);

        // Remote extras inside mirror dirs are removed; the file we still
        // deploy is kept.
        assert_eq!(
            plan.removals,
            [
                path("BepInEx/plugins/Example/Delete.dll"),
                path("BepInEx/plugins/Other/Mod.dll")
            ]
        );
        assert_eq!(plan.uploads, [path("BepInEx/plugins/Example/Keep.dll")]);
    }

    #[test]
    fn previous_manifest_paths_are_removed_when_stale() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("BepInEx/plugins/Example")).unwrap();
        fs::write(
            dir.path().join("BepInEx/plugins/Example/Keep.dll"),
            b"plugin",
        )
        .unwrap();

        let local = local(dir.path(), false);
        let mut previous = manifest_of(&local);
        previous.files.insert(
            path("doorstop_config.ini"),
            ManifestEntry {
                hash: "x".into(),
                size: 1,
            },
        );

        // The local doorstop_config.ini vanished; it was ours, so it gets
        // removed even though it sits outside the mirror dirs.
        let plan = plan(&local, previous, &["BepInEx/plugins/Example/Keep.dll"], &[]);

        assert_eq!(plan.removals, [path("doorstop_config.ini")]);
    }

    #[test]
    fn disabled_files_become_remote_deletions() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("BepInEx/plugins/Example")).unwrap();
        fs::write(
            dir.path().join("BepInEx/plugins/Example/Enabled.dll"),
            b"enabled",
        )
        .unwrap();
        fs::write(
            dir.path().join("BepInEx/plugins/Example/Disabled.dll.old"),
            b"disabled",
        )
        .unwrap();

        let local = local(dir.path(), false);

        assert!(
            local
                .files
                .contains_key(&path("BepInEx/plugins/Example/Enabled.dll"))
        );
        assert!(local.files.keys().all(|p| !p.as_str().ends_with(".old")));

        // The enabled counterpart is not deployed, so the mirror rules remove
        // the still-present remote copy.
        let plan = plan(
            &local,
            DeploymentManifest::default(),
            &["BepInEx/plugins/Example/Disabled.dll"],
            &[],
        );
        assert_eq!(
            plan.removals,
            [path("BepInEx/plugins/Example/Disabled.dll")]
        );
    }

    #[test]
    fn manifest_path_maps_to_remote_layout() {
        let spec = spec();
        let mapper = |layout| RemoteMapper {
            spec: &spec,
            base: RemotePathBuf::new("/srv").unwrap(),
            layout,
        };

        let standard = mapper(RemoteLayout::Standard).remote_path(&spec.manifest_path);
        assert_eq!(
            standard.as_str(),
            "/srv/BepInEx/config/.gale-server-manifest.json"
        );

        let restricted = mapper(RemoteLayout::MirrorRoot).remote_path(&spec.manifest_path);
        assert_eq!(
            restricted.as_str(),
            "/srv/config/.gale-server-manifest.json"
        );
    }

    #[test]
    fn removes_empty_remote_mod_directories_deepest_first() {
        let dir = tempfile::tempdir().unwrap();
        let local = local(dir.path(), false);
        let remote_files = ["BepInEx/plugins/HearthBelow/lib/Old.dll"];
        let remote_dirs = [
            "BepInEx/plugins/HearthBelow",
            "BepInEx/plugins/HearthBelow/lib",
        ];

        let plan = plan(
            &local,
            DeploymentManifest::default(),
            &remote_files,
            &remote_dirs,
        );

        assert_eq!(
            plan.directory_removals,
            [
                path("BepInEx/plugins/HearthBelow/lib"),
                path("BepInEx/plugins/HearthBelow")
            ]
        );
    }

    #[test]
    fn keeps_package_manifest_files() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("BepInEx/plugins/Example")).unwrap();
        fs::write(
            dir.path().join("BepInEx/plugins/Example/manifest.json"),
            b"{}",
        )
        .unwrap();
        fs::write(dir.path().join("BepInEx/plugins/Example/icon.png"), b"icon").unwrap();

        let local = local(dir.path(), false);

        assert!(
            local
                .files
                .contains_key(&path("BepInEx/plugins/Example/manifest.json")),
            "mods can depend on each other's manifest.json"
        );
        assert!(
            !local
                .files
                .contains_key(&path("BepInEx/plugins/Example/icon.png"))
        );
    }
}
