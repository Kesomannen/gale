use std::{borrow::Borrow, fmt, ops::Deref};

use eyre::{Result, ensure};
use serde::{Deserialize, Deserializer, Serialize, de::Error};

/// A path on the remote server, always separated by `/`.
///
/// Remote servers follow POSIX path semantics even when Gale itself runs on
/// Windows, so these must never be joined with [`std::path::Path`]. Convert to
/// the transport library's expected type only at the transport boundary.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RemotePathBuf {
    value: String,
}

/// Borrowed counterpart of [`RemotePathBuf`].
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct RemotePath {
    value: str,
}

/// A path relative to the profile/deployment root, always separated by `/`.
///
/// Deployment paths identify files in both the local profile and the remote
/// manifest, so they must be strictly relative with no platform-specific
/// separators, `.`/`..` segments or empty segments.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct DeployPathBuf {
    value: String,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct DeployPath {
    value: str,
}

fn check_remote_chars(value: &str) -> Result<()> {
    ensure!(
        !value.contains('\\'),
        "remote path contains a native separator: {value}"
    );
    ensure!(!value.contains('\0'), "remote path contains a NUL byte");
    Ok(())
}

impl RemotePathBuf {
    /// Parses and normalizes a remote path.
    ///
    /// Repeated separators are collapsed and any trailing separator is removed
    /// (except for the root itself). `.` and `..` segments are rejected: the
    /// remote server resolves them, so allowing them would make paths that
    /// escape the configured server directory indistinguishable.
    pub fn new(value: impl AsRef<str>) -> Result<Self> {
        let value = value.as_ref();
        check_remote_chars(value)?;

        let absolute = value.starts_with('/');
        let mut normalized = String::with_capacity(value.len());
        if absolute {
            normalized.push('/');
        }

        for segment in value.split('/') {
            if segment.is_empty() {
                continue;
            }
            ensure!(
                segment != "." && segment != "..",
                "remote path contains a relative segment: {value}"
            );

            if normalized.len() > usize::from(absolute) {
                normalized.push('/');
            }
            normalized.push_str(segment);
        }

        if normalized.is_empty() && absolute {
            normalized.push('/');
        }
        ensure!(
            !normalized.is_empty(),
            "remote path cannot be empty or relative to nothing"
        );

        Ok(Self { value: normalized })
    }

    pub fn root() -> Self {
        Self {
            value: "/".to_owned(),
        }
    }

    pub fn as_path(&self) -> &RemotePath {
        RemotePath::new_unchecked(&self.value)
    }
}

impl RemotePath {
    fn new_unchecked(value: &str) -> &Self {
        // SAFETY: RemotePath is a transparent wrapper over str.
        unsafe { &*(value as *const str as *const Self) }
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// The final path segment, if this isn't the root.
    pub fn file_name(&self) -> Option<&str> {
        self.value
            .rsplit('/')
            .next()
            .filter(|name| !name.is_empty() && self.value.len() > 1)
    }

    pub fn is_root(&self) -> bool {
        self.value == *"/"
    }

    pub fn join(&self, relative: &DeployPath) -> RemotePathBuf {
        let mut value = self.value.to_owned();
        if !value.ends_with('/') {
            value.push('/');
        }
        value.push_str(&relative.value);

        RemotePathBuf { value }
    }

    /// Appends a suffix to the path as-is (used for `.tmp`, `.gale-upload`
    /// and `.gale-backup` markers).
    pub fn with_suffix(&self, suffix: &str) -> RemotePathBuf {
        RemotePathBuf {
            value: format!("{}{suffix}", &self.value),
        }
    }

    pub fn to_owned_buf(&self) -> RemotePathBuf {
        RemotePathBuf {
            value: self.value.to_owned(),
        }
    }
}

impl Deref for RemotePathBuf {
    type Target = RemotePath;

