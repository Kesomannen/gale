use std::{
    borrow::Cow,
    collections::HashMap,
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use eyre::{Context, OptionExt, Result};
use serde::{Deserialize, Serialize};
use tracing::warn;

use super::{PackageInstaller, PackageZip};
use crate::{
    profile::{
        install::{
            self,
            fs::{ConflictResolution, FileInstallMethod},
        },
        Profile, ProfileMod,
    },
    util::{self, fs::JsonStyle},
};

pub struct SubdirInstaller<'a> {
    subdirs: &'a [Subdir<'a>],
    default_subdir: Option<usize>,
    extra_subdirs: &'a [Subdir<'a>],
    ignored_files: &'a [&'a str],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subdir<'a> {
    /// The name which "triggers" the subdir. Must be a single path component.
    pub name: &'a str,
    /// The target paths of the subdir, relative to the profile dir.
    /// Files will be copied to all specified targets.
    ///
    /// Use forward slashes to separate path components.
    pub targets: Vec<&'a str>,
    #[serde(default)]
    pub mode: SubdirMode,
    /// Whether files in this subdir can be/are expected to be mutated.
    ///
    /// When this is `false` (as default), files are installed using hard links
    /// instead of copying, which saves disk space and copy time.
    #[serde(default)]
    pub mutable: bool,
    /// File extension(s) that automatically route to this subdir.
    /// Multiple extensions are separated by a comma.
    #[serde(default)]
    pub extension: Option<&'a str>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SubdirMode {
    /// Separate mods into `author-name` dirs.
    Separate,
    /// Same as [`SubdirMode::Separate`], but also flatten any dirs that
    /// come before the subdir.
    #[default]
    SeparateFlatten,
    /// Track which files are installed by which mod.
    Track,
    /// Don't track or separate mods. This prevents disabling
    /// or uninstallation of files in the subdir.
    None,
}

impl<'a> Subdir<'a> {
    pub fn new(name: &'a str, targets: Vec<&'a str>, mode: SubdirMode) -> Self {
        Self {
            name,
            targets,
            mode,
            mutable: false,
            extension: None,
        }
    }

    pub fn separated(name: &'a str, target: &'a str) -> Self {
        Self::new(name, vec![target], SubdirMode::Separate)
    }

    pub fn flat_separated(name: &'a str, target: &'a str) -> Self {
        Self::new(name, vec![target], SubdirMode::SeparateFlatten)
    }

    pub fn tracked(name: &'a str, target: &'a str) -> Self {
        Self::new(name, vec![target], SubdirMode::Track)
    }

    pub fn untracked(name: &'a str, target: &'a str) -> Self {
        Self::new(name, vec![target], SubdirMode::None)
    }

    pub fn mutable(mut self) -> Self {
        self.mutable = true;
        self
    }

    pub fn extension(mut self, ext: &'a str) -> Self {
        self.extension = Some(ext);
        self
    }
}

impl<'a> SubdirInstaller<'a> {
    pub fn new(subdirs: &'a [Subdir<'a>]) -> Self {
        const DEFAULT_EXTRA: &[Subdir] = &[];
        const DEFAULT_IGNORED: &[&str] = &[];

        Self {
            subdirs,
            default_subdir: None,
            extra_subdirs: DEFAULT_EXTRA,
            ignored_files: DEFAULT_IGNORED,
        }
    }

    pub fn with_default(mut self, index: usize) -> Self {
        self.default_subdir = Some(index);
        self
    }

    pub fn with_extras(mut self, subdirs: &'a [Subdir<'a>]) -> Self {
        self.extra_subdirs = subdirs;
        self
    }

    pub fn with_ignored_files(mut self, files: &'a [&'a str]) -> Self {
        self.ignored_files = files;
        self
    }

    fn subdirs(&'_ self) -> impl Iterator<Item = &'_ Subdir<'_>> {
        self.extra_subdirs.iter().chain(self.subdirs.iter())
    }

    fn match_subdir(&'_ self, name: &str) -> Option<&'_ Subdir<'_>> {
        self.subdirs().find(|subdir| {
            util::cmp_ignore_case(subdir.name, name).is_eq()
                || subdir
                    .extension
                    .is_some_and(|ext| ext.split(',').any(|ext| name.ends_with(ext)))
        })
    }

    fn map_file<'p>(
        &self,
        relative_path: &'p Path,
        package_name: &str,
    ) -> Result<Vec<Cow<'p, Path>>> {
        use std::path::Component;

        if let Some(str) = relative_path.to_str() {
            if self.ignored_files.contains(&str) {
                return Ok(Vec::new());
            }
        }

        let mut prev = PathBuf::new();
        let mut components = relative_path.components();

        let subdir = loop {
            match components.next() {
                Some(Component::Normal(name)) => {
                    prev.push(name);
                    if let Some(name) = name.to_str() {
                        if let Some(subdir) = self.match_subdir(name) {
                            break subdir;
                        }
                    }
                }
                // remove the previous parent
                Some(Component::ParentDir) => {
                    prev.pop();
                }
                // we don't care/don't expect any of these
                Some(Component::RootDir | Component::Prefix(_) | Component::CurDir) => continue,
                // default when the whole path is exhausted
                None => match self.default_subdir {
                    Some(index) => break &self.subdirs[index],
                    None => return Ok(Vec::new()),
                },
            }
        };

        let separate = matches!(
            subdir.mode,
            SubdirMode::Separate | SubdirMode::SeparateFlatten
        );
        let flatten = matches!(
            subdir.mode,
            SubdirMode::SeparateFlatten | SubdirMode::Track | SubdirMode::None
        );
        let is_top_level = components.clone().next().is_none();

        let mut results = Vec::new();
        for target_path in &subdir.targets {
            let mut target = PathBuf::from(target_path);

            if separate {
                target.push(package_name);
            }

            if is_top_level {
                if flatten {
                    let file_name = prev.file_name().ok_or_eyre("malformed archive")?;
                    target.push(file_name);
                } else {
                    target.push(&prev);
                }
            } else {
                if !flatten {
                    let mut prev_components = prev.components();
                    // remove the subdir component itself
                    prev_components.next_back();
                    target.push(prev_components);
                }

                // When the target path contains the trigger name followed by more path components,
                // we should strip the overlapping part from the source path
                let mut components_to_add = components.clone();
                if flatten && target_path.starts_with(subdir.name) {
                    // Extract the suffix after the trigger name from the target
                    if let Some(target_suffix) = target_path.strip_prefix(subdir.name) {
                        if !target_suffix.is_empty() {
                            // Remove leading slash if present
                            let target_suffix =
                                target_suffix.strip_prefix("/").unwrap_or(target_suffix);

                            // Check if the remaining components start with the same pattern as target_suffix
                            let suffix_parts: Vec<&str> = target_suffix.split('/').collect();
                            let mut temp_components = components_to_add.clone();
                            let mut matches = true;

                            for expected_part in &suffix_parts {
                                if let Some(Component::Normal(actual)) = temp_components.next() {
                                    if actual.to_str() != Some(expected_part) {
                                        matches = false;
                                        break;
                                    }
                                } else {
                                    matches = false;
                                    break;
                                }
                            }

                            // If the components match the target suffix, skip them
                            if matches {
                                for _ in &suffix_parts {
                                    components_to_add.next();
                                }
                            }
                        }
                    }
                }

                target.push(components_to_add);
            }

            results.push(Cow::Owned(target));
        }

        Ok(results)
    }

    fn scan_mod<F>(&self, profile_mod: &ProfileMod, profile: &Profile, mut scan: F) -> Result<bool>
    where
        F: FnMut(&Path) -> Result<()>,
    {
        let mut scanned_tracked_files = false;
        let package_name = profile_mod.full_name();

        for subdir in self.subdirs() {
            match subdir.mode {
                SubdirMode::Separate | SubdirMode::SeparateFlatten => {
                    for target in &subdir.targets {
                        let mut path = profile.path.to_path_buf();
                        path.push(target);
                        path.push(&*package_name);

                        scan(&path)?;
                    }
                }
                SubdirMode::Track if !scanned_tracked_files => {
                    scanned_tracked_files = true;

                    let mut state = PackageStateHandle::new(&package_name, profile);
                    for file in state.files() {
                        scan(&profile.path.join(file))?;
                    }
                }
                SubdirMode::Track => (),
                SubdirMode::None => (),
            };
        }

        Ok(scanned_tracked_files)
    }
}

fn state_file_path(name: &str, profile: &Profile) -> PathBuf {
    let mut path = profile.path.to_path_buf();

    path.push("_state");
    path.push(name);
    path.set_extension("json");

    path
}

struct PackageStateHandle {
    path: PathBuf,
    state: PackageState,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct PackageState {
    files: Vec<PathBuf>,
}

impl PackageStateHandle {
    fn new(package_name: &str, profile: &Profile) -> Self {
        let path = state_file_path(package_name, profile);
        let state = util::fs::read_json(&path).unwrap_or_default();
        Self { path, state }
    }

    fn from_profile_mod(profile_mod: &ProfileMod, profile: &Profile) -> Self {
        Self::new(&profile_mod.full_name(), profile)
    }

    fn files(&mut self) -> &mut Vec<PathBuf> {
        &mut self.state.files
    }

    fn commit(&self) -> Result<()> {
        fs::create_dir_all(self.path.parent().unwrap())?;
        util::fs::write_json(&self.path, &self.state, JsonStyle::Pretty)
    }

    fn delete(self) -> Result<()> {
        fs::remove_file(self.path)?;
        Ok(())
    }
}

struct ProfileStateHandle {
    path: PathBuf,
    state: ProfileState,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct ProfileState {
    file_map: HashMap<PathBuf, String>,
}

impl ProfileStateHandle {
    fn new(profile: &Profile) -> Self {
        let path = state_file_path("profile", profile);
        let state = util::fs::read_json(&path).unwrap_or_default();
        Self { path, state }
    }

    fn file_map(&mut self) -> &mut HashMap<PathBuf, String> {
        &mut self.state.file_map
    }

    fn commit(&self) -> Result<()> {
        fs::create_dir_all(self.path.parent().unwrap())?;
        util::fs::write_json(&self.path, &self.state, JsonStyle::Pretty)
    }
}

impl PackageInstaller for SubdirInstaller<'_> {
    fn extract(
        &mut self,
        mut archive: PackageZip,
        package_name: &str,
        dest: PathBuf,
    ) -> Result<()> {
        for i in 0..archive.len() {
            let mut source_file = archive.by_index(i)?;

            if source_file.is_dir() {
                continue;
            }

            let name = source_file.name().to_owned();
            let relative_path: Cow<'_, Path> = if cfg!(unix) && name.contains('\\') {
                PathBuf::from(name.replace('\\', "/")).into()
            } else {
                Path::new(&name).into()
            };

            if !util::fs::is_enclosed(&relative_path) {
                warn!(
                    "file {} escapes the archive root, skipping",
                    relative_path.display()
                );
                continue;
            }

            let target_paths = self.map_file(&relative_path, package_name)?;

            if target_paths.is_empty() {
                continue;
            }

            let mut content = Vec::new();
            source_file.read_to_end(&mut content)?;

            for relative_target in target_paths {
                let target_path = dest.join(&*relative_target);
                fs::create_dir_all(target_path.parent().unwrap())?;
                fs::write(&target_path, &content)?;
            }
        }

        Ok(())
    }

    fn install(&mut self, src: &Path, package_name: &str, profile: &Profile) -> Result<()> {
        let mut state: Option<PackageStateHandle> = None;
        let mut profile_state: Option<ProfileStateHandle> = None;

        install::fs::install(src, profile, |relative_path, exists| {
            let subdir = self
                .subdirs()
                .find(|subdir| {
                    subdir
                        .targets
                        .iter()
                        .any(|target| relative_path.starts_with(target))
                })
                .expect("file should be in a subdir");

            let method = if subdir.mutable {
                FileInstallMethod::Copy
            } else {
                FileInstallMethod::Link
            };

            let conflict = match subdir.mode {
                // this should never happen
                SubdirMode::Separate | SubdirMode::SeparateFlatten => ConflictResolution::Skip,
                SubdirMode::None => ConflictResolution::Overwrite,
                SubdirMode::Track => {
                    state
                        .get_or_insert_with(|| PackageStateHandle::new(package_name, profile))
                        .files()
                        .push(relative_path.to_owned());

                    let profile_state =
                        profile_state.get_or_insert_with(|| ProfileStateHandle::new(profile));

                    if exists {
                        if let Some(owner) = profile_state.file_map().get(relative_path) {
                            let mut package = PackageStateHandle::new(owner, profile);
                            package.files().retain(|file| file != relative_path);
                            package.commit()?;
                        }
                    }

                    profile_state
                        .file_map()
                        .insert(relative_path.to_owned(), package_name.to_owned());

                    ConflictResolution::Overwrite
                }
            };

            Ok((method, conflict))
        })?;

        if let Some(state) = state {
            state.commit().context("failed to write state")?;
        }

        if let Some(state) = profile_state {
            state.commit().context("failed to write profile state")?;
        }

        Ok(())
    }

    fn toggle(&mut self, enabled: bool, profile_mod: &ProfileMod, profile: &Profile) -> Result<()> {
        self.scan_mod(profile_mod, profile, |path| {
            install::fs::toggle_any(path, enabled)
        })?;

        Ok(())
    }

    fn uninstall(&mut self, profile_mod: &ProfileMod, profile: &Profile) -> Result<()> {
        let has_tracked_files = self.scan_mod(profile_mod, profile, |path| {
            install::fs::uninstall_any(path)
        })?;

        if has_tracked_files {
            if let Err(err) = PackageStateHandle::from_profile_mod(profile_mod, profile).delete() {
                warn!(
                    "failed to delete state file for {}: {:#}",
                    profile_mod.full_name(),
                    err
                );
            }

            let mut profile_state = ProfileStateHandle::new(profile);
            profile_state
                .file_map()
                .retain(|_, package| *package != profile_mod.full_name());
            profile_state
                .commit()
                .context("failed to write profile state")?;
        }

        Ok(())
    }

    fn mod_dir(&self, package_name: &str, profile: &Profile) -> Option<PathBuf> {
        self.default_subdir.and_then(|index| {
            self.subdirs[index].targets.first().map(|target| {
                let mut path = profile.path.to_path_buf();
                path.push(target);
                path.push(package_name);
                path
            })
        })
    }
}
