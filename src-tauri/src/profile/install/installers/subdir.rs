use std::{
    borrow::Cow,
    collections::HashMap,
    fs,
    path::{Components, Path, PathBuf},
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

/// A configurable mod installer based on r2modman's install rules:
/// https://github.com/ebkr/r2modmanPlus/wiki/Structuring-your-Thunderstore-package
pub struct SubdirInstaller<'a> {
    subdirs: &'a [Subdir<'a>],
    extra_subdirs: &'a [Subdir<'a>],
    /// Index of the default subdir to place files into.
    /// If set to `None`, files will be ignored by default.
    default_subdir: Option<usize>,
    /// File paths that the installer should always ignore.
    ignored_files: &'a [&'a str],
}

/// A directory inside the profile where files will be placed into.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subdir<'a> {
    /// The string which "triggers" the subdir. Must be a single path component.
    /// Case-independent, that is `plugins` would be matched by `Plugins`.
    pub name: &'a str,
    /// The target path of the subdir, relative to the profile dir.
    ///
    /// Use forward slashes to separate path components.
    pub target: &'a str,
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
    /// come before the subdir name.
    #[default]
    SeparateFlatten,
    /// Don't separate mods, but instead track which files are installed by which mod.
    Track,
    /// Don't track or separate mods. This prevents disabling
    /// or uninstallation of files in the subdir.
    None,
}

impl<'a> Subdir<'a> {
    pub const fn new(name: &'a str, target: &'a str, mode: SubdirMode) -> Self {
        Self {
            name,
            target,
            mode,
            mutable: false,
            extension: None,
        }
    }

    pub const fn separated(name: &'a str, target: &'a str) -> Self {
        Self::new(name, target, SubdirMode::Separate)
    }

    pub const fn flat_separated(name: &'a str, target: &'a str) -> Self {
        Self::new(name, target, SubdirMode::SeparateFlatten)
    }

    pub const fn tracked(name: &'a str, target: &'a str) -> Self {
        Self::new(name, target, SubdirMode::Track)
    }

    pub const fn untracked(name: &'a str, target: &'a str) -> Self {
        Self::new(name, target, SubdirMode::None)
    }

    pub const fn mutable(mut self) -> Self {
        self.mutable = true;
        self
    }

    pub const fn extension(mut self, ext: &'a str) -> Self {
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

    /// Determines if a path component matches any subdir names/extensions.
    fn match_subdir(&'_ self, name: &str) -> Option<&'_ Subdir<'_>> {
        self.subdirs().find(|subdir| {
            util::cmp_ignore_case(subdir.name, name).is_eq()
                || subdir
                    .extension
                    .is_some_and(|ext| ext.split(',').any(|ext| name.ends_with(ext)))
        })
    }

    /// Checks if we need to skip overlapping path segments.
    fn handle_overlap<'p>(
        &self,
        mut components: Components<'p>,
        subdir: &Subdir,
        flatten: bool,
    ) -> Components<'p> {
        // Only process if we're flattening and target starts with the trigger
        if !flatten || !subdir.target.starts_with(subdir.name) {
            return components;
        }

        // Get the suffix after the trigger name
        let Some(suffix) = subdir.target.strip_prefix(subdir.name) else {
            return components;
        };

        let suffix = suffix.strip_prefix('/').unwrap_or(suffix);
        if suffix.is_empty() {
            return components;
        }

        // Check if the source components match the target suffix pattern
        let suffix_path = Path::new(suffix);
        let suffix_components: Vec<_> = suffix_path.components().collect();

        // Test if components start with the same pattern
        let mut test_components = components.clone();
        let matches = suffix_components
            .iter()
            .all(|expected| test_components.next() == Some(*expected));

        // Skip the matching prefix if found
        if matches {
            for _ in &suffix_components {
                components.next();
            }
        }

