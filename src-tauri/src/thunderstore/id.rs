use std::{
    cmp,
    fmt::{self, Debug, Display},
    hash::Hash,
    str::FromStr,
};

use serde::{Deserialize, Serialize};

#[derive(Eq, Clone, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub struct VersionIdent {
    repr: String,
    name_start: usize,
    version_start: usize,
}

impl VersionIdent {
    pub fn new(owner: &str, name: &str, version: &str) -> Self {
        let repr = format!("{}-{}-{}", owner, name, version);
        let name_start = owner.len() + 1;
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

    pub fn split(&self) -> (&str, &str, &str) {
        (self.owner(), self.name(), self.version())
    }

    pub fn path(&self) -> impl Display + '_ {
        VersionIdentPath(self)
    }

    pub fn into_string(self) -> String {
        self.repr
    }

    pub fn as_str(&self) -> &str {
        &self.repr
    }
}

impl PartialEq for VersionIdent {
    fn eq(&self, other: &Self) -> bool {
        self.repr == other.repr
    }
}

impl Ord for VersionIdent {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.repr.cmp(&other.repr)
    }
}

impl PartialOrd for VersionIdent {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for VersionIdent {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.repr.hash(state);
    }
}

impl AsRef<str> for VersionIdent {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<VersionIdent> for String {
    fn from(id: VersionIdent) -> Self {
        id.repr
    }
}

impl Display for VersionIdent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.repr)
    }
}

impl Debug for VersionIdent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VersionId({:?})", self.repr)
    }
}

#[derive(Debug)]
pub struct SyntaxError;

impl Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid package identifier syntax")
    }
}

impl TryFrom<String> for VersionIdent {
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

impl FromStr for VersionIdent {
    type Err = SyntaxError;

    fn from_str(s: &str) -> Result<Self, SyntaxError> {
        s.to_string().try_into()
    }
}

impl<T, U, V> From<(T, U, V)> for VersionIdent
where
    T: AsRef<str>,
    U: AsRef<str>,
    V: AsRef<str>,
{
    fn from((owner, name, version): (T, U, V)) -> Self {
        Self::new(owner.as_ref(), name.as_ref(), version.as_ref())
    }
}

impl<T, U> From<(T, U, u32, u32, u32)> for VersionIdent
where
    T: AsRef<str>,
    U: AsRef<str>,
{
    fn from((owner, name, major, minor, patch): (T, U, u32, u32, u32)) -> Self {
        Self::new(
            owner.as_ref(),
            name.as_ref(),
            &format!("{}.{}.{}", major, minor, patch),
        )
    }
}

struct VersionIdentPath<'a>(&'a VersionIdent);

impl<'a> Display for VersionIdentPath<'a> {
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
pub struct PackageIdent {
    repr: String,
    name_start: usize,
}

impl PackageIdent {
    pub fn new(owner: &str, name: &str) -> Self {
        let repr = format!("{}-{}", owner, name);
        let name_start = owner.len() + 1;
        Self { repr, name_start }
    }

    pub fn owner(&self) -> &str {
        &self.repr[..self.name_start - 1]
    }

    pub fn name(&self) -> &str {
        &self.repr[self.name_start..]
    }

    pub fn split(&self) -> (&str, &str) {
        (self.owner(), self.name())
    }

    pub fn path(&self) -> impl Display + '_ {
        PackageIdentPath(self)
    }

    pub fn into_string(self) -> String {
        self.repr
    }

    pub fn as_str(&self) -> &str {
        &self.repr
    }
}

impl PartialEq for PackageIdent {
    fn eq(&self, other: &Self) -> bool {
        self.repr == other.repr
    }
}

impl Ord for PackageIdent {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.repr.cmp(&other.repr)
    }
}

impl PartialOrd for PackageIdent {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for PackageIdent {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.repr.hash(state);
    }
}

impl AsRef<str> for PackageIdent {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<PackageIdent> for String {
    fn from(id: PackageIdent) -> Self {
        id.repr
    }
}

impl Display for PackageIdent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.repr)
    }
}

impl Debug for PackageIdent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PackageId({:?})", self.repr)
    }
}

impl TryFrom<String> for PackageIdent {
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

impl FromStr for PackageIdent {
    type Err = SyntaxError;

    fn from_str(s: &str) -> Result<Self, SyntaxError> {
        s.to_string().try_into()
    }
}

impl From<(&str, &str)> for PackageIdent {
    fn from((owner, name): (&str, &str)) -> Self {
        Self::new(owner, name)
    }
}

impl From<VersionIdent> for PackageIdent {
    fn from(id: VersionIdent) -> Self {
        let version_start = id.version_start;
        let name_start = id.name_start;

        let mut repr = id.into_string();
        repr.truncate(version_start - 1);
        repr.shrink_to_fit();

        Self { repr, name_start }
    }
}

struct PackageIdentPath<'a>(&'a PackageIdent);

impl<'a> Display for PackageIdentPath<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.0.owner(), self.0.name(),)
    }
}