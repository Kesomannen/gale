use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use eyre::{Context, OptionExt, Result, bail, ensure};
use serde::Serialize;
use tracing::{info, warn};
use walkdir::{DirEntry, WalkDir};

use super::{
    config::RemoteServerSettings,
    manifest::{self, DeploymentManifest, ManifestEntry},
    remote::{self, ConnectionAttempt},
};

const MAX_MANIFEST_BYTES: u64 = 8 * 1024 * 1024;
const MIRROR_DIRECTORIES: [&str; 3] = ["BepInEx/plugins", "BepInEx/patchers", "BepInEx/config"];
const TRANSFER_ATTEMPTS: usize = 3;
const RETRY_DELAY: Duration = Duration::from_millis(500);
const HASH_BUFFER_SIZE: usize = 64 * 1024;

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
        fingerprint: Option<String>,
        upload_files: Vec<String>,
        upload_bytes: u64,
        remove_files: Vec<String>,
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

struct LocalFile {
    path: PathBuf,
    manifest: ManifestEntry,
}

struct LocalDeployment {
    files: BTreeMap<String, LocalFile>,
    disabled: BTreeSet<String>,
}

#[derive(Default)]
struct ServerFileFilter {
    client_only_mods: BTreeSet<String>,
    tracked_files: BTreeSet<PathBuf>,
    preserve_server_bepinex: bool,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PackageState {
    files: Vec<PathBuf>,
}

struct DeploymentPlan {
    current: DeploymentManifest,
    uploads: Vec<String>,
    upload_bytes: u64,
    removals: Vec<String>,
    directory_removals: Vec<String>,
    unchanged_files: usize,
}

#[derive(Clone, Copy)]
struct RemoteLayout {
    bepinex_at_root: bool,
}

impl RemoteLayout {
    fn path(self, base: &str, relative: &str) -> String {
        let relative = if self.bepinex_at_root {
            relative.strip_prefix("BepInEx/").unwrap_or(relative)
        } else {
            relative
        };
        remote_path(base, relative)
    }

