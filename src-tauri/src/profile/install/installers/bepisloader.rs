use std::{
    borrow::Cow,
    io::Read,
    path::{Component, Path, PathBuf},
};

use eyre::{OptionExt, Result};
use tracing::warn;

use super::{
    subdir::{Subdir, SubdirInstaller, SubdirMode},
    PackageInstaller, PackageZip,
};
use crate::{
    profile::{Profile, ProfileMod},
    util,
};

/// BepisLoader-specific installer that handles Renderer mod routing.
/// 
/// Unlike standard BepInEx, BepisLoader mods with Renderer components need special
/// path handling to avoid double-nesting when the target path overlaps with the source.
pub struct BepisLoaderInstaller<'a> {
    subdirs: &'a [Subdir<'a>],
    extra_subdirs: &'a [Subdir<'a>],
}

impl<'a> BepisLoaderInstaller<'a> {
    pub fn new(subdirs: &'a [Subdir<'a>]) -> Self {
        Self {
            subdirs,
            extra_subdirs: &[],
        }
    }

    pub fn with_extras(mut self, extra_subdirs: &'a [Subdir<'a>]) -> Self {
        self.extra_subdirs = extra_subdirs;
        self
    }

    fn map_file<'p>(
        &self,
        relative_path: &'p Path,
        package_name: &str,
    ) -> Result<Vec<Cow<'p, Path>>> {
        let mut prev = PathBuf::new();
        let mut components = relative_path.components();

        let subdir = loop {
            match components.next() {
                Some(Component::Normal(name)) => {
                    prev.push(name);
                    if let Some(name) = name.to_str() {
                        if let Some(subdir) = self
                            .subdirs
                            .iter()
                            .chain(self.extra_subdirs.iter())
                            .find(|subdir| {
                                util::cmp_ignore_case(subdir.name, name).is_eq()
                                    || subdir.extension.is_some_and(|ext| {
                                        ext.split(',').any(|ext| name.ends_with(ext))
                                    })
                            })
                        {
                            break subdir;
                        }
                    }
                }
                Some(Component::ParentDir) => {
                    prev.pop();
                }
                Some(Component::RootDir | Component::Prefix(_) | Component::CurDir) => continue,
                None => return Ok(Vec::new()),
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

        let mut target = PathBuf::from(subdir.target);

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
                prev_components.next_back();
                target.push(prev_components);
            }

            // Strip redundant path segments when target already contains them.
            // Example: Renderer mod with source "Renderer/BepInEx/plugins/AudioBridgeRenderer"
            // and target "Renderer/BepInEx/plugins" should map to just "AudioBridgeRenderer"
            let mut components_to_add = components.clone();
            if flatten && subdir.target.starts_with(subdir.name) {
                if let Some(target_suffix) = subdir.target.strip_prefix(subdir.name) {
                    if !target_suffix.is_empty() {
                        let target_suffix =
                            target_suffix.strip_prefix("/").unwrap_or(target_suffix);

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

        Ok(vec![Cow::Owned(target)])
    }
}

impl PackageInstaller for BepisLoaderInstaller<'_> {
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
                std::fs::create_dir_all(target_path.parent().unwrap())?;
                std::fs::write(&target_path, &content)?;
            }
        }

        Ok(())
    }

    fn install(&mut self, src: &Path, package_name: &str, profile: &Profile) -> Result<()> {
        let mut installer = SubdirInstaller::new(self.subdirs)
            .with_default(1)
            .with_extras(self.extra_subdirs);

        installer.install(src, package_name, profile)
    }

    fn toggle(&mut self, enabled: bool, profile_mod: &ProfileMod, profile: &Profile) -> Result<()> {
        let mut installer = SubdirInstaller::new(self.subdirs)
            .with_default(1)
            .with_extras(self.extra_subdirs);

        installer.toggle(enabled, profile_mod, profile)
    }

    fn uninstall(&mut self, profile_mod: &ProfileMod, profile: &Profile) -> Result<()> {
        let mut installer = SubdirInstaller::new(self.subdirs)
            .with_default(1)
            .with_extras(self.extra_subdirs);

        installer.uninstall(profile_mod, profile)
    }

    fn mod_dir(&self, package_name: &str, profile: &Profile) -> Option<PathBuf> {
        self.subdirs.get(1).map(|subdir| {
            let mut path = profile.path.join(subdir.target);
            path.push(package_name);
            path
        })
    }
}
