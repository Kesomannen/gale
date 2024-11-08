use std::{
    borrow::Cow,
    ffi::OsStr,
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::profile::{
    install::{
        self,
        fs::{ConflictResolution, FileInstallMethod},
    },
    Profile, ProfileMod,
};

use super::{ModArchive, PackageInstaller};

pub struct SubdirInstaller<'a> {
    subdirs: &'a [Subdir<'a>],
    default_subdir: Option<&'a Subdir<'a>>,
    ignored_files: &'a [&'a str],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subdir<'a> {
    /// The name which "triggers" the subdir. Must be a single path component.
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

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
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
    pub fn new(
        subdirs: &'a [Subdir<'a>],
        default: Option<&'a Subdir<'a>>,
        ignored_files: &'a [&'a str],
    ) -> Self {
        Self {
            subdirs,
            default_subdir: default,
            ignored_files,
        }
    }

    fn match_subdir(&self, name: &str) -> Option<&Subdir> {
        self.subdirs.iter().find(|subdir| {
            subdir.name == name
                || subdir
                    .extension
                    .is_some_and(|ext| ext.split(',').any(|ext| name.ends_with(ext)))
        })
    }

    fn map_file<'p>(
        &self,
        relative_path: &'p Path,
        package_name: &str,
    ) -> Result<Option<Cow<'p, Path>>> {
        use std::path::Component;

        if let Some(str) = relative_path.to_str() {
            if self.ignored_files.iter().any(|file| *file == str) {
                return Ok(None);
            }
        }

        // first, flatten the path until a subdir appears
        // if the path contains no subdirs, default to /plugins

        let mut prev: Vec<&OsStr> = Vec::new();
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
                // default to plugins when the whole path is exhausted
                None => match self.default_subdir {
                    Some(subdir) => break subdir,
                    None => return Ok(None),
                },
            }
        };

        // components now contains either:
        // - the remaining path if a subdir was found, or
        // - None if the file is top level and thus defaulted to plugins

        // prev is the canonical path leading up to a subdir,
        // or the whole path if we defaulted

        // e.g. profile/BepInEx/plugins
        let mut target = PathBuf::from(subdir.target);

        if matches!(
            subdir.mode,
            SubdirMode::SeparateFlatten | SubdirMode::Separate
        ) {
            // e.g. profile/BepInEx/plugins/Kesomannen-CoolMod
            target.push(package_name);
        }

        if components.clone().next().is_none() {
            // since we advanced components to the end, prev.pop() will give the
            // last component, i.e. the file name
            let file_name = prev.pop().context("malformed mod archive file")?;

            // e.g. profile/BepInEx/plugins/Kesomannen-CoolMod/CoolMod.dll
            target.push(file_name);
        } else {
            if subdir.mode != SubdirMode::SeparateFlatten {
                // don't include the subdir component itself
                let len = prev.len() - 1;
                let prev = prev.iter().take(len).collect::<PathBuf>();

                target.push(prev);
            }

            // add the remainder of the path after the subdir
            // e.g. profile/BepInEx/plugins/Kesomannen-CoolMod/assets/cool_icon.png
            target.push(components);
        }

        Ok(Some(Cow::Owned(target)))
    }

    fn scan_mod<F>(&self, profile_mod: &ProfileMod, profile: &Profile, mut scan: F) -> Result<()>
    where
        F: FnMut(&Path) -> Result<()>,
    {
        let ident = profile_mod.ident();
        let full_name = ident.full_name();

        for subdir in self.subdirs {
            match subdir.mode {
                SubdirMode::Separate | SubdirMode::SeparateFlatten => {
                    let mut path = profile.path.to_path_buf();
                    path.push(subdir.target);
                    path.push(full_name);

                    scan(&path)?;
                }
                SubdirMode::Track => todo!(),
                SubdirMode::None => (),
            };
        }

        Ok(())
    }
}

struct StateFile {
    package_state: File,
    profile_state: File,
}

impl StateFile {
    pub fn create(package_name: &str, profile: &Profile) -> Result<Self> {
        let path = profile.path.join("_state");
        fs::create_dir_all(&path).context("failed to create state directory");

        let package_state = open(path.clone(), package_name)?;
        let profile_state = open(path, "profile")?;

        return Ok(Self {
            package_state,
            profile_state,
        });

        fn open(mut path: PathBuf, name: &str) -> io::Result<File> {
            path.push(name);
            path.set_extension("txt");

            File::options().append(true).create(true).open(path)
        }
    }
}

impl<'a> PackageInstaller for SubdirInstaller<'a> {
    fn extract(&mut self, archive: ModArchive, package_name: &str, dest: PathBuf) -> Result<()> {
        install::fs::extract(archive, dest, |relative_path| {
            self.map_file(relative_path, package_name)
        })
    }

    fn install(
        &mut self,
        src: &Path,
        package_name: &str,
        overwrite: bool,
        profile: &Profile,
    ) -> Result<()> {
        let mut state_file: Option<StateFile> = None;

        install::fs::install(
            src,
            profile,
            |relative_path| {
                let subdir = self
                    .subdirs
                    .iter()
                    .find(|subdir| relative_path.starts_with(subdir.target))
                    .expect("file should be in a subdir");

                if subdir.mode == SubdirMode::Track {
                    let file = match &mut state_file {
                        Some(file) => file,
                        None => {
                            let file = StateFile::create(package_name, profile)
                                .context("failed to open state file")?;

                            state_file = Some(file);
                            state_file.as_mut().unwrap()
                        }
                    };

                    file.write_all(relative_path.as_os_str().as_encoded_bytes())
                        .unwrap();
                    file.write_all(b"\n").unwrap();
                }

                Ok(FileInstallMethod::Link)
            },
            |_| ConflictResolution::overwrite(overwrite),
        )
    }

    fn toggle(&mut self, enabled: bool, profile_mod: &ProfileMod, profile: &Profile) -> Result<()> {
        self.scan_mod(profile_mod, profile, |path| {
            install::fs::toggle_any(path, enabled)
        })
    }

    fn uninstall(&mut self, profile_mod: &ProfileMod, profile: &Profile) -> Result<()> {
        self.scan_mod(profile_mod, profile, |path| {
            install::fs::uninstall_any(path)?;
            Ok(())
        })?;

        Ok(())
    }
}
