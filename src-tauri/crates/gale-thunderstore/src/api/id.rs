use std::{
    cmp,
    fmt::{self, Debug, Display},
    hash::Hash,
    str::FromStr,
};

use serde::{Deserialize, Serialize};

#[derive(Eq, Clone, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub struct VersionId {
    repr: String,
    name_start: usize,
    version_start: usize,
}

impl VersionId {
    pub fn new(namespace: &str, name: &str, version: &str) -> Self {
        let repr = format!("{}-{}-{}", namespace, name, version);
        let name_start = namespace.len() + 1;
        let version_start = name_start + name.len() + 1;
        Self {
            repr,
            name_start,
            version_start,
        }
    }

    pub fn owner(&self) -> &str {
        &self.repr[..self.name_start - 1]
    }

    pub fn name(&self) -> &str {
        &self.repr[self.name_start..self.version_start - 1]
    }

    pub fn full_name(&self) -> &str {
        &self.repr[..self.version_start - 1]
    }

    pub fn version(&self) -> &str {
        &self.repr[self.version_start..]
    }

    pub fn version_split(&self) -> (u32, u32, u32) {
        let version = self.version();
        let mut parts = version.split('.');

        return (next(&mut parts), next(&mut parts), next(&mut parts));

        fn next<'a>(parts: &mut impl Iterator<Item = &'a str>) -> u32 {
            parts.next().unwrap_or("0").parse().unwrap_or(0)
        }
    }

    pub fn path(&self) -> impl Display + '_ {
        VersionIdPath(self)
    }

    pub fn into_string(self) -> String {
        self.repr
    }

    pub fn as_str(&self) -> &str {
        &self.repr
    }
}

impl PartialEq for VersionId {
    fn eq(&self, other: &Self) -> bool {
        self.repr == other.repr
    }
}

impl Ord for VersionId {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.repr.cmp(&other.repr)
    }
}

impl PartialOrd for VersionId {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for VersionId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.repr.hash(state);
    }
}

impl AsRef<str> for VersionId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<VersionId> for String {
    fn from(id: VersionId) -> Self {
        id.repr
    }
}

impl Display for VersionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.repr)
    }
}

impl Debug for VersionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VersionId({:?})", self.repr)
    }
}

#[derive(Debug)]
pub struct SyntaxError;

impl Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid package id syntax")
    }
}

impl TryFrom<String> for VersionId {
    type Error = SyntaxError;

    fn try_from(value: String) -> Result<Self, SyntaxError> {
        let mut indices = value.match_indices('-').map(|(i, _)| i);

        let name_start = indices.next().ok_or(SyntaxError)? + 1;
        let version_start = indices.next().ok_or(SyntaxError)? + 1;

        Ok(Self {
            repr: value,
            name_start,
            version_start,
        })
    }
}

impl FromStr for VersionId {
    type Err = SyntaxError;

    fn from_str(s: &str) -> Result<Self, SyntaxError> {
        s.to_string().try_into()
    }
}

impl<T, U, V> From<(T, U, V)> for VersionId
where
    T: AsRef<str>,
    U: AsRef<str>,
    V: AsRef<str>,
{
    fn from((namespace, name, version): (T, U, V)) -> Self {
        Self::new(namespace.as_ref(), name.as_ref(), version.as_ref())
    }
}

impl<T, U> From<(T, U, i64, i64, i64)> for VersionId
where
    T: AsRef<str>,
    U: AsRef<str>,
{
    fn from((namespace, name, major, minor, patch): (T, U, i64, i64, i64)) -> Self {
        Self::new(
            namespace.as_ref(),
            name.as_ref(),
            &format!("{}.{}.{}", major, minor, patch),
        )
    }
}

struct VersionIdPath<'a>(&'a VersionId);

impl<'a> Display for VersionIdPath<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}/{}/{}",
            self.0.owner(),
            self.0.name(),
            self.0.version()
        )
    }
}

#[derive(Eq, Clone, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub struct PackageId {
    repr: String,
    name_start: usize,
}

impl PackageId {
    pub fn new(namespace: &str, name: &str) -> Self {
        let repr = format!("{}-{}", namespace, name);
        let name_start = namespace.len() + 1;
        Self { repr, name_start }
    }

    pub fn namespace(&self) -> &str {
        &self.repr[..self.name_start - 1]
    }

    pub fn name(&self) -> &str {
        &self.repr[self.name_start..]
    }

    pub fn path(&self) -> impl Display + '_ {
        PackageIdPath(self)
    }

    pub fn into_string(self) -> String {
        self.repr
    }

    pub fn as_str(&self) -> &str {
        &self.repr
    }
}

impl PartialEq for PackageId {
    fn eq(&self, other: &Self) -> bool {
        self.repr == other.repr
    }
}

impl Ord for PackageId {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.repr.cmp(&other.repr)
    }
}

impl PartialOrd for PackageId {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for PackageId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.repr.hash(state);
    }
}

impl AsRef<str> for PackageId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<PackageId> for String {
    fn from(id: PackageId) -> Self {
        id.repr
    }
}

impl Display for PackageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.repr)
    }
}

impl Debug for PackageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PackageId({:?})", self.repr)
    }
}

impl TryFrom<String> for PackageId {
    type Error = SyntaxError;

    fn try_from(value: String) -> Result<Self, SyntaxError> {
        let mut indices = value.match_indices('-').map(|(i, _)| i);

        let name_start = indices.next().ok_or(SyntaxError)? + 1;

        Ok(Self {
            repr: value,
            name_start,
        })
    }
}

impl FromStr for PackageId {
    type Err = SyntaxError;

    fn from_str(s: &str) -> Result<Self, SyntaxError> {
        s.to_string().try_into()
    }
}

impl From<(&str, &str)> for PackageId {
    fn from((namespace, name): (&str, &str)) -> Self {
        Self::new(namespace, name)
    }
}

impl From<VersionId> for PackageId {
    fn from(id: VersionId) -> Self {
        let version_start = id.version_start;
        let name_start = id.name_start;

        let mut repr = id.into_string();
        repr.truncate(version_start - 1);
        repr.shrink_to_fit();

        Self { repr, name_start }
    }
}

struct PackageIdPath<'a>(&'a PackageId);

impl<'a> Display for PackageIdPath<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.0.namespace(), self.0.name(),)
    }
}
