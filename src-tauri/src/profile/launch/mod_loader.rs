use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use eyre::{bail, eyre, Context, OptionExt, Result};
use tracing::{info, warn};

use crate::{
    game::mod_loader::{ModLoader, ModLoaderKind},
    util::error::IoResultExt,
};

pub struct ArgsContext<'a> {
    command: &'a mut Command,
    profile_dir: &'a Path,
    is_proton: bool,
}

impl<'a> ArgsContext<'a> {
    pub fn new(command: &'a mut Command, profile_dir: &'a Path, is_proton: bool) -> Self {
        Self {
            command,
            profile_dir,
            is_proton,
        }
    }

    fn format_path(&self, path: impl Into<PathBuf>) -> Result<String> {
        let str_path = path
            .into()
            .into_os_string()
            .into_string()
            .map_err(|path| eyre!("path is not valid UTF-8: {}", path.display()))?;

        if self.is_proton {
            Ok(format!("Z:{}", str_path))
        } else {
            Ok(str_path)
        }
    }

    pub fn add_args(&mut self, mod_loader: &ModLoader) -> Result<()> {
        match &mod_loader.kind {
            ModLoaderKind::BepInEx { .. } => self.add_bepinex_args(),
            ModLoaderKind::BepisLoader { .. } => self.add_bepisloader_args(),
            ModLoaderKind::MelonLoader { .. } => self.add_melon_loader_args(),
            ModLoaderKind::Northstar {} => self.add_northstar_args(),
            ModLoaderKind::GDWeave {} => self.add_gd_weave_args(),
            ModLoaderKind::Shimloader {} => self.add_shimloader_args(),
            ModLoaderKind::Lovely {} => self.add_lovely_args(),
            ModLoaderKind::ReturnOfModding { .. } => self.add_return_of_modding_args(),
        }
    }

    fn add_bepinex_args(&mut self) -> Result<()> {
        let (enable_prefix, target_prefix) = self.doorstop_args(None)?;
        let preloader = self.bepinex_preloader_path(None)?;
        let preloader_path = self.format_path(&preloader)?;

        self.command
            .args([enable_prefix, "true", target_prefix])
            .arg(preloader_path);

        #[cfg(target_os = "macos")]
        self.add_macos_doorstop_env(&preloader);

        Ok(())
    }

    /// Injects doorstop into the game with DYLD environment variables, the
    /// equivalent of BepInEx's `run_bepinex.sh`. The doorstop CLI arguments
    /// only work on Windows, where the proxy dll is loaded by the game itself.
    #[cfg(target_os = "macos")]
    fn add_macos_doorstop_env(&mut self, preloader: &Path) {
        const DYLIB_PATHS: &[&str] = &[
            "doorstop_libs/libdoorstop_x64.dylib", // doorstop v3
            "libdoorstop.dylib",                   // doorstop v4
        ];

        let Some(dylib) = DYLIB_PATHS
            .iter()
            .map(|path| self.profile_dir.join(path))
            .find(|path| path.exists())
        else {
            warn!(
                "no doorstop library found in the profile - \
                the game will launch without mods unless the launch is otherwise modded"
            );
            return;
        };

        info!("injecting doorstop via DYLD from {}", dylib.display());

        self.command
            .env("DYLD_INSERT_LIBRARIES", &dylib)
            .env("DYLD_LIBRARY_PATH", dylib.parent().unwrap())
            .env(
                "DOORSTOP_MONO_DLL_SEARCH_PATH_OVERRIDE",
                preloader.parent().unwrap_or(preloader),
            )
            // doorstop v3 configuration
            .env("DOORSTOP_ENABLE", "TRUE")
            .env("DOORSTOP_INVOKE_DLL_PATH", preloader)
            // doorstop v4 configuration
            .env("DOORSTOP_ENABLED", "1")
            .env("DOORSTOP_TARGET_ASSEMBLY", preloader);
    }

    fn add_bepisloader_args(&mut self) -> Result<()> {
        // Don't use format_path as BepisLoader escapes Proton and therefore
        // does not recognize the Z: prefix, see https://github.com/Kesomannen/gale/issues/627
        let bepinex_target = self.profile_dir.join("BepInEx");

        self.command
            .arg("--hookfxr-enable")
            .arg("--bepinex-target")
            .arg(bepinex_target);

        let preloader_result = self
            .bepinex_preloader_path(Some("Renderer"))
            .and_then(|path| self.format_path(path));

        match preloader_result {
            Ok(preloader_path) => {
                let (enable_prefix, target_prefix) = self.doorstop_args(Some(4))?;
                self.command
                    .arg(enable_prefix)
                    .arg("true")
                    .arg(target_prefix)
                    .arg(preloader_path);
            }
            Err(err) => {
                warn!(err = ?err, "failed to find BepInEx preloader, launching without doorstep")
            }
        }

        Ok(())
    }