        components
    }

    /// Map a file in the mod archive (at relative_path) to the target path relative to the profile's root.
    /// Ok(None) means a file should be ignored
    fn map_file<'p>(
        &self,
        relative_path: &'p Path,
        package_name: &str,
    ) -> Result<Option<Cow<'p, Path>>> {
        use std::path::Component;

        if let Some(str) = relative_path.to_str() {
            if self.ignored_files.contains(&str) {
                return Ok(None);
            }
        }

        // find a subdir in the file path, ex.
        // MyFolder/plugins/MyMod.dll
        //          ^-----^
        // `prev` holds the path up to the subdir component (or the whole path if it doesn't contain one)
        // `components` holds the remaining components of the path

        let mut prev = PathBuf::new();
        let mut components = relative_path.components();

        let subdir = loop {
            match components.next() {
                Some(Component::Normal(name)) => {
                    prev.push(name);
                    if let Some(name) = name.to_str() {
                        if let Some(subdir) = self.match_subdir(name) {
                            break subdir; // found a subdir
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
                    None => return Ok(None),
                },
            }
        };

        // whether to separete mod files by source mod
        let separate = matches!(
            subdir.mode,
            SubdirMode::Separate | SubdirMode::SeparateFlatten
        );
        // whether to flatten the target path (i.e. placing the files directly in the subdir)
        let flatten = matches!(
            subdir.mode,
            SubdirMode::SeparateFlatten | SubdirMode::Track | SubdirMode::None
        );
        // this means the path didn't contain any subdirs
        let defaulted = components.clone().next().is_none();

        // ex. BepInEx/plugins
        let mut target = PathBuf::from(subdir.target);

        if separate {
            // ex. BepInEx/plugins/Author-ModName
            target.push(package_name);
        }

        if defaulted {
            if flatten {
                // place the file directly into the default directory
                let file_name = prev.file_name().ok_or_eyre("malformed archive")?;

                // ex. icon.png -> BepInEx/plugins/icon.png
                target.push(file_name);
            } else {
                // place the file along with its parent directories into the default directory
                // ex. MyIcons/icon.png -> BepInEx/plugins/MyIcons/icon.png
                target.push(&prev);
            }
        } else {
            if !flatten {
                let mut prev = prev.components();
                // remove the subdir component itself
                prev.next_back();

                target.push(prev);
            }

            // ex. relative_path: MyFolder/plugins/MyOtherFolder/Plugin.dll
            //    (with flatten): BepInEx/plugins/MyOtherFolder/Plugin.dll
            // (without flatten): BepInEx/plugins/MyFolder/MyOtherFolder/Plugin.dll
            let components_to_add = self.handle_overlap(components, subdir, flatten);
            target.push(components_to_add);
        }

        Ok(Some(Cow::Owned(target)))
    }

    /// Executes a function for each of a mod's installed files within a profile.
    /// Returns whether any tracked files where scanned.
    fn scan_mod<F>(&self, profile_mod: &ProfileMod, profile: &Profile, mut scan: F) -> Result<bool>
    where
        F: FnMut(&Path) -> Result<()>,
    {
        // file tracking doesn't differentiate between subdirs, so we need to make
        // sure to only scan the tracked files once, even if we have multiple subdirs
        // with [`SubdirMode::Track`]
        let mut scanned_tracked_files = false;

        let package_name = profile_mod.full_name();

        for subdir in self.subdirs() {
            match subdir.mode {
                SubdirMode::Separate | SubdirMode::SeparateFlatten => {
                    let mut path = profile.path.to_path_buf();
                    // ex. BepInEx/plugins
                    path.push(subdir.target);
                    // ex. BepInEx/plugins/Author-CoolMod
                    path.push(&*package_name);

                    scan(&path)?;
                }
                SubdirMode::Track if !scanned_tracked_files => {
                    scanned_tracked_files = true;

                    let mut state = PackageStateHandle::new(&package_name, profile);
                    for file in state.files() {
                        scan(&profile.path.join(file))?;
                    }
                }
                SubdirMode::Track => (), // we have already scanned tracked files
                SubdirMode::None => (), // we can't know which files in the subdir belong to this mod; ignore
            };
        }

        Ok(scanned_tracked_files)
    }
}

/// The state files are used by subdirs with [`SubdirMode::Track`] to know which files belong to which mods.
/// This system is similar to r2modman's, with the exception that we use json instead of yaml.
///
/// Each mod has its own json file in the `_state` folder that lists the mod's own files.
/// The profile also has a main json file that links each file to its corresponding mod.
///
/// When we want to write a file to the state, we thus have to write to both the mod-specific file as well as
/// the profile "registry".

fn state_file_path(name: &str, profile: &Profile) -> PathBuf {
    let mut path = profile.path.to_path_buf();

    path.push("_state");
    path.push(name);
    path.set_extension("json");

    path
}

/// A handle to an opened state file for one mod/package.
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

/// A handle to an opened profile state file.
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
    fn extract(&mut self, archive: PackageZip, package_name: &str, dest: PathBuf) -> Result<()> {
        install::fs::extract(archive, dest, |relative_path| {
            self.map_file(relative_path, package_name)
        })
    }

    fn install(&mut self, src: &Path, package_name: &str, profile: &Profile) -> Result<()> {
        let mut state: Option<PackageStateHandle> = None;
        let mut profile_state: Option<ProfileStateHandle> = None;

        install::fs::install(src, profile, |relative_path, exists| {
            let subdir = self
                .subdirs()
                .find(|subdir| relative_path.starts_with(subdir.target))
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
        self.default_subdir.map(|index| {
            let mut path = profile.path.to_path_buf();

            path.push(self.subdirs[index].target);
            path.push(package_name);

            path
        })
    }
}
