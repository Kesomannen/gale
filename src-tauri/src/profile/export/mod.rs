use std::{
    collections::BTreeMap,
    fmt::Display,
    fs::{self, File},
    io::{self, Cursor, Seek, Write},
    path::{Component, Path, PathBuf},
    sync::LazyLock,
};

use base64::{Engine, prelude::BASE64_STANDARD};
use eyre::{Context, OptionExt, bail, ensure};
use globset::{Glob, GlobBuilder, GlobSet, GlobSetBuilder};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tracing::info;
use uuid::Uuid;
use walkdir::WalkDir;
use zip::{ZipWriter, write::SimpleFileOptions};

use super::{Profile, Result, install::ModInstall};
use crate::thunderstore::Backend;
use crate::{
    game::Game,
    state::ManagerExt,
    thunderstore::{LegacyProfileCreateResponse, PackageIdent, Thunderstore, VersionIdent},
};

mod changelog;
pub mod commands;
pub mod modpack;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProfileManifest {
    #[serde(rename = "profileName")]
    pub name: String,
    pub mods: Vec<R2Mod>,
    #[serde(default, rename = "community")]
    pub game: Option<String>,
    #[serde(default, rename = "ignoredUpdates")]
    pub ignored_version_updates: Vec<Uuid>,
    #[serde(default)]
    pub ignored_package_updates: Vec<Uuid>,
    #[serde(default, rename = "galeSync", skip_serializing_if = "Option::is_none")]
    pub sync: Option<SyncManifest>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct R2Mod {
    #[serde(rename = "name")]
    pub ident: PackageIdent,
    #[serde(alias = "versionNumber")]
    pub version: R2Version,
    pub enabled: bool,
    #[serde(default)]
    pub source: Backend,
}

impl R2Mod {
    pub fn version_ident(&self) -> VersionIdent {
        self.ident.with_version(&self.version)
    }

    pub fn to_install(&self, thunderstore: &Thunderstore) -> Result<ModInstall> {
        // Prefer backend, otherwise fallback to generic lookup
        let borrowed_mod = thunderstore
            .backend(self.source)
            .find_ident(&self.version_ident())
            .or_else(|_| thunderstore.find_ident(&self.version_ident()))?;

        Ok(ModInstall::new(borrowed_mod).with_state(self.enabled))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(try_from = "RawR2Version", into = "RawR2Version")]
pub struct R2Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub pre: semver::Prerelease,
    pub build: semver::BuildMetadata,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawR2Version {
    major: u64,
    minor: u64,
    patch: u64,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pre: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    build: String,
}

impl TryFrom<RawR2Version> for R2Version {
    type Error = semver::Error;

    fn try_from(raw: RawR2Version) -> Result<Self, Self::Error> {
        Ok(Self {
            major: raw.major,
            minor: raw.minor,
            patch: raw.patch,
            pre: semver::Prerelease::new(&raw.pre)?,
            build: semver::BuildMetadata::new(&raw.build)?,
        })
    }
}

impl From<R2Version> for RawR2Version {
    fn from(version: R2Version) -> Self {
        Self {
            major: version.major,
            minor: version.minor,
            patch: version.patch,
            pre: version.pre.as_str().to_owned(),
            build: version.build.as_str().to_owned(),
        }
    }
}

impl Display for R2Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if !self.pre.is_empty() {
            write!(f, "-{}", self.pre)?;
        }
        if !self.build.is_empty() {
            write!(f, "+{}", self.build)?;
        }
        Ok(())
    }
}

impl From<semver::Version> for R2Version {
    fn from(value: semver::Version) -> Self {
        Self {
            major: value.major,
            minor: value.minor,
            patch: value.patch,
            pre: value.pre,
            build: value.build,
        }
    }
}