    fn bepinex_preloader_path(&mut self, prefix: Option<&str>) -> Result<PathBuf> {
        let mut core_dir = self.profile_dir.to_path_buf();

        if let Some(prefix) = prefix {
            core_dir.push(prefix);
        }

        core_dir.push("BepInEx");
        core_dir.push("core");

        const PRELOADER_NAMES: &[&str] = &[
            "BepInEx.Unity.Mono.Preloader.dll",
            "BepInEx.Unity.IL2CPP.dll",
            "BepInEx.Preloader.dll",
            "BepInEx.IL2CPP.dll",
        ];

        let result = core_dir
            .read_dir()
            .context("failed to read BepInEx core directory. Is BepInEx installed?")?
            .filter_map(|entry| entry.ok())
            .find(|entry| {
                let file_name = entry.file_name();
                PRELOADER_NAMES.iter().any(|name| file_name == **name)
            })
            .ok_or_eyre("BepInEx preloader not found. Your BepInEx installation may be corrupted.")?
            .path();

        Ok(result)
    }

    fn doorstop_args(
        &mut self,
        version_override: Option<u32>,
    ) -> Result<(&'static str, &'static str)> {
        let version = if let Some(v) = version_override {
            info!("using doorstop version override: {}", v);
            v
        } else {
            let path = self.profile_dir.join(".doorstop_version");

            if path.exists() {
                let version = fs::read_to_string(&path)
                    .fs_context("reading version file", &path)?
                    .split('.') // read only the major version number
                    .next()
                    .and_then(|str| str.parse().ok())
                    .ok_or_eyre("invalid version format")?;

                info!("doorstop version read: {}", version);
                version
            } else {
                warn!(".doorstop_version file is missing, defaulting to 3");
                3
            }
        };

        match version {
            3 => Ok(("--doorstop-enable", "--doorstop-target")),
            4 => Ok(("--doorstop-enabled", "--doorstop-target-assembly")),
            vers => bail!("unsupported doorstop version: {}", vers),
        }
    }

    fn add_melon_loader_args(&mut self) -> Result<()> {
        let profile_dir = self.format_path(self.profile_dir)?;

        self.command.arg("--melonloader.basedir").arg(profile_dir);

        let mono_assembly_exists = self
            .profile_dir
            .join("MelonLoader/Managed/Assembly-CSharp.dll")
            .exists();
        let il2cpp_assembly_exists = self
            .profile_dir
            .join("MelonLoader/Il2CppAssemblies/Assembly-CSharp.dll")
            .exists();

        if !mono_assembly_exists && !il2cpp_assembly_exists {
            self.command.arg("--melonloader.agfregenerate");
        }

        Ok(())
    }

    fn add_northstar_args(&mut self) -> Result<()> {
        let path = self.format_path(self.profile_dir.join("R2Northstar"))?;

        self.command
            .arg("-northstar")
            .arg(format!("-profile={path}"));

        Ok(())
    }

    fn add_gd_weave_args(&mut self) -> Result<()> {
        let path = self.format_path(self.profile_dir.join("GDWeave"))?;

        self.command
            .arg(format!("--gdweave-folder-override={path}"));

        Ok(())
    }

    fn add_shimloader_args(&mut self) -> Result<()> {
        let path = self.profile_dir.join("shimloader");
        let mod_path = self.format_path(path.join("mod"))?;
        let pak_path = self.format_path(path.join("pak"))?;
        let cfg_path = self.format_path(path.join("cfg"))?;

        self.command
            .arg("--mod-dir")
            .arg(mod_path)
            .arg("--pak-dir")
            .arg(pak_path)
            .arg("--cfg-dir")
            .arg(cfg_path);

        Ok(())
    }

    fn add_lovely_args(&mut self) -> Result<()> {
        let path = self.format_path(self.profile_dir.join("mods"))?;
        self.command.arg("--mod-dir").arg(path);

        Ok(())
    }

    fn add_return_of_modding_args(&mut self) -> Result<()> {
        let path = self.format_path(self.profile_dir)?;

        self.command.arg("--rom_modding_root_folder").arg(path);

        Ok(())
    }
}
