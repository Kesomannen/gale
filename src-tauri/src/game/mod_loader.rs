use serde::{Deserialize, Serialize};

use crate::profile::install::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModLoader<'a> {
    #[serde(default)]
    pub package_name: Option<&'a str>,
    #[serde(flatten)]
    pub kind: ModLoaderKind<'a>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "name")]
pub enum ModLoaderKind<'a> {
    BepInEx {
        #[serde(default, borrow, rename = "subdirs")]
        extra_subdirs: Vec<Subdir<'a>>,
    },
    BepisLoader {
        #[serde(default, borrow, rename = "subdirs")]
        extra_subdirs: Vec<Subdir<'a>>,
    },
    MelonLoader {
        #[serde(default, borrow, rename = "subdirs")]
        extra_subdirs: Vec<Subdir<'a>>,
    },
    Northstar {},
    GDWeave {},
    Shimloader {},
    Lovely {},
    ReturnOfModding {
        files: Vec<&'a str>,
    },
}

impl ModLoader<'_> {
    pub fn as_str(&self) -> &'static str {
        match &self.kind {
            ModLoaderKind::BepInEx { .. } => "BepInEx",
            ModLoaderKind::BepisLoader { .. } => "BepisLoader",
            ModLoaderKind::MelonLoader { .. } => "MelonLoader",
            ModLoaderKind::Northstar {} => "Northstar",
            ModLoaderKind::GDWeave {} => "GDWeave",
            ModLoaderKind::Shimloader {} => "Shimloader",
            ModLoaderKind::Lovely {} => "Lovely",
            ModLoaderKind::ReturnOfModding { .. } => "ReturnOfModding",
        }
    }

    /// Checks for the mod loader's own package on Thunderstore.
    fn is_loader_package(&self, full_name: &str) -> bool {
        if let Some(package_name) = self.package_name {
            full_name == package_name
        } else {
            match &self.kind {
                ModLoaderKind::BepInEx { .. } => full_name.starts_with("BepInEx-BepInExPack"),
                ModLoaderKind::BepisLoader { .. } => {
                    full_name == "ResoniteModding-BepisLoader"
                        || full_name == "ResoniteModding-BepInExRenderer"
                }
                ModLoaderKind::MelonLoader { .. } => full_name == "LavaGang-MelonLoader",
                ModLoaderKind::GDWeave {} => full_name == "NotNet-GDWeave",
                ModLoaderKind::Northstar {} => full_name == "northstar-Northstar",
                ModLoaderKind::Shimloader {} => full_name == "Thunderstore-unreal_shimloader",
                ModLoaderKind::Lovely {} => full_name == "Thunderstore-lovely",
                ModLoaderKind::ReturnOfModding { .. } => {
                    full_name == "ReturnOfModding-ReturnOfModding"
                }
            }
        }
    }

    pub fn log_path(&self) -> Option<&str> {
        match &self.kind {
            ModLoaderKind::BepInEx { .. } => Some("BepInEx/LogOutput.log"),
            ModLoaderKind::BepisLoader { .. } => Some("BepInEx/LogOutput.log"),
            ModLoaderKind::MelonLoader { .. } => Some("MelonLoader/Latest.log"),
            ModLoaderKind::GDWeave {} => Some("GDWeave/GDWeave.log"),
            ModLoaderKind::Northstar {} => None,
            ModLoaderKind::Shimloader {} => None,
            ModLoaderKind::Lovely {} => Some("mods/lovely/log"),
            ModLoaderKind::ReturnOfModding { .. } => None,
        }
    }

    pub fn mod_config_dirs(&self) -> &[&str] {
        match &self.kind {
            ModLoaderKind::BepInEx { .. } => &["BepInEx/config"],
            ModLoaderKind::BepisLoader { .. } => &["BepInEx/config", "Renderer/BepInEx/config"],
            ModLoaderKind::MelonLoader { .. } => &["."],
            ModLoaderKind::GDWeave {} => &["GDWeave/configs"],
            ModLoaderKind::Northstar {} => &["."],
            ModLoaderKind::Shimloader {} => &["."],
            ModLoaderKind::Lovely {} => &["."],
            ModLoaderKind::ReturnOfModding { .. } => &["ReturnOfModding/config"],
        }
    }
}