impl From<R2Version> for semver::Version {
    fn from(value: R2Version) -> Self {
        Self {
            major: value.major,
            minor: value.minor,
            patch: value.patch,
            pre: value.pre,
            build: value.build,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SyncManifest {
    pub version: u32,
    pub mods_revision: ModRevision,
    pub config: BTreeMap<ConfigPath, SyncFileEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SyncFileEntry {
    pub hash: ContentHash,
}

/// DOS device names Windows reserves regardless of extension.
const DEVICE_NAMES: &[&str] = &[
    "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
    "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
];

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ConfigPath(String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ContentHash(String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ModRevision(String);

impl ConfigPath {
    pub fn as_path(&self) -> &Path {
        Path::new(&self.0)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for ConfigPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl TryFrom<String> for ConfigPath {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        ensure!(!value.is_empty(), "config path is empty");
        ensure!(
            !value.contains(['\\', '\0', ':', '<', '>', '|', '?', '*', '"']),
            "config path contains a forbidden character"
        );
        ensure!(
            !value.chars().any(char::is_control),
            "config path contains a control character"
        );
        // Path::components() collapses repeated separators, so check the raw
        // text for empty segments before normalizing
        ensure!(
            !value.split('/').any(|segment| segment.is_empty()),
            "config path contains an empty segment"
        );

        let path = Path::new(&value);

        ensure!(
            path.components()
                .all(|component| matches!(component, Component::Normal(_))),
            "config path must consist of normal components only"
        );
        ensure!(
            !value.eq_ignore_ascii_case("export.r2x"),
            "config path is the profile manifest"
        );
        ensure!(
            !path
                .components()
                .next()
                .and_then(|component| component.as_os_str().to_str())
                .is_some_and(|component| component.eq_ignore_ascii_case("_state")),
            "config path is inside the state directory"
        );

        // names Windows silently rewrites or reserves can't round-trip to a
        // subscriber's filesystem
        for component in path.components() {
            let name = component
                .as_os_str()
                .to_str()
                .ok_or_eyre("config path is not valid UTF-8")?;

            ensure!(
                !name.ends_with(['.', ' ']),
                "config path component ends with '.' or ' ': {name}"
            );

            let stem = name.split('.').next().unwrap_or(name);
            ensure!(
                !DEVICE_NAMES.contains(&stem.to_ascii_lowercase().as_str()),
                "config path contains a reserved device name: {name}"
            );
        }

        Ok(Self(value))
    }
}

impl TryFrom<&str> for ConfigPath {
    type Error = eyre::Report;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_owned())
    }
}

impl From<ConfigPath> for String {
    fn from(value: ConfigPath) -> Self {
        value.0
    }
}

fn is_valid_hash_str(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

impl ContentHash {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn from_hash(hash: blake3::Hash) -> Self {
        Self(hash.to_hex().to_string())
    }
}

impl TryFrom<String> for ContentHash {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        ensure!(is_valid_hash_str(&value), "invalid content hash");
        Ok(Self(value))
    }
}

impl TryFrom<&str> for ContentHash {
    type Error = eyre::Report;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_owned())
    }
}

impl From<ContentHash> for String {
    fn from(value: ContentHash) -> Self {
        value.0
    }
}

impl ModRevision {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn from_hash(hash: blake3::Hash) -> Self {
        Self(hash.to_hex().to_string())
    }
}

impl TryFrom<String> for ModRevision {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        ensure!(is_valid_hash_str(&value), "invalid revision hash");
        Ok(Self(value))
    }
}

impl TryFrom<&str> for ModRevision {
    type Error = eyre::Report;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_owned())
    }
}

impl From<ModRevision> for String {
    fn from(value: ModRevision) -> Self {
        value.0
    }
}