    fn manifest_path(self, base: &str) -> String {
        if self.bepinex_at_root {
            self.path(base, &format!("BepInEx/config/{}", manifest::FILE_NAME))
        } else {
            remote_path(base, manifest::FILE_NAME)
        }
    }
}

pub fn preview(
    profile_dir: &Path,
    client_only_mods: BTreeSet<String>,
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
    let (local, plan, _) =
        prepare_deployment(&mut connection, profile_dir, settings, &client_only_mods)?;
    drop(local);

    Ok(DeploymentPreviewResult::Preview {
        fingerprint: connection.fingerprint,
        upload_files: plan.uploads,
        upload_bytes: plan.upload_bytes,
        remove_files: plan
            .removals
            .into_iter()
            .chain(
                plan.directory_removals
                    .into_iter()
                    .map(|path| format!("{path}/")),
            )
            .collect(),
        unchanged_files: plan.unchanged_files,
        skipped_client_only_mods: client_only_mods.into_iter().collect(),
    })
}

pub fn deploy<F>(
    profile_dir: &Path,
    client_only_mods: BTreeSet<String>,
    settings: &RemoteServerSettings,
    password: &str,
    mut report: F,
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

    let (local, plan, layout) =
        prepare_deployment(&mut connection, profile_dir, settings, &client_only_mods)?;
    let total = plan.uploads.len() + plan.removals.len() + plan.directory_removals.len();
    let mut completed = 0;

    let mut removed_files = 0;
    let mut cleanup_warnings = Vec::new();
    for relative in &plan.removals {
        let remote_path = layout.path(&settings.server_directory, relative);

        match connection.remove_file(&remote_path) {
            Ok(true) => removed_files += 1,
            Ok(false) => {}
            Err(error) => {
                let warning = format!("Could not remove {relative}: {error}");
                warn!("{warning}");
                cleanup_warnings.push(warning);
            }
        }
        completed += 1;
        report(DeploymentProgress {
            completed,
            total,
            path: relative.clone(),
            operation: DeploymentOperation::Remove,
        });
    }

    for relative in &plan.directory_removals {
        if let Err(error) =
            connection.remove_directory(&layout.path(&settings.server_directory, relative))
        {
            let warning = format!("Could not remove {relative}/: {error}");
            warn!("{warning}");
            cleanup_warnings.push(warning);
        }
        completed += 1;
        report(DeploymentProgress {
            completed,
            total,
            path: format!("{relative}/"),
            operation: DeploymentOperation::Remove,
        });
    }

    let mut ensured_directories = BTreeSet::new();
    for relative in &plan.uploads {
        let file = &local.files[relative];
        upload_file_with_retry(
            &mut connection,
            settings,
            password,
            layout,
            &settings.server_directory,
            relative,
            file,
            &mut ensured_directories,
        )?;
        completed += 1;
        report(DeploymentProgress {
            completed,
            total,
            path: relative.clone(),
            operation: DeploymentOperation::Upload,
        });
    }

    write_manifest_with_retry(
        &mut connection,
        settings,
        password,
        layout,
        &settings.server_directory,
        &plan.current,
    )?;

    Ok(DeploymentResult::Deployed {
        fingerprint: connection.fingerprint,
        uploaded_files: plan.uploads.len(),
        uploaded_bytes: plan.upload_bytes,
        removed_files,
        unchanged_files: plan.unchanged_files,
        skipped_client_only_mods: client_only_mods.into_iter().collect(),
        cleanup_warnings,
    })
}

fn prepare_deployment(
    connection: &mut remote::RemoteConnection,
    profile_dir: &Path,
    settings: &RemoteServerSettings,
    client_only_mods: &BTreeSet<String>,
) -> Result<(LocalDeployment, DeploymentPlan, RemoteLayout)> {
    connection
        .check_directory(&settings.server_directory)
        .with_context(|| {
            format!(
                "remote server directory '{}' could not be accessed",
                settings.server_directory
            )
        })?;
    let standard_layout = connection
        .directory_exists(&remote_path(&settings.server_directory, "BepInEx/plugins"))
        .context("failed to check for a server-managed BepInEx installation")?;
    let bepinex_at_root = !standard_layout
        && connection
            .directory_exists(&remote_path(&settings.server_directory, "plugins"))
            .context("failed to check for a restricted BepInEx server layout")?;
    let server_has_bepinex = standard_layout
        || bepinex_at_root
        || connection
            .directory_exists(&remote_path(&settings.server_directory, "BepInEx/core"))
            .context("failed to check for a server-managed BepInEx installation")?;
    let layout = RemoteLayout { bepinex_at_root };
    let filter = ServerFileFilter::load(profile_dir, client_only_mods, server_has_bepinex);
    let local = collect_local_files(profile_dir, &filter)?;
    let previous = read_remote_manifest(connection, &settings.server_directory, layout)?;
    ensure!(
        previous.version == manifest::VERSION,
        "unsupported remote Gale manifest version {}",
        previous.version
    );
    let (remote_mirror_files, remote_mirror_directories) =
        collect_remote_mirror_files(connection, &settings.server_directory, layout)?;
    let legacy_old_files = if previous.old_files_cleaned {
        BTreeSet::new()
    } else {
        find_legacy_old_files(connection, &settings.server_directory, &local)?
    };
    let plan = build_plan(
        &local,
        previous,
        legacy_old_files,
        remote_mirror_files,
        remote_mirror_directories,
    )?;
    info!(
        uploads = plan.uploads.len(),
        removals = plan.removals.len(),
        unchanged = plan.unchanged_files,
        upload_bytes = plan.upload_bytes,
        "prepared remote deployment"
    );
    Ok((local, plan, layout))
}

fn collect_remote_mirror_files(
    connection: &mut remote::RemoteConnection,
    base: &str,
    layout: RemoteLayout,
) -> Result<(BTreeSet<String>, BTreeSet<String>)> {
    let mut files = BTreeSet::new();
    let mut directories = BTreeSet::new();
    for directory in MIRROR_DIRECTORIES {
        collect_remote_files(
            connection,
            base,
            directory,
            layout,
            &mut files,
            &mut directories,
        )?;
    }
    Ok((files, directories))
}

fn collect_remote_files(
    connection: &mut remote::RemoteConnection,
    base: &str,
    directory: &str,
    layout: RemoteLayout,
    files: &mut BTreeSet<String>,
    directories: &mut BTreeSet<String>,
) -> Result<()> {
    let remote_directory = layout.path(base, directory);
    for entry in connection
        .list_directory_entries(&remote_directory)
        .with_context(|| format!("failed to inspect remote directory {remote_directory}"))?
    {
        ensure!(
            !entry.name.is_empty()
                && entry.name != "."
                && entry.name != ".."
                && !entry.name.contains(['/', '\\']),
            "remote directory contains an unsafe name: {}",
            entry.name
        );
        let relative = join_relative(directory, &entry.name);
        if entry.is_directory {
            directories.insert(relative.clone());
            collect_remote_files(connection, base, &relative, layout, files, directories)?;
        } else if relative != format!("BepInEx/config/{}", manifest::FILE_NAME)
            && relative != format!("BepInEx/config/{}.tmp", manifest::FILE_NAME)
        {
            files.insert(relative);
        }
    }
    Ok(())
}

fn find_legacy_old_files(
    connection: &mut remote::RemoteConnection,
    base: &str,
    local: &LocalDeployment,
) -> Result<BTreeSet<String>> {
    let candidates = local
        .files
        .keys()
        .filter(|path| !is_mirrored(path))
        .map(|path| format!("{path}.old"))
        .collect::<BTreeSet<_>>();
    let directories = candidates
        .iter()
        .map(|path| path.rsplit_once('/').map_or("", |(directory, _)| directory))
        .collect::<BTreeSet<_>>();
    let mut found = BTreeSet::new();
    let base = base.trim_end_matches('/');

    for directory in directories {
        let remote_directory = if directory.is_empty() {
            if base.is_empty() { "/" } else { base }.to_owned()
        } else {
            join_remote(base, directory)
        };
        for name in connection
            .list_directory(&remote_directory)
            .with_context(|| format!("failed to inspect remote directory {remote_directory}"))?
        {
            let relative = if directory.is_empty() {
                name
            } else {
                join_remote(directory, &name)
            };
            if candidates.contains(&relative) {
                found.insert(relative);
            }
        }
    }

    Ok(found)
}

fn build_plan(
    local: &LocalDeployment,
    previous: DeploymentManifest,
    legacy_old_files: BTreeSet<String>,
    remote_mirror_files: BTreeSet<String>,
    remote_mirror_directories: BTreeSet<String>,
) -> Result<DeploymentPlan> {
    let current = DeploymentManifest {
        version: manifest::VERSION,
        old_files_cleaned: true,
        files: local
            .files
            .iter()
            .map(|(path, file)| (path.clone(), file.manifest.clone()))
            .collect(),
    };
    let uploads = local
        .files
        .iter()
        .filter(|(path, file)| {
            previous.files.get(*path) != Some(&file.manifest)
                || (is_mirrored(path) && !remote_mirror_files.contains(*path))
        })
        .map(|(path, _)| path.clone())
        .collect::<Vec<_>>();
    let upload_bytes = uploads
        .iter()
        .map(|path| local.files[path].manifest.size)
        .sum();
    let unchanged_files = local.files.len() - uploads.len();
    let mut removals = remote_mirror_files
        .into_iter()
        .filter(|path| !current.files.contains_key(path))
        .collect::<BTreeSet<_>>();
    removals.extend(
        previous
            .files
            .keys()
            .filter(|path| !is_mirrored(path))
            .cloned(),
    );
    removals.extend(
        local
            .disabled
            .iter()
            .filter(|path| !is_mirrored(path))
            .cloned(),
    );
    removals.extend(
        local
            .disabled
            .iter()
            .map(|path| format!("{path}.old"))
            .filter(|path| !is_mirrored(path)),
    );
    removals.extend(legacy_old_files);
    removals.retain(|path| !current.files.contains_key(path));
    let mut directory_removals = remote_mirror_directories
        .into_iter()
        .filter(|directory| {
            let prefix = format!("{directory}/");
            !current.files.keys().any(|path| path.starts_with(&prefix))
        })
        .collect::<Vec<_>>();
    directory_removals.sort_by_key(|path| std::cmp::Reverse(path.matches('/').count()));
    for path in &removals {
        ensure!(
            manifest::is_safe_relative_path(path),
            "remote Gale manifest contains an unsafe path: {path}"
        );
    }

    Ok(DeploymentPlan {
        current,
        uploads,
        upload_bytes,
        removals: removals.into_iter().collect(),
        directory_removals,
        unchanged_files,
    })
}

fn is_mirrored(path: &str) -> bool {
    MIRROR_DIRECTORIES
        .iter()
        .any(|directory| path == *directory || path.starts_with(&format!("{directory}/")))
}

impl ServerFileFilter {
    fn load(
        profile_dir: &Path,
        client_only_mods: &BTreeSet<String>,
        preserve_server_bepinex: bool,
    ) -> Self {
        let mut tracked_files = BTreeSet::new();
        for name in client_only_mods {
            let path = profile_dir.join("_state").join(name).with_extension("json");
            if let Ok(state) = crate::util::fs::read_json::<PackageState>(&path) {
                tracked_files.extend(state.files);
            }
        }
        Self {
            client_only_mods: client_only_mods.clone(),
            tracked_files,
            preserve_server_bepinex,
        }
    }

