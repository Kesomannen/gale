use eyre::Result;
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};

use super::{
    manifest,
    paths::{DeployPath, DeployPathBuf},
};
use crate::game::mod_loader::ModLoader;

/// Top-level names inside a profile directory that belong to Gale itself and
/// are never deployed.
const PROFILE_INTERNAL_NAMES: &[&str] = &["profile.json", "mods.yml", "snapshots", "_state"];

/// How a profile should be laid out on a dedicated server, derived from the
/// game's mod loader. This is the single place where mod-loader-specific
/// deployment policy lives; the rest of the deployment pipeline is generic.
pub struct DeploymentSpec {
    /// The top-level directory the mod loader installs into inside the
    /// profile, e.g. `BepInEx`. Managed hosts may expose this directory
    /// directly at the remote root instead of the profile root.
    pub mirror_root: DeployPathBuf,

    /// Directories whose remote contents are kept in sync with the profile.
    /// Remote files inside them that are not in the deployment manifest are
    /// removed. Everything outside them is upload-only: the host may keep its
    /// own files there, and Gale only ever touches paths it recorded in the
    /// manifest.
    pub mirror_dirs: Vec<DeployPathBuf>,

    /// Deploy paths that identify the mod loader's own payload in the
    /// profile. A manifest containing any of these means Gale deployed the
    /// loader itself on a previous run, so it keeps managing it.
    loader_owned_globs: GlobSet,

    /// Deploy paths whose remote presence marks the loader installation as
    /// managed by the host rather than by Gale.
    pub loader_markers: Vec<DeployPathBuf>,

    /// Where the deployment manifest lives, inside a mirror directory so
    /// restricted hosts can store it too.
    pub manifest_path: DeployPathBuf,

    /// Deploy paths that are never uploaded: generated caches, logs and
    /// package metadata that is only meaningful locally.
    exclude_globs: GlobSet,
}

impl DeploymentSpec {
    /// Builds the deployment spec for a mod loader, or fails if the loader
    /// has no dedicated-server deployment support.
    pub fn for_loader(mod_loader: &ModLoader<'static>) -> Result<Self> {
        let mirror_dirs = mod_loader
            .server_mirror_dirs()
            .ok_or_else(|| {
                eyre::eyre!(
                    "dedicated server deployment is not supported for {}",
                    mod_loader.as_str()
                )
            })?
            .iter()
            .map(DeployPathBuf::new)
            .collect::<Result<Vec<_>>>()?;

        let mirror_root = mirror_dirs
            .first()
            .and_then(|dir| dir.segments().next().map(str::to_owned))
            .map(DeployPathBuf::new)
            .ok_or_else(|| eyre::eyre!("mod loader has no mirror directories"))??;

        let config_dir = mod_loader
            .mod_config_dirs()
            .first()
            .ok_or_else(|| eyre::eyre!("mod loader has no config directory"))?;
        let manifest_path = DeployPathBuf::new(config_dir)?.join(manifest::FILE_NAME)?;

        let loader_owned_globs = build_globs(&[
            // Loader payload installed at the profile root. Restricted to
            // well-known mod loader files; mods live in mirror dirs and never
            // decide loader ownership.
            "bepinex/core/**",
            "doorstop_config.ini",
            "winhttp.dll",
            ".doorstop_version",
            "doorstop_libs/**",
            "dotnet/**",
        ])?;

        let loader_markers = [DeployPathBuf::new("BepInEx/core")?].into_iter().collect();

        // Patterns are matched against lowercased deploy paths.
        let exclude_globs = build_globs(&[
            "bepinex/cache/**",
            "bepinex/dumpedassemblies/**",
            "bepinex/interop/**",
            "renderer/bepinex/cache/**",
            "renderer/bepinex/dumpedassemblies/**",
            "renderer/bepinex/interop/**",
            "**/logoutput.log",
            // Package metadata mods only need locally. `manifest.json` is
            // deliberately not excluded: some mods read it at runtime.
            "bepinex/plugins/*/{readme.md,changelog.md,icon.png}",
            "renderer/bepinex/plugins/*/{readme.md,changelog.md,icon.png}",
            &format!(
                "**/{MANIFEST}",
                MANIFEST = manifest::FILE_NAME.to_lowercase()
            ),
        ])?;

        Ok(Self {
            mirror_root,
            mirror_dirs,
            loader_owned_globs,
            loader_markers,
            manifest_path,
            exclude_globs,
        })
    }

    /// Strips the mirror root prefix from `path`, e.g. `BepInEx/plugins/x`
    /// becomes `plugins/x` on a restricted host.
    pub fn strip_mirror_root(&self, path: &DeployPath) -> Option<DeployPathBuf> {
        path.strip_prefix(self.mirror_root.as_str())
    }

    /// Whether the manifest recorded a loader deployment, meaning Gale (and
    /// not the host) owns the loader files on the server.
    pub fn owns_loader(&self, manifest: &manifest::DeploymentManifest) -> bool {
        manifest.files.keys().any(|path| {
            self.loader_owned_globs
                .is_match(path.as_str().to_lowercase())
        })
    }