pub fn manifest_revision(manifest: &ProfileManifest) -> Result<ModRevision> {
    #[derive(Serialize)]
    struct RevisionMod<'a> {
        ident: &'a PackageIdent,
        major: u64,
        minor: u64,
        patch: u64,
        #[serde(skip_serializing_if = "str::is_empty")]
        pre: &'a str,
        #[serde(skip_serializing_if = "str::is_empty")]
        build: &'a str,
        enabled: bool,
        source: Backend,
    }

    #[derive(Serialize)]
    struct RevisionManifest<'a> {
        game: Option<&'a str>,
        mods: Vec<RevisionMod<'a>>,
        ignored_version_updates: Vec<Uuid>,
        ignored_package_updates: Vec<Uuid>,
    }

    let mut mods: Vec<RevisionMod> = manifest
        .mods
        .iter()
        .map(|r2_mod| RevisionMod {
            ident: &r2_mod.ident,
            major: r2_mod.version.major,
            minor: r2_mod.version.minor,
            patch: r2_mod.version.patch,
            pre: r2_mod.version.pre.as_str(),
            build: r2_mod.version.build.as_str(),
            enabled: r2_mod.enabled,
            source: r2_mod.source,
        })
        .collect();

    mods.sort_by(|a, b| {
        (
            a.source, a.ident, a.major, a.minor, a.patch, a.pre, a.build, a.enabled,
        )
            .cmp(&(
                b.source, b.ident, b.major, b.minor, b.patch, b.pre, b.build, b.enabled,
            ))
    });

    let mut ignored_version_updates = manifest.ignored_version_updates.clone();
    ignored_version_updates.sort();

    let mut ignored_package_updates = manifest.ignored_package_updates.clone();
    ignored_package_updates.sort();

    let canonical = RevisionManifest {
        game: manifest.game.as_deref(),
        mods,
        ignored_version_updates,
        ignored_package_updates,
    };

    let bytes = serde_json::to_vec(&canonical).context("failed to serialize revision manifest")?;

    Ok(ModRevision::from_hash(blake3::hash(&bytes)))
}

pub const PROFILE_DATA_PREFIX: &str = "#r2modman\n";

pub(super) fn export_zip(profile: &Profile, writer: impl Write + Seek, game: Game) -> Result<()> {
    let manifest = build_manifest(profile, game);
    let config = collect_config_files(&profile.path, game.mod_loader.mod_config_dirs())?;

    write_archive(&manifest, &config, writer)
}

pub(super) fn build_manifest(profile: &Profile, game: Game) -> ProfileManifest {
    let mods = profile
        .thunderstore_mods()
        .map(|(ts_mod, enabled)| {
            let ident = ts_mod.ident.without_version();
            let version = ts_mod
                .ident
                .version()
                .parse::<semver::Version>()
                .expect("thunderstore version was not a semver")
                .into();

            R2Mod {
                ident,
                version,
                enabled,
                source: ts_mod.id.backend,
            }
        })
        .collect();

    ProfileManifest {
        name: profile.name.clone(),
        game: Some(game.slug.to_string()),
        mods,
        ignored_version_updates: profile.ignored_version_updates.iter().copied().collect(),
        ignored_package_updates: profile.ignored_package_updates.iter().copied().collect(),
        sync: None,
    }
}

pub(super) fn collect_config_files(
    root: &Path,
    config_dirs: &[&str],
) -> Result<BTreeMap<ConfigPath, Vec<u8>>> {
    let mut files = BTreeMap::new();

    for file in find_config(root, config_dirs) {
        let path = file
            .components()
            .map(|component| {
                component
                    .as_os_str()
                    .to_str()
                    .ok_or_eyre("config path is not valid UTF-8")
            })
            .collect::<Result<Vec<_>>>()?
            .join("/");

        let path = ConfigPath::try_from(path)
            .with_context(|| format!("invalid config path: {}", file.display()))?;

        let bytes = fs::read(root.join(&file))
            .with_context(|| format!("failed to read config file: {}", file.display()))?;

        ensure!(
            files.insert(path, bytes).is_none(),
            "duplicate config file: {}",
            file.display()
        );
    }

    Ok(files)
}

pub(super) fn write_archive(
    manifest: &ProfileManifest,
    config: &BTreeMap<ConfigPath, Vec<u8>>,
    writer: impl Write + Seek,
) -> Result<()> {
    let mut zip = ZipWriter::new(writer);

    zip.start_file("export.r2x", SimpleFileOptions::default())
        .context("failed to create manifest entry")?;
    serde_yaml::to_writer(&mut zip, manifest).context("failed to write profile manifest")?;

    for (path, bytes) in config {
        zip.start_file(path.as_str(), SimpleFileOptions::default())
            .with_context(|| format!("failed to create archive entry: {path}"))?;
        zip.write_all(bytes)
            .with_context(|| format!("failed to write archive entry: {path}"))?;
    }

    zip.finish().context("failed to finish archive")?;

    Ok(())
}

#[derive(Serialize)]
pub struct ExportCode {
    code: Uuid,
    backend: Backend,
}