    fn deref(&self) -> &Self::Target {
        self.as_path()
    }
}

impl Borrow<RemotePath> for RemotePathBuf {
    fn borrow(&self) -> &RemotePath {
        self.as_path()
    }
}

impl fmt::Display for RemotePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.value)
    }
}

impl fmt::Display for RemotePathBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.value)
    }
}

impl Serialize for RemotePathBuf {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.value)
    }
}

impl<'de> Deserialize<'de> for RemotePathBuf {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(D::Error::custom)
    }
}

impl DeployPathBuf {
    pub fn new(value: impl AsRef<str>) -> Result<Self> {
        let value = value.as_ref();
        ensure!(Self::is_valid(value), "unsafe relative path: {value}");

        Ok(Self {
            value: value.to_owned(),
        })
    }

    pub fn is_valid(value: &str) -> bool {
        !value.is_empty()
            && !value.starts_with('/')
            && !value.contains('\\')
            && !value.contains('\0')
            && value
                .split('/')
                .all(|segment| !segment.is_empty() && segment != "." && segment != "..")
    }

    pub fn as_path(&self) -> &DeployPath {
        DeployPath::new_unchecked(&self.value)
    }
}

impl DeployPath {
    fn new_unchecked(value: &str) -> &Self {
        // SAFETY: DeployPath is a transparent wrapper over str.
        unsafe { &*(value as *const str as *const Self) }
    }

    /// Returns the already-validated path unchanged.
    ///
    /// Useful where a `&str` is statically known to be a valid deploy path.
    pub fn from_static(value: &'static str) -> &'static Self {
        assert!(
            DeployPathBuf::is_valid(value),
            "invalid static deploy path: {value}"
        );
        Self::new_unchecked(value)
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// The final path segment.
    pub fn file_name(&self) -> &str {
        self.value
            .rsplit('/')
            .next()
            .expect("deploy path is never empty")
    }

    /// The parent directory, unless this path is a top-level name.
    pub fn parent(&self) -> Option<DeployPathBuf> {
        let (parent, _) = self.value.rsplit_once('/')?;
        DeployPathBuf::new(parent).ok()
    }

    /// Whether this path is a strict ancestor of `other`, at any depth.
    pub fn is_ancestor_of(&self, other: &DeployPath) -> bool {
        other.value.starts_with(&self.value) && other.value[self.value.len()..].starts_with('/')
    }

    /// Whether this path equals or lies within `directory`.
    pub fn is_within(&self, directory: &DeployPath) -> bool {
        self == directory || directory.is_ancestor_of(self)
    }

    /// Strips the leading `prefix/` from the path.
    pub fn strip_prefix(&self, prefix: &str) -> Option<DeployPathBuf> {
        let stripped = self.value.strip_prefix(prefix)?.strip_prefix('/')?;
        DeployPathBuf::new(stripped).ok()
    }

    /// Joins a single segment onto the path, validating the result.
    pub fn join(&self, name: &str) -> Result<DeployPathBuf> {
        DeployPathBuf::new(format!("{}/{name}", &self.value))
    }

    /// Appends a suffix to the path as-is (used for `.old` and `.tmp`
    /// markers).
    pub fn with_suffix(&self, suffix: &str) -> Result<DeployPathBuf> {
        DeployPathBuf::new(format!("{}{suffix}", &self.value))
    }

    /// Path segments.
    pub fn segments(&self) -> impl Iterator<Item = &str> {
        self.value.split('/')
    }

    /// Path segments plus the full path itself — i.e. every ancestor of the
    /// path and the path itself, shortest first.
    ///
    /// Used to let directory walks descend into a mirrored tree.
    pub fn self_and_ancestors(&self) -> impl Iterator<Item = DeployPathBuf> {
        self.segments().scan(String::new(), |current, segment| {
            if !current.is_empty() {
                current.push('/');
            }
            current.push_str(segment);
            DeployPathBuf::new(current.clone()).ok()
        })
    }

    pub fn to_owned_buf(&self) -> DeployPathBuf {
        DeployPathBuf {
            value: self.value.to_owned(),
        }
    }
}

impl Deref for DeployPathBuf {
    type Target = DeployPath;

    fn deref(&self) -> &Self::Target {
        self.as_path()
    }
}

impl Borrow<DeployPath> for DeployPathBuf {
    fn borrow(&self) -> &DeployPath {
        self.as_path()
    }
}

impl AsRef<DeployPath> for DeployPathBuf {
    fn as_ref(&self) -> &DeployPath {
        self.as_path()
    }
}

impl fmt::Display for DeployPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.value)
    }
}