impl ModLoader<'static> {
    pub fn installer_for(&'static self, package_name: &str) -> Box<dyn PackageInstaller> {
        match (self.is_loader_package(package_name), &self.kind) {
            (true, ModLoaderKind::BepInEx { .. }) => Box::new(BepinexInstaller),
            (false, ModLoaderKind::BepInEx { extra_subdirs, .. }) => {
                const SUBDIRS: &[Subdir] = &[
                    Subdir::flat_separated("plugins", "BepInEx/plugins"),
                    Subdir::flat_separated("patchers", "BepInEx/patchers"),
                    Subdir::flat_separated("monomod", "BepInEx/monomod").extension(".mm.dll"),
                    Subdir::flat_separated("core", "BepInEx/core"),
                    Subdir::untracked("config", "BepInEx/config").mutable(),
                ];

                Box::new(
                    SubdirInstaller::new(SUBDIRS)
                        .with_default(0)
                        .with_extras(extra_subdirs),
                )
            }

            (true, ModLoaderKind::BepisLoader { .. }) => Box::new(BepinexInstaller),
            (false, ModLoaderKind::BepisLoader { extra_subdirs, .. }) => {
                const SUBDIRS: &[Subdir] = &[
                    Subdir::flat_separated("Renderer", "Renderer/BepInEx/plugins"),
                    Subdir::flat_separated("plugins", "BepInEx/plugins"),
                    Subdir::flat_separated("patchers", "BepInEx/patchers"),
                    Subdir::flat_separated("monomod", "BepInEx/monomod").extension(".mm.dll"),
                    Subdir::flat_separated("core", "BepInEx/core"),
                    Subdir::untracked("config", "BepInEx/config").mutable(),
                ];

                Box::new(
                    SubdirInstaller::new(SUBDIRS)
                        .with_default(1)
                        .with_extras(extra_subdirs),
                )
            }

            (true, ModLoaderKind::MelonLoader { .. }) => {
                const FILES: &[&str] = &[
                    "dobby.dll",
                    "version.dll",
                    "MelonLoader/Dependencies",
                    "MelonLoader/Documentation",
                    "MelonLoader/net6",
                    "MelonLoader/net35",
                ];

                Box::new(ExtractInstaller::new(FILES, FlattenTopLevel::No))
            }
            (false, ModLoaderKind::MelonLoader { extra_subdirs }) => {
                const SUBDIRS: &[Subdir] = &[
                    Subdir::tracked("UserLibs", "UserLibs").extension(".lib.dll"),
                    Subdir::tracked("Managed", "MelonLoader/Managed").extension(".managed.dll"),
                    Subdir::tracked("Mods", "Mods").extension(".dll"),
                    Subdir::separated("ModManager", "UserData/ModManager"),
                    Subdir::tracked("MelonLoader", "MelonLoader"),
                    Subdir::tracked("Libs", "MelonLoader/Libs"),
                ];
                const IGNORED: &[&str] = &["manifest.json", "icon.png", "README.md"];

                Box::new(
                    SubdirInstaller::new(SUBDIRS)
                        .with_default(2)
                        .with_extras(extra_subdirs)
                        .with_ignored_files(IGNORED),
                )
            }

            (true, ModLoaderKind::GDWeave {}) => {
                const FILES: &[&str] = &["winmm.dll", "GDWeave/core"];

                Box::new(ExtractInstaller::new(FILES, FlattenTopLevel::No))
            }
            (false, ModLoaderKind::GDWeave {}) => Box::new(GDWeaveModInstaller),

            (true, ModLoaderKind::Northstar {}) => {
                const FILES: &[&str] = &[
                    "Northstar.dll",
                    "NorthstarLauncher.exe",
                    "r2ds.bat",
                    "bin",
                    "R2Northstar/plugins",
                    "R2Northstar/mods/Northstar.Client",
                    "R2Northstar/mods/Northstar.Custom",
                    "R2Northstar/mods/Northstar.CustomServers",
                    "R2Northstar/mods/md5sum.text",
                ];

                Box::new(ExtractInstaller::new(FILES, FlattenTopLevel::Yes))
            }
            (false, ModLoaderKind::Northstar {}) => {
                const SUBDIRS: &[Subdir] = &[Subdir::tracked("mods", "R2Northstar/mods")];
                const IGNORED: &[&str] = &["manifest.json", "icon.png", "README.md", "LICENSE"];

                Box::new(SubdirInstaller::new(SUBDIRS).with_ignored_files(IGNORED))
            }

            (true, ModLoaderKind::Shimloader {}) => Box::new(ShimloaderInstaller),
            (false, ModLoaderKind::Shimloader {}) => {
                const SUBDIRS: &[Subdir] = &[
                    Subdir::flat_separated("mod", "shimloader/mod"),
                    Subdir::flat_separated("pak", "shimloader/pak"),
                    Subdir::untracked("cfg", "shimloader/cfg").mutable(),
                ];

                Box::new(SubdirInstaller::new(SUBDIRS).with_default(0))
            }

            (true, ModLoaderKind::ReturnOfModding { files }) => {
                Box::new(ExtractInstaller::new(files, FlattenTopLevel::Yes))
            }
            (false, ModLoaderKind::ReturnOfModding { .. }) => {
                const SUBDIRS: &[Subdir] = &[
                    Subdir::separated("plugins", "ReturnOfModding/plugins"),
                    Subdir::separated("plugins_data", "ReturnOfModding/plugins_data"),
                    Subdir::separated("config", "ReturnOfModding/config").mutable(),
                ];

                Box::new(SubdirInstaller::new(SUBDIRS).with_default(0))
            }

            (true, ModLoaderKind::Lovely {}) => {
                const FILES: &[&str] = &["version.dll"];

                Box::new(ExtractInstaller::new(FILES, FlattenTopLevel::No))
            }
            (false, ModLoaderKind::Lovely {}) => {
                const SUBDIRS: &[Subdir] = &[Subdir::separated("", "mods")];

                Box::new(SubdirInstaller::new(SUBDIRS).with_default(0))
            }
        }
    }

    pub fn proxy_dll(&'static self) -> Option<&'static str> {
        match &self.kind {
            ModLoaderKind::BepInEx { .. } => Some("winhttp"),
            ModLoaderKind::GDWeave {} => Some("winmm"),
            ModLoaderKind::ReturnOfModding { files } => Some(files[0]),
            _ => None,
        }
    }
}