async fn export_code(app: &AppHandle) -> Result<ExportCode> {
    let (backend, base64) = {
        let mut manager = app.lock_manager();

        let game = manager.active_game().game;
        let profile = manager.active_profile_mut();

        let mut data = Cursor::new(Vec::new());
        export_zip(profile, &mut data, game)?;

        let mut base64 = String::from(PROFILE_DATA_PREFIX);
        base64.push_str(&BASE64_STANDARD.encode(data.get_ref()));

        let backend = if profile.has_hexium_exclusive_mods(&app.lock_thunderstore()) {
            Backend::Hexium
        } else {
            Backend::Thunderstore
        };

        (backend, base64)
    };

    let len = base64.len();

    info!(len, "exporting profile code");

    let response = app
        .http()
        .post(backend.profile_export())
        .header("Content-Type", "application/octet-stream")
        .body(base64)
        .send()
        .await?;

    let response = match response.status() {
        status if status.is_success() => response.json::<LegacyProfileCreateResponse>().await?,
        StatusCode::PAYLOAD_TOO_LARGE => {
            bail!(
                "profile config is too large to export: {}, please reduce the size by removing heavy and/or unneeded config files",
                humansize::format_size(len, humansize::BINARY)
            );
        }
        _ => bail!("upload failed with status: {}", response.status()),
    };

    Ok(ExportCode {
        code: response.key,
        backend,
    })
}

fn write_config<P, I, W>(files: I, source: &Path, zip: &mut ZipWriter<W>) -> Result<()>
where
    P: AsRef<Path>,
    I: Iterator<Item = P>,
    W: Write + Seek,
{
    for file in files {
        let path = file.as_ref().to_string_lossy().replace('\\', "/");
        zip.start_file(path, SimpleFileOptions::default())?;

        let mut reader = File::open(source.join(file))?;

        io::copy(&mut reader, zip)?;
    }

    Ok(())
}

pub(super) fn find_config<'a>(
    root: &'a Path,
    config_dirs: &'a [&str],
) -> impl Iterator<Item = PathBuf> + 'a {
    static INCLUDE_SET: LazyLock<GlobSet> = LazyLock::new(|| {
        GlobSetBuilder::new()
            .add(Glob::new("*.{cfg,txt,json,yml,yaml,ini}").unwrap())
            .build()
            .unwrap()
    });

    static EXCLUDE_SET: LazyLock<GlobSet> = LazyLock::new(|| {
        GlobSetBuilder::new()
            .add(Glob::new("{dotnet,_state,snapshots,MelonLoader}/*").unwrap())
            .add(Glob::new("GDWeave/{GDWeave.log,core/*,mods/*}").unwrap())
            .add(Glob::new("mods.yml").unwrap())
            .add(
                GlobBuilder::new("BepInEx/plugins/*/manifest.json")
                    .literal_separator(true)
                    .build()
                    .unwrap(),
            )
            .build()
            .unwrap()
    });

    list_files(root).filter(move |path| {
        (config_dirs.iter().any(|dir| path.starts_with(dir)) || INCLUDE_SET.is_match(path))
            && !EXCLUDE_SET.is_match(path)
    })
}