impl fmt::Display for DeployPathBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.value)
    }
}

impl<'de> Deserialize<'de> for DeployPathBuf {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(D::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::{DeployPathBuf, RemotePathBuf};

    #[test]
    fn remote_paths_normalize_and_validate() {
        assert_eq!(RemotePathBuf::new("/").unwrap().as_str(), "/");
        assert_eq!(
            RemotePathBuf::new("/home//user/server/").unwrap().as_str(),
            "/home/user/server"
        );
        assert_eq!(RemotePathBuf::new("server").unwrap().as_str(), "server");
        assert!(RemotePathBuf::new("").is_err());
        assert!(RemotePathBuf::new("/etc/../passwd").is_err());
        assert!(RemotePathBuf::new("/./hidden").is_err());
        assert!(RemotePathBuf::new("C:\\server").is_err());
        assert!(RemotePathBuf::new("/with\0nul").is_err());
    }

    #[test]
    fn remote_joining_keeps_posix_semantics() {
        let base = RemotePathBuf::new("/srv/valheim/").unwrap();
        let file = DeployPathBuf::new("BepInEx/plugins/Mod.dll").unwrap();

        assert_eq!(
            base.join(&file).as_str(),
            "/srv/valheim/BepInEx/plugins/Mod.dll"
        );
        assert_eq!(
            RemotePathBuf::root().join(&file).as_str(),
            "/BepInEx/plugins/Mod.dll"
        );
    }

    #[test]
    fn deploy_paths_validate() {
        assert!(DeployPathBuf::new("BepInEx/plugins/Example.dll").is_ok());
        assert!(DeployPathBuf::new("doorstop_config.ini").is_ok());
        assert!(DeployPathBuf::new("a/./b").is_err());
        assert!(DeployPathBuf::new("a//b").is_err());
        assert!(DeployPathBuf::new("/abs").is_err());
        assert!(DeployPathBuf::new("../escape").is_err());
        assert!(DeployPathBuf::new("a\\b").is_err());
        assert!(DeployPathBuf::new("").is_err());
    }

    #[test]
    fn deploy_path_ancestry() {
        let dir = DeployPathBuf::new("BepInEx/plugins").unwrap();
        let file = DeployPathBuf::new("BepInEx/plugins/Mod.dll").unwrap();
        let other = DeployPathBuf::new("BepInEx/core/x.dll").unwrap();

        assert!(file.is_within(&dir));
        assert!(dir.is_ancestor_of(&file));
        assert!(!dir.is_ancestor_of(&other));
        assert_eq!(file.parent().unwrap().as_str(), "BepInEx/plugins");
        assert_eq!(file.file_name(), "Mod.dll");
        assert_eq!(
            file.self_and_ancestors()
                .map(|path| path.to_string())
                .collect::<Vec<_>>(),
            ["BepInEx", "BepInEx/plugins", "BepInEx/plugins/Mod.dll"]
        );
    }

    #[test]
    fn deploy_paths_serialize_as_strings() {
        let path = DeployPathBuf::new("BepInEx/plugins/Mod.dll").unwrap();
        let json = serde_json::to_string(&path).unwrap();

        assert_eq!(json, "\"BepInEx/plugins/Mod.dll\"");
        assert_eq!(serde_json::from_str::<DeployPathBuf>(&json).unwrap(), path);
        assert!(serde_json::from_str::<DeployPathBuf>("\"../bad\"").is_err());
    }
}