    fn excludes(&self, relative: &Path) -> bool {
        if relative.as_os_str().is_empty() {
            return false;
        }
        if self.preserve_server_bepinex {
            let relative = relative.to_string_lossy().replace('\\', "/");
            if !is_mirrored(&relative)
                && !MIRROR_DIRECTORIES
                    .iter()
                    .any(|directory| directory.starts_with(&format!("{relative}/")))
            {
                return true;
            }
        }
        if self.tracked_files.contains(relative) {
            return true;
        }
        let parts = relative
            .components()
            .filter_map(|part| part.as_os_str().to_str())
            .collect::<Vec<_>>();
        parts.len() >= 3
            && parts[0].eq_ignore_ascii_case("BepInEx")
            && parts[1].eq_ignore_ascii_case("plugins")
            && self
                .client_only_mods
                .iter()
                .any(|name| parts[2].eq_ignore_ascii_case(name))
    }
}

fn collect_local_files(profile_dir: &Path, filter: &ServerFileFilter) -> Result<LocalDeployment> {
    let mut files = BTreeMap::new();
    let mut disabled = BTreeSet::new();

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
        if is_disabled(relative) {
            disabled.insert(relative_path(&enabled_path(relative))?);
            continue;
        }
        let relative = relative_path(relative)?;
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

    Ok(LocalDeployment { files, disabled })
}

fn is_disabled(path: &Path) -> bool {
    path.extension().and_then(|extension| extension.to_str()) == Some("old")
}

fn enabled_path(path: &Path) -> PathBuf {
    let mut path = path.to_path_buf();
    while is_disabled(&path) {
        path.set_extension("");
    }
    path
}

fn is_excluded(entry: &DirEntry, profile_dir: &Path, filter: &ServerFileFilter) -> bool {
    if entry
        .path()
        .strip_prefix(profile_dir)
        .is_ok_and(|relative| filter.excludes(relative))
    {
        return true;
    }
    if entry.depth() == 1
        && matches!(
            entry.file_name().to_str(),
            Some("profile.json" | "mods.yml" | "snapshots" | "_state" | manifest::FILE_NAME)
        )
    {
        return true;
    }

    let components = entry
        .path()
        .components()
        .filter_map(|component| component.as_os_str().to_str())
        .collect::<Vec<_>>();
    if components.windows(2).any(|parts| {
        parts[0].eq_ignore_ascii_case("BepInEx")
            && matches!(
                parts[1].to_ascii_lowercase().as_str(),
                "cache" | "dumpedassemblies" | "interop"
            )
    }) {
        return true;
    }

    let comparable_path = enabled_path(entry.path());
    let Some(name) = comparable_path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    if name.eq_ignore_ascii_case("LogOutput.log") {
        return true;
    }

    let plugin_index = components.windows(2).position(|parts| {
        parts[0].eq_ignore_ascii_case("BepInEx") && parts[1].eq_ignore_ascii_case("plugins")
    });
    plugin_index.is_some_and(|index| components.len() == index + 4)
        && ["README.md", "CHANGELOG.md", "icon.png", "manifest.json"]
            .iter()
            .any(|metadata| name.eq_ignore_ascii_case(metadata))
}

fn relative_path(path: &Path) -> Result<String> {
    let value = path
        .components()
        .map(|component| component.as_os_str().to_str())
        .collect::<Option<Vec<_>>>()
        .ok_or_eyre("profile contains a non-Unicode path")?
        .join("/");

    ensure!(
        manifest::is_safe_relative_path(&value),
        "profile contains an unsafe path: {value}"
    );

    Ok(value)
}

fn hash_file(path: &Path) -> Result<String> {
    let mut reader = BufReader::new(
        File::open(path).with_context(|| format!("failed to open {}", path.display()))?,
    );
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0_u8; HASH_BUFFER_SIZE];

    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    Ok(hasher.finalize().to_hex().to_string())
}