pub(super) fn list_files(root: &Path) -> impl Iterator<Item = PathBuf> + '_ {
    WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(move |entry| {
            entry
                .into_path()
                .strip_prefix(root)
                .expect("path should be child of root")
                .to_path_buf()
        })
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;
    use crate::profile::sync::archive::{self, SyncArchiveFormat};

    fn r2_mod(
        name: &str,
        major: u64,
        minor: u64,
        patch: u64,
        enabled: bool,
        source: Backend,
    ) -> R2Mod {
        R2Mod {
            ident: PackageIdent::from(("Author", name)),
            version: semver::Version::new(major, minor, patch).into(),
            enabled,
            source,
        }
    }

    fn base_manifest() -> ProfileManifest {
        ProfileManifest {
            name: "Base".to_owned(),
            mods: vec![
                r2_mod("ModA", 1, 2, 3, true, Backend::Thunderstore),
                r2_mod("ModB", 4, 5, 6, false, Backend::Hexium),
            ],
            game: Some("risk-of-rain-2".to_owned()),
            ignored_version_updates: vec![Uuid::from_u128(0x01), Uuid::from_u128(0x02)],
            ignored_package_updates: vec![Uuid::from_u128(0x03), Uuid::from_u128(0x04)],
            sync: None,
        }
    }

    #[test]
    fn revision_ignores_mod_and_uuid_order() {
        let base = base_manifest();
        let mut reordered = base.clone();
        reordered.mods.reverse();
        reordered.ignored_version_updates.reverse();
        reordered.ignored_package_updates.reverse();

        assert_eq!(
            manifest_revision(&base).unwrap(),
            manifest_revision(&reordered).unwrap()
        );
    }

    #[test]
    fn revision_changes_per_field() {
        let base = base_manifest();
        let base_revision = manifest_revision(&base).unwrap();

        let mut changed = base.clone();
        changed.game = Some("among-us".to_owned());
        assert_ne!(manifest_revision(&changed).unwrap(), base_revision);

        let mut changed = base.clone();
        changed.mods[0].ident = PackageIdent::from(("Author", "OtherMod"));
        assert_ne!(manifest_revision(&changed).unwrap(), base_revision);

        let mut changed = base.clone();
        changed.mods[0].version.major += 1;
        assert_ne!(manifest_revision(&changed).unwrap(), base_revision);

        let mut changed = base.clone();
        changed.mods[0].version.minor += 1;
        assert_ne!(manifest_revision(&changed).unwrap(), base_revision);

        let mut changed = base.clone();
        changed.mods[0].version.patch += 1;
        assert_ne!(manifest_revision(&changed).unwrap(), base_revision);

        let mut changed = base.clone();
        changed.mods[0].enabled = false;
        assert_ne!(manifest_revision(&changed).unwrap(), base_revision);

        let mut changed = base.clone();
        changed.mods[0].source = Backend::Hexium;
        assert_ne!(manifest_revision(&changed).unwrap(), base_revision);

        let mut changed = base.clone();
        changed.ignored_version_updates[0] = Uuid::from_u128(0xff);
        assert_ne!(manifest_revision(&changed).unwrap(), base_revision);

        let mut changed = base.clone();
        changed.ignored_package_updates[0] = Uuid::from_u128(0xee);
        assert_ne!(manifest_revision(&changed).unwrap(), base_revision);
    }

    #[test]
    fn revision_ignores_name_and_sync() {
        let base = base_manifest();
        let base_revision = manifest_revision(&base).unwrap();

        let mut renamed = base.clone();
        renamed.name = "Other".to_owned();
        assert_eq!(manifest_revision(&renamed).unwrap(), base_revision);

        let mut synced = base.clone();
        synced.sync = Some(SyncManifest {
            version: 1,
            mods_revision: base_revision.clone(),
            config: BTreeMap::new(),
        });
        assert_eq!(manifest_revision(&synced).unwrap(), base_revision);
    }

    #[test]
    fn config_path_validation() {
        let valid = ConfigPath::try_from("BepInEx/config/example.cfg").unwrap();
        assert_eq!(valid.as_path(), Path::new("BepInEx/config/example.cfg"));

        for invalid in [
            "",
            "BepInEx\\config\\example.cfg",
            "/absolute",
            "C:/absolute",
            ".",
            "./a",
            "a/../b",
            "../a",
            "export.r2x",
            "Export.r2x",
            "_state/x",
            "_STATE/x",
            "a<b",
            "a>b",
            "a|b",
            "a?b",
            "a*b",
            "a\u{7}b",
            "a\"b",
            "a//b.cfg",
            "a//",
            "//a.cfg",
            "a/b/",
            "dir./x.cfg",
            "dir /x.cfg",
            "con.cfg",
            "CON",
            "aux/x.cfg",
            "com1/x.cfg",
            "lpt9/x.cfg",
        ] {
            assert!(ConfigPath::try_from(invalid).is_err(), "{invalid}");
        }
    }

    #[test]
    fn hash_newtypes_validate_and_roundtrip() {
        let hash = blake3::hash(b"test");
        let hex = hash.to_hex().to_string();

        let content_hash = ContentHash::try_from(hex.clone()).unwrap();
        assert_eq!(content_hash.as_str(), hex);
        assert_eq!(ContentHash::from_hash(hash), content_hash);

        let revision = ModRevision::try_from(hex.clone()).unwrap();
        assert_eq!(revision.as_str(), hex);
        assert_eq!(ModRevision::from_hash(hash), revision);

        for invalid in [
            "a".repeat(63),
            "a".repeat(65),
            "A".repeat(64),
            "g".repeat(64),
        ] {
            assert!(ContentHash::try_from(invalid.clone()).is_err(), "{invalid}");
            assert!(ModRevision::try_from(invalid).is_err());
        }

        let serialized = serde_json::to_string(&content_hash).unwrap();
        assert_eq!(serialized, format!("\"{hex}\""));
        assert_eq!(
            serde_json::from_str::<ContentHash>(&serialized).unwrap(),
            content_hash
        );
        assert!(serde_json::from_str::<ContentHash>("\"ABC\"").is_err());
    }

    #[test]
    fn manifest_sync_yaml_roundtrip() {
        let mut manifest = base_manifest();
        manifest.sync = Some(SyncManifest {
            version: 1,
            mods_revision: manifest_revision(&manifest).unwrap(),
            config: BTreeMap::from([(
                ConfigPath::try_from("BepInEx/config/example.cfg").unwrap(),
                SyncFileEntry {
                    hash: ContentHash::from_hash(blake3::hash(b"cfg")),
                },
            )]),
        });

        let yaml = serde_yaml::to_string(&manifest).unwrap();
        assert!(yaml.contains("galeSync"));

        let parsed: ProfileManifest = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.sync, manifest.sync);

        let yaml = serde_yaml::to_string(&base_manifest()).unwrap();
        assert!(!yaml.contains("galeSync"));

        let parsed: ProfileManifest = serde_yaml::from_str(&yaml).unwrap();
        assert!(parsed.sync.is_none());
    }

    #[test]
    fn write_archive_legacy_validates() {
        let manifest = base_manifest();
        let path = ConfigPath::try_from("BepInEx/config/example.cfg").unwrap();
        let config = BTreeMap::from([(path.clone(), b"data".to_vec())]);

        let mut cursor = Cursor::new(Vec::new());
        write_archive(&manifest, &config, &mut cursor).unwrap();

        let validated = archive::validate(cursor.get_ref()).unwrap();
        assert!(matches!(validated.format, SyncArchiveFormat::Legacy));
        assert_eq!(validated.config[&path].bytes, b"data");
    }

    #[test]
    fn write_archive_selective_validates() {
        let mut manifest = base_manifest();
        let path = ConfigPath::try_from("BepInEx/config/example.cfg").unwrap();
        let bytes = b"custom".to_vec();

        manifest.sync = Some(SyncManifest {
            version: 1,
            mods_revision: manifest_revision(&manifest).unwrap(),
            config: BTreeMap::from([(
                path.clone(),
                SyncFileEntry {
                    hash: ContentHash::from_hash(blake3::hash(&bytes)),
                },
            )]),
        });

        let config = BTreeMap::from([(path.clone(), bytes.clone())]);

        let mut cursor = Cursor::new(Vec::new());
        write_archive(&manifest, &config, &mut cursor).unwrap();

        let validated = archive::validate(cursor.get_ref()).unwrap();
        assert!(matches!(validated.format, SyncArchiveFormat::Selective(_)));
        assert_eq!(validated.config[&path].bytes, bytes);
    }

    #[test]
    fn collect_config_files_reads_nested_configs() {
        let root = tempdir().unwrap();
        let nested = root.path().join("BepInEx/config/sub");
        fs::create_dir_all(&nested).unwrap();
        fs::write(root.path().join("BepInEx/config/example.cfg"), "settings").unwrap();
        fs::write(nested.join("nested.cfg"), "nested").unwrap();
        fs::write(root.path().join("binary.bin"), b"\x00\x01").unwrap();

        let files = collect_config_files(root.path(), &["BepInEx/config"]).unwrap();

        assert_eq!(files.len(), 2);
        assert_eq!(
            files[&ConfigPath::try_from("BepInEx/config/example.cfg").unwrap()],
            b"settings"
        );
        assert_eq!(
            files[&ConfigPath::try_from("BepInEx/config/sub/nested.cfg").unwrap()],
            b"nested"
        );
    }

    #[test]
    fn r2_version_preserves_prerelease_and_build() {
        let beta = R2Version::from(semver::Version::parse("2.0.14-beta.2").unwrap());
        assert_eq!(beta.to_string(), "2.0.14-beta.2");

        let json = serde_json::to_string(&beta).unwrap();
        assert!(json.contains("\"pre\":\"beta.2\""));
        assert!(!json.contains("\"build\""));
        assert_eq!(serde_json::from_str::<R2Version>(&json).unwrap(), beta);

        let build = R2Version::from(semver::Version::parse("2.0.14+build.7").unwrap());
        let json = serde_json::to_string(&build).unwrap();
        assert!(json.contains("\"build\":\"build.7\""));
        assert_eq!(serde_json::from_str::<R2Version>(&json).unwrap(), build);

        let legacy: R2Version =
            serde_json::from_str(r#"{"major":2,"minor":0,"patch":14}"#).unwrap();
        assert_eq!(legacy.to_string(), "2.0.14");
        assert!(legacy.pre.is_empty() && legacy.build.is_empty());

        let mmp = R2Version::from(semver::Version::new(2, 0, 14));
        assert_eq!(
            serde_json::to_string(&mmp).unwrap(),
            r#"{"major":2,"minor":0,"patch":14}"#
        );

        assert!(
            serde_json::from_str::<R2Version>(
                r#"{"major":1,"minor":0,"patch":0,"pre":"not valid!"}"#
            )
            .is_err()
        );
    }

    #[test]
    fn version_ident_preserves_prerelease() {
        let r2_mod = R2Mod {
            ident: PackageIdent::from(("ArgusMagnus", "ServersideQoL")),
            version: semver::Version::parse("2.0.14-beta.2").unwrap().into(),
            enabled: true,
            source: Backend::Thunderstore,
        };

        let ident = r2_mod.version_ident();
        assert_eq!(ident.to_string(), "ArgusMagnus-ServersideQoL-2.0.14-beta.2");
        assert_eq!(ident.version(), "2.0.14-beta.2");
    }

    #[test]
    fn revision_differentiates_prerelease_and_build() {
        let mut base = base_manifest();
        base.mods[0].version = semver::Version::parse("2.0.14-beta.2").unwrap().into();
        let base_revision = manifest_revision(&base).unwrap();

        for version in ["2.0.14", "2.0.14-beta.1", "2.0.14-beta.3", "2.0.14+build.1"] {
            let mut changed = base.clone();
            changed.mods[0].version = semver::Version::parse(version).unwrap().into();
            assert_ne!(
                manifest_revision(&changed).unwrap(),
                base_revision,
                "{version}"
            );
        }

        let mut mmp = base.clone();
        mmp.mods[0].version = semver::Version::new(2, 0, 14).into();
        let mut parsed = base.clone();
        parsed.mods[0].version = semver::Version::parse("2.0.14").unwrap().into();
        assert_eq!(
            manifest_revision(&mmp).unwrap(),
            manifest_revision(&parsed).unwrap()
        );
    }

    #[test]
    fn archive_round_trip_preserves_prerelease_version() {
        let mut manifest = base_manifest();
        manifest.mods[0] = R2Mod {
            ident: PackageIdent::from(("ArgusMagnus", "ServersideQoL")),
            version: semver::Version::parse("2.0.14-beta.2").unwrap().into(),
            enabled: true,
            source: Backend::Thunderstore,
        };

        manifest.sync = Some(SyncManifest {
            version: 1,
            mods_revision: manifest_revision(&manifest).unwrap(),
            config: BTreeMap::new(),
        });

        let mut cursor = Cursor::new(Vec::new());
        write_archive(&manifest, &BTreeMap::new(), &mut cursor).unwrap();

        let validated = archive::validate(cursor.get_ref()).unwrap();
        assert!(matches!(validated.format, SyncArchiveFormat::Selective(_)));

        let parsed_mod = &validated.manifest.mods[0];
        assert_eq!(
            parsed_mod.version_ident().to_string(),
            "ArgusMagnus-ServersideQoL-2.0.14-beta.2"
        );

        let mods_revision = validated.manifest.sync.unwrap().mods_revision;
        assert_eq!(mods_revision, manifest_revision(&manifest).unwrap());

        let mut mmp = manifest.clone();
        mmp.mods[0].version = semver::Version::new(2, 0, 14).into();
        assert_ne!(mods_revision, manifest_revision(&mmp).unwrap());
    }
}