    /// Whether `path` is covered by the mirrored directories.
    pub fn is_mirrored(&self, path: &DeployPath) -> bool {
        self.mirror_dirs.iter().any(|dir| dir.is_ancestor_of(path))
    }

    /// Whether a local file should be deployed. `host_managed` means the
    /// remote host provides the loader itself, in which case only mirror dirs
    /// are writable and everything else is left alone.
    pub fn deploys(&self, path: &DeployPath, host_managed: bool) -> bool {
        if self.is_internal(path) || self.is_excluded(path) {
            return false;
        }

        !host_managed || self.is_mirrored(path)
    }

    /// Whether a directory walk should descend into `path`.
    pub fn descends(&self, path: &DeployPath, host_managed: bool) -> bool {
        if self.is_internal(path) || self.is_excluded(path) {
            return false;
        }

        if !host_managed {
            return true;
        }

        // Only descend into mirror dirs, their ancestors and their contents.
        self.mirror_dirs.iter().any(|dir| {
            dir.as_path() == path || dir.is_ancestor_of(path) || path.is_ancestor_of(dir)
        })
    }

    fn is_internal(&self, path: &DeployPath) -> bool {
        path.segments()
            .next()
            .is_some_and(|name| PROFILE_INTERNAL_NAMES.contains(&name))
    }

    fn is_excluded(&self, path: &DeployPath) -> bool {
        self.exclude_globs.is_match(path.as_str().to_lowercase())
    }
}

fn build_globs(patterns: &[&str]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(GlobBuilder::new(pattern).literal_separator(true).build()?);
    }
    Ok(builder.build()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bepinex() -> DeploymentSpec {
        DeploymentSpec::for_loader(&ModLoader {
            package_name: None,
            file_target: None,
            kind: crate::game::mod_loader::ModLoaderKind::BepInEx {
                extra_subdirs: Vec::new(),
            },
        })
        .unwrap()
    }

    fn path(value: &str) -> DeployPathBuf {
        DeployPathBuf::new(value).unwrap()
    }

    #[test]
    fn mirror_dirs_cover_mod_and_config_dirs() {
        let spec = bepinex();

        assert!(spec.is_mirrored(&path("BepInEx/plugins/Mod/Mod.dll")));
        assert!(spec.is_mirrored(&path("BepInEx/config/mod.cfg")));
        assert!(!spec.is_mirrored(&path("BepInEx/core/bepinex.dll")));
        assert!(!spec.is_mirrored(&path("doorstop_config.ini")));
        assert_eq!(spec.mirror_root.as_str(), "BepInEx");
        assert_eq!(
            spec.manifest_path.as_str(),
            "BepInEx/config/.gale-server-manifest.json"
        );
    }

    #[test]
    fn excludes_generated_and_internal_paths() {
        let spec = bepinex();

        for excluded in [
            "BepInEx/cache/x",
            "BepInEx/DumpedAssemblies/Game.dll",
            "BepInEx/interop/x.dll",
            "BepInEx/LogOutput.log",
            "BepInEx/plugins/Mod/README.md",
            "BepInEx/plugins/Mod/icon.png",
            "profile.json",
            "mods.yml",
            "_state/Author-Mod.json",
        ] {
            assert!(!spec.deploys(&path(excluded), false), "deployed {excluded}");
        }

        for deployed in [
            "BepInEx/plugins/Mod/Mod.dll",
            "BepInEx/plugins/Mod/manifest.json",
            "BepInEx/config/mod.cfg",
            "doorstop_config.ini",
            "BepInEx/core/bepinex.dll",
        ] {
            assert!(spec.deploys(&path(deployed), false), "skipped {deployed}");
        }
    }

    #[test]
    fn managed_hosts_only_receive_mirror_dirs() {
        let spec = bepinex();

        assert!(spec.deploys(&path("BepInEx/plugins/Mod/Mod.dll"), true));
        assert!(!spec.deploys(&path("BepInEx/core/bepinex.dll"), true));
        assert!(!spec.deploys(&path("doorstop_config.ini"), true));
    }

    #[test]
    fn detects_loader_ownership_from_manifest() {
        let spec = bepinex();
        let mut manifest = manifest::DeploymentManifest::default();

        assert!(!spec.owns_loader(&manifest));

        manifest.files.insert(
            path("BepInEx/core/bepinex.dll"),
            manifest::ManifestEntry {
                hash: "x".into(),
                size: 1,
            },
        );
        assert!(spec.owns_loader(&manifest));
    }

    #[test]
    fn rejects_loaders_without_deployment_support() {
        let melon = ModLoader {
            package_name: None,
            file_target: None,
            kind: crate::game::mod_loader::ModLoaderKind::MelonLoader {
                extra_subdirs: Vec::new(),
            },
        };

        assert!(DeploymentSpec::for_loader(&melon).is_err());
    }
}