fn upload_file(
    connection: &mut remote::RemoteConnection,
    layout: RemoteLayout,
    base: &str,
    relative: &str,
    file: &LocalFile,
    ensured_directories: &mut BTreeSet<String>,
) -> Result<()> {
    ensure_remote_parents(connection, layout, base, relative, ensured_directories)?;

    let target = layout.path(base, relative);
    let temporary = layout.path(base, &format!("{relative}.gale-upload"));
    let bytes = std::fs::read(&file.path)
        .with_context(|| format!("failed to open {}", file.path.display()))?;
    let uploaded_hash = blake3::hash(&bytes).to_hex().to_string();
    if uploaded_hash != file.manifest.hash {
        bail!("profile file changed during deployment: {relative}");
    }

    if !connection.supports_atomic_replace() {
        connection
            .write_file(&target, &bytes)
            .with_context(|| format!("failed to upload remote file {relative}"))?;
        return Ok(());
    }

    connection
        .write_file(&temporary, &bytes)
        .with_context(|| format!("failed to create remote temporary file for {relative}"))?;

    replace_remote_file(connection, &temporary, &target)
        .with_context(|| format!("failed to replace remote file {relative}"))?;

    Ok(())
}

fn upload_file_with_retry(
    connection: &mut remote::RemoteConnection,
    settings: &RemoteServerSettings,
    password: &str,
    layout: RemoteLayout,
    base: &str,
    relative: &str,
    file: &LocalFile,
    ensured_directories: &mut BTreeSet<String>,
) -> Result<()> {
    let mut last_error = None;
    for attempt in 1..=TRANSFER_ATTEMPTS {
        match upload_file(
            connection,
            layout,
            base,
            relative,
            file,
            ensured_directories,
        ) {
            Ok(()) => return Ok(()),
            Err(error) => {
                last_error = Some(error);
                if attempt == TRANSFER_ATTEMPTS {
                    break;
                }
                warn!(
                    path = relative,
                    attempt,
                    "upload failed; reconnecting"
                );
                thread::sleep(RETRY_DELAY * attempt as u32);
                *connection = reconnect(settings, password)?;
                ensured_directories.clear();
            }
        }
    }
    Err(last_error.unwrap())
}

