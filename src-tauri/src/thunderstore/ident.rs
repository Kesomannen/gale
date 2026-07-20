use std::{
    cmp,
    fmt::{self, Debug, Display},
    hash::Hash,
    str::FromStr,
};

use internment::Intern;
use serde::{Deserialize, Serialize};

/// A unique identifier for a specific version of a package.
///
/// Often formatted as `owner-name-version`, also known as a dependency string.
#[derive(Eq, Clone, Serialize, Deserialize)]
#[serde(into = "Intern<String>", try_from = "Intern<String>")]
pub struct VersionIdent {
    repr: Intern<String>,
    name_start: u32,
    version_start: u32,
}

impl VersionIdent {
    /// Creates a new identifier with the given parts.
    ///
    /// This allocates a new string.
    pub fn new(owner: &str, name: &str, version: &str) -> Self {
        let repr = format!("{owner}-{name}-{version}").into();

        let name_start = owner.len() as u32 + 1;
        let version_start = name_start + name.len() as u32 + 1;

        Self {
            repr,
            name_start,
            version_start,
        }
    }

    pub fn owner(&self) -> &str {
        &self.repr[..self.name_start as usize - 1]
    }

    pub fn name(&self) -> &str {
        &self.repr[self.name_start as usize..self.version_start as usize - 1]
    }

    pub fn full_name(&self) -> &str {
        &self.repr[..self.version_start as usize - 1]
    }

    pub fn version(&self) -> &str {
        &self.repr[self.version_start as usize..]
    }

    pub fn split(&self) -> (&str, &str, &str) {
        (self.owner(), self.name(), self.version())
    }

    pub fn without_version(&self) -> PackageIdent {
        let repr = self.repr[..self.version_start as usize - 1].to_string();

        PackageIdent {
            repr,
            name_start: self.name_start,
        }
    }

    #[inline]
    pub fn into_string(self) -> Intern<String> {
        self.repr
    }

    #[inline]
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

impl From<VersionIdent> for Intern<String> {
    fn from(id: VersionIdent) -> Self {
        id.into_string()
    }
}

impl Display for VersionIdent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.repr)
    }
}

impl Debug for VersionIdent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("VersionIdent").field(&self.repr).finish()
    }
}

#[derive(Debug)]
pub struct ParseError;

impl Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid identifier")
    }
}

impl TryFrom<Intern<String>> for VersionIdent {
    type Error = ParseError;

    /// Parses a string into a `VersionIdent`.
    ///
    /// This does not allocate or copy memory.
    fn try_from(value: Intern<String>) -> Result<Self, ParseError> {
        // we parse backwards here because Thunderstore team names may include a hyphen

        let mut indices = value.match_indices('-').map(|(i, _)| i);

        // semver versions may contain a hyphen
        let mut version_start = next_segment(&mut indices)?;
        if semver::Version::parse(&value[version_start..]).is_err() {
            version_start = next_segment(&mut indices)?;

            if semver::Version::parse(&value[version_start..]).is_err() {
                return Err(ParseError);
            }
        }

        let name_start = next_segment(&mut indices)?;

        return Ok(Self {
            repr: value,
            name_start: name_start as u32,
            version_start: version_start as u32,
        });

        fn next_segment<I: Iterator<Item = usize> + DoubleEndedIterator>(
            indicies: &mut I,
        ) -> Result<usize, ParseError> {
            indicies.next_back().ok_or(ParseError).map(|i| i + 1)
        }
    }
}

impl FromStr for VersionIdent {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, ParseError> {
        Intern::new(s.to_string()).try_into()
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
            &format!("{major}.{minor}.{patch}"),
        )
    }
}

/// A unique identifier for a package, often formatted as `owner-name`.
#[derive(Eq, Clone, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub struct PackageIdent {
    repr: String,
    name_start: u32,
}

impl PackageIdent {
    /// Creates a new identifier with the given parts.
    ///
    /// This allocates a new string and copies the slices into it.
    pub fn new(owner: &str, name: &str) -> Self {
        let repr = format!("{owner}-{name}");
        let name_start = owner.len() as u32 + 1;
        Self { repr, name_start }
    }

    pub fn owner(&self) -> &str {
        &self.repr[..self.name_start as usize - 1]
    }

    pub fn name(&self) -> &str {
        &self.repr[self.name_start as usize..]
    }

    pub fn split(&self) -> (&str, &str) {
        (self.owner(), self.name())
    }

    pub fn path(&self) -> impl Display + '_ {
        PackageIdentPath(self)
    }

    pub fn with_version(&self, version: impl Display) -> VersionIdent {
        let repr = format!("{}-{}", self.repr, version).into();

        VersionIdent {
            repr,
            name_start: self.name_start,
            version_start: self.repr.len() as u32 + 1,
        }
    }

    #[inline]
    pub fn into_string(self) -> String {
        self.repr
    }

    #[inline]
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
        id.into_string()
    }
}

impl Display for PackageIdent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.repr)
    }
}

impl Debug for PackageIdent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("PackageId").field(&self.repr).finish()
    }
}

impl TryFrom<String> for PackageIdent {
    type Error = ParseError;

    /// Parses a string into a `VersionIdent`.
    ///
    /// This does not allocate or copy memory.
    fn try_from(value: String) -> Result<Self, ParseError> {
        let mut indices = value.match_indices('-').map(|(i, _)| i);

        let name_start = indices.next_back().ok_or(ParseError)? as u32 + 1;

        Ok(Self {
            repr: value,
            name_start,
        })
    }
}

impl FromStr for PackageIdent {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, ParseError> {
        s.to_string().try_into()
    }
}

impl From<(&str, &str)> for PackageIdent {
    fn from((owner, name): (&str, &str)) -> Self {
        Self::new(owner, name)
    }
}

struct PackageIdentPath<'a>(&'a PackageIdent);

impl Display for PackageIdentPath<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.0.owner(), self.0.name(),)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_package_ident() {
        let ident = PackageIdent::from_str("owner-name").unwrap();
        assert_eq!(ident.owner(), "owner");
        assert_eq!(ident.name(), "name");

        let ident = PackageIdent::from_str("owner-name-with-hyphen-package").unwrap();
        assert_eq!(ident.owner(), "owner-name-with-hyphen");
        assert_eq!(ident.name(), "package");
    }

    #[test]
    fn parse_version_ident() {
        let ident = VersionIdent::from_str("owner-name-1.0.0").unwrap();
        assert_eq!(ident.owner(), "owner");
        assert_eq!(ident.name(), "name");
        assert_eq!(ident.version(), "1.0.0");

        let ident = VersionIdent::from_str("owner-name-with-hyphen-package-1.0.0").unwrap();
        assert_eq!(ident.owner(), "owner-name-with-hyphen");
        assert_eq!(ident.name(), "package");
        assert_eq!(ident.version(), "1.0.0");

        let ident = VersionIdent::from_str("owner-package-1.0.0-beta.1").unwrap();
        assert_eq!(ident.owner(), "owner");
        assert_eq!(ident.name(), "package");
        assert_eq!(ident.version(), "1.0.0-beta.1");

        let ident = VersionIdent::from_str("owner-with-hyphen-package-1.0.0-beta.1").unwrap();
        assert_eq!(ident.owner(), "owner-with-hyphen");
        assert_eq!(ident.name(), "package");
        assert_eq!(ident.version(), "1.0.0-beta.1");
    }
}