fn write_manifest_with_retry(
    connection: &mut remote::RemoteConnection,
    settings: &RemoteServerSettings,
    password: &str,
    layout: RemoteLayout,
    base: &str,
    deployment: &DeploymentManifest,
) -> Result<()> {
    let mut last_error = None;
    for attempt in 1..=TRANSFER_ATTEMPTS {
        match write_remote_manifest(connection, layout, base, deployment) {
            Ok(()) => return Ok(()),
            Err(error) => {
                last_error = Some(error);
                if attempt == TRANSFER_ATTEMPTS {
                    break;
                }
                warn!(
                    attempt,
                    "manifest upload failed; reconnecting"
                );
                thread::sleep(RETRY_DELAY * attempt as u32);
                *connection = reconnect(settings, password)?;
            }
        }
    }
    Err(last_error.unwrap())
}

fn reconnect(settings: &RemoteServerSettings, password: &str) -> Result<remote::RemoteConnection> {
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

fn ensure_remote_parents(
    connection: &mut remote::RemoteConnection,
    layout: RemoteLayout,
    base: &str,
    relative: &str,
    ensured_directories: &mut BTreeSet<String>,
) -> Result<()> {
    let relative = if layout.bepinex_at_root {
        relative.strip_prefix("BepInEx/").unwrap_or(relative)
    } else {
        relative
    };
    let mut current = base.trim_end_matches('/').to_owned();
    let components = relative.split('/').collect::<Vec<_>>();

    for component in components.iter().take(components.len().saturating_sub(1)) {
        current = join_remote(&current, component);
        if !ensured_directories.insert(current.clone()) {
            continue;
        }
        connection
            .ensure_directory(&current)
            .with_context(|| format!("failed to create remote directory {current}"))?;
    }

    Ok(())
}

fn read_remote_manifest(
    connection: &mut remote::RemoteConnection,
    base: &str,
    layout: RemoteLayout,
) -> Result<DeploymentManifest> {
    let path = layout.manifest_path(base);
    let bytes = match connection.read_file(&path)? {
        Some(bytes) => bytes,
        None => return Ok(DeploymentManifest::default()),
    };
    ensure!(
        bytes.len() as u64 <= MAX_MANIFEST_BYTES,
        "remote Gale manifest is unexpectedly large"
    );

    serde_json::from_slice(&bytes).context("remote Gale manifest is invalid")
}

fn write_remote_manifest(
    connection: &mut remote::RemoteConnection,
    layout: RemoteLayout,
    base: &str,
    deployment: &DeploymentManifest,
) -> Result<()> {
    let target = layout.manifest_path(base);
    let temporary = format!("{target}.tmp");
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

fn replace_remote_file(
    connection: &mut remote::RemoteConnection,
    temporary: &str,
    target: &str,
) -> Result<()> {
    if connection.rename_file(temporary, target, true).is_ok() {
        return Ok(());
    }

    let target_exists = connection.file_exists(&target)?;
    let backup = format!("{target}.gale-backup");

    if target_exists {
        if connection.file_exists(&backup)? {
            connection
                .remove_file(&backup)
                .context("failed to clear stale remote backup")?;
        }
        if let Err(err) = connection.rename_file(&target, &backup, false) {
            let _ = connection.remove_file(&temporary);
            return Err(err).context("failed to back up existing remote file");
        }
    }

    if let Err(err) = connection.rename_file(&temporary, &target, false) {
        if target_exists {
            let _ = connection.rename_file(&backup, &target, false);
        }
        let _ = connection.remove_file(&temporary);
        return Err(err).context("failed to move uploaded file into place");
    }

    if target_exists {
        let _ = connection.remove_file(&backup);
    }

    Ok(())
}

fn remote_path(base: &str, relative: &str) -> String {
    join_remote(base.trim_end_matches('/'), relative)
}

fn join_remote(base: &str, relative: &str) -> String {
    if base.is_empty() {
        format!("/{relative}")
    } else {
        format!("{base}/{relative}")
    }
}

fn join_relative(base: &str, relative: &str) -> String {
    format!("{base}/{relative}")
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeSet, fs};

    use super::{
        DeploymentOperation, DeploymentPreviewResult, DeploymentProgress, DeploymentResult,
        RemoteLayout, ServerFileFilter, build_plan, collect_local_files,
    };
    use crate::profile::server::manifest::DeploymentManifest;

    #[test]
    fn serializes_frontend_field_names() {
        let value = serde_json::to_value(DeploymentResult::Deployed {
            fingerprint: Some("SHA256:test".to_owned()),
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
            fingerprint: None,
            upload_files: vec!["BepInEx/plugins/Test.dll".to_owned()],
            upload_bytes: 42,
            remove_files: vec!["BepInEx/plugins/Old.dll".to_owned()],
            unchanged_files: 2,
            skipped_client_only_mods: vec!["Author-ClientOnly".to_owned()],
        })
        .unwrap();
        assert_eq!(preview["status"], "preview");
        assert_eq!(preview["uploadBytes"], 42);
        assert_eq!(preview["removeFiles"][0], "BepInEx/plugins/Old.dll");

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
    fn maps_restricted_bepinex_roots() {
        let standard = RemoteLayout {
            bepinex_at_root: false,
        };
        let restricted = RemoteLayout {
            bepinex_at_root: true,
        };

        assert_eq!(
            standard.path("/", "BepInEx/plugins/Example.dll"),
            "/BepInEx/plugins/Example.dll"
        );
        assert_eq!(
            restricted.path("/", "BepInEx/plugins/Example.dll"),
            "/plugins/Example.dll"
        );
        assert_eq!(
            restricted.manifest_path("/"),
            "/config/.gale-server-manifest.json"
        );
    }

    #[test]
    fn disabled_files_become_remote_deletions() {
        let directory = tempfile::tempdir().unwrap();
        let plugin = directory.path().join("BepInEx/plugins/Example");
        fs::create_dir_all(&plugin).unwrap();
        fs::write(plugin.join("Enabled.dll"), b"enabled").unwrap();
        fs::write(plugin.join("Disabled.dll.old"), b"disabled").unwrap();

        let deployment =
            collect_local_files(directory.path(), &ServerFileFilter::default()).unwrap();

        assert!(
            deployment
                .files
                .contains_key("BepInEx/plugins/Example/Enabled.dll")
        );
        assert!(
            !deployment
                .files
                .contains_key("BepInEx/plugins/Example/Disabled.dll.old")
        );
        assert!(
            deployment
                .disabled
                .contains("BepInEx/plugins/Example/Disabled.dll")
        );
    }

    #[test]
    fn ignores_generated_files_and_package_metadata() {
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
        fs::write(plugin.join("manifest.json"), b"manifest").unwrap();
        fs::write(plugin.join("RuntimeAsset.json"), b"asset").unwrap();

        let deployment =
            collect_local_files(directory.path(), &ServerFileFilter::default()).unwrap();

        assert_eq!(deployment.files.len(), 1);
        assert!(
            deployment
                .files
                .contains_key("BepInEx/plugins/Example/RuntimeAsset.json")
        );
    }

    #[test]
    fn excludes_client_only_package_files() {
        let directory = tempfile::tempdir().unwrap();
        let client = directory.path().join("BepInEx/plugins/Author-ClientOnly");
        let shared = directory.path().join("BepInEx/plugins/Author-Shared");
        let patchers = directory.path().join("BepInEx/patchers");
        let state = directory.path().join("_state");
        fs::create_dir_all(&client).unwrap();
        fs::create_dir_all(&shared).unwrap();
        fs::create_dir_all(&patchers).unwrap();
        fs::create_dir_all(&state).unwrap();
        fs::write(client.join("Client.dll"), b"client").unwrap();
        fs::write(shared.join("Shared.dll"), b"shared").unwrap();
        fs::write(patchers.join("ClientPatcher.dll"), b"patcher").unwrap();
        fs::write(
            state.join("Author-ClientOnly.json"),
            r#"{"files":["BepInEx/patchers/ClientPatcher.dll"]}"#,
        )
        .unwrap();
        let client_only = ["Author-ClientOnly".to_owned()].into_iter().collect();
        let filter = ServerFileFilter::load(directory.path(), &client_only, false);

        let deployment = collect_local_files(directory.path(), &filter).unwrap();

        assert_eq!(deployment.files.len(), 1);
        assert!(
            deployment
                .files
                .contains_key("BepInEx/plugins/Author-Shared/Shared.dll")
        );
    }

    #[test]
    fn preserves_host_bepinex_files_when_core_is_installed() {
        let directory = tempfile::tempdir().unwrap();
        fs::create_dir_all(directory.path().join("BepInEx/plugins/Example")).unwrap();
        fs::create_dir_all(directory.path().join("BepInEx/patchers/Example")).unwrap();
        fs::write(directory.path().join("start_game_bepinex.sh"), b"start").unwrap();
        fs::write(directory.path().join(".doorstop_version"), b"doorstop").unwrap();
        fs::write(
            directory.path().join("BepInEx/plugins/Example/Example.dll"),
            b"plugin",
        )
        .unwrap();
        fs::write(
            directory
                .path()
                .join("BepInEx/patchers/Example/Example.Patcher.dll"),
            b"patcher",
        )
        .unwrap();
        let filter = ServerFileFilter::load(directory.path(), &BTreeSet::new(), true);

        let deployment = collect_local_files(directory.path(), &filter).unwrap();

        assert_eq!(deployment.files.len(), 2);
        assert!(
            deployment
                .files
                .contains_key("BepInEx/plugins/Example/Example.dll")
        );
        assert!(
            deployment
                .files
                .contains_key("BepInEx/patchers/Example/Example.Patcher.dll")
        );
    }

    #[test]
    fn cleans_legacy_old_files_only_once() {
        let directory = tempfile::tempdir().unwrap();
        let plugin = directory.path().join("BepInEx/plugins/Example");
        fs::create_dir_all(&plugin).unwrap();
        fs::write(plugin.join("Example.dll"), b"plugin").unwrap();
        let local = collect_local_files(directory.path(), &ServerFileFilter::default()).unwrap();

        let legacy: BTreeSet<String> = ["BepInEx/plugins/Example/Example.dll.old".to_owned()]
            .into_iter()
            .collect();
        let first = build_plan(
            &local,
            DeploymentManifest::default(),
            legacy.clone(),
            legacy,
            BTreeSet::new(),
        )
        .unwrap();
        assert!(
            first
                .removals
                .contains(&"BepInEx/plugins/Example/Example.dll.old".to_owned())
        );

        let second = build_plan(
            &local,
            first.current,
            BTreeSet::new(),
            ["BepInEx/plugins/Example/Example.dll".to_owned()]
                .into_iter()
                .collect(),
            BTreeSet::new(),
        )
        .unwrap();
        assert!(second.uploads.is_empty());
        assert!(second.removals.is_empty());
    }

    #[test]
    fn mirrored_directories_remove_only_remote_extras() {
        let directory = tempfile::tempdir().unwrap();
        let plugin = directory.path().join("BepInEx/plugins/Example");
        fs::create_dir_all(&plugin).unwrap();
        fs::write(plugin.join("Keep.dll"), b"plugin").unwrap();
        let local = collect_local_files(directory.path(), &ServerFileFilter::default()).unwrap();
        let remote = [
            "BepInEx/plugins/Example/Keep.dll".to_owned(),
            "BepInEx/plugins/Example/Delete.dll".to_owned(),
        ]
        .into_iter()
        .collect();

        let plan = build_plan(
            &local,
            DeploymentManifest::default(),
            BTreeSet::new(),
            remote,
            BTreeSet::new(),
        )
        .unwrap();

        assert_eq!(plan.removals, ["BepInEx/plugins/Example/Delete.dll"]);
    }

    #[test]
    fn reuploads_manifest_file_missing_from_server() {
        let directory = tempfile::tempdir().unwrap();
        let plugin = directory.path().join("BepInEx/plugins/Example");
        fs::create_dir_all(&plugin).unwrap();
        fs::write(plugin.join("Example.dll"), b"plugin").unwrap();
        let local = collect_local_files(directory.path(), &ServerFileFilter::default()).unwrap();
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
            BTreeSet::new(),
        )
        .unwrap();

        assert_eq!(plan.uploads, ["BepInEx/plugins/Example/Example.dll"]);
        assert_eq!(plan.unchanged_files, 0);
    }

    #[test]
    fn removal_preview_only_includes_files_found_on_server() {
        let directory = tempfile::tempdir().unwrap();
        let plugin = directory.path().join("BepInEx/plugins/Example");
        fs::create_dir_all(&plugin).unwrap();
        fs::write(plugin.join("Disabled.dll.old"), b"disabled").unwrap();
        let local = collect_local_files(directory.path(), &ServerFileFilter::default()).unwrap();
        let remote = ["BepInEx/plugins/Example/Disabled.dll".to_owned()]
            .into_iter()
            .collect();

        let plan = build_plan(
            &local,
            DeploymentManifest::default(),
            BTreeSet::new(),
            remote,
            BTreeSet::new(),
        )
        .unwrap();

        assert_eq!(plan.removals, ["BepInEx/plugins/Example/Disabled.dll"]);
    }

    #[test]
    fn removes_empty_remote_mod_directories_deepest_first() {
        let directory = tempfile::tempdir().unwrap();
        let local = collect_local_files(directory.path(), &ServerFileFilter::default()).unwrap();
        let remote_files = ["BepInEx/plugins/HearthBelow/lib/Old.dll".to_owned()]
            .into_iter()
            .collect();
        let remote_directories = [
            "BepInEx/plugins/HearthBelow".to_owned(),
            "BepInEx/plugins/HearthBelow/lib".to_owned(),
        ]
        .into_iter()
        .collect();

        let plan = build_plan(
            &local,
            DeploymentManifest::default(),
            BTreeSet::new(),
            remote_files,
            remote_directories,
        )
        .unwrap();

        assert_eq!(
            plan.directory_removals,
            [
                "BepInEx/plugins/HearthBelow/lib",
                "BepInEx/plugins/HearthBelow"
            ]
        );
    }
}
