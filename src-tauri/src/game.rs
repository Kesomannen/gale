use std::{
    borrow::Cow,
    hash::{self, Hash},
    path::PathBuf,
};

use heck::{ToKebabCase, ToPascalCase};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

use crate::profile::install::{
    BepinexInstaller, ExtractInstaller, FlattenTopLevel, GDWeaveModInstaller, PackageInstaller,
    ShimloaderInstaller, Subdir, SubdirInstaller,
};

const GAMES_JSON: &str = include_str!("../games.json");

lazy_static! {
    static ref GAMES: Vec<GameData<'static>> = serde_json::from_str(GAMES_JSON).unwrap();
}

pub type Game = &'static GameData<'static>;

pub fn all() -> impl Iterator<Item = Game> {
    GAMES.iter()
}

pub fn from_slug(slug: &str) -> Option<Game> {
    GAMES.iter().find(|game| game.slug == slug)
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct JsonGame<'a> {
    name: &'a str,
    #[serde(default)]
    slug: Option<&'a str>,
    #[serde(default)]
    popular: bool,
    #[serde(default)]
    server: bool,
    #[serde(default, rename = "r2dirName")]
    r2_dir_name: Option<&'a str>,
    #[serde(borrow)]
    mod_loader: ModLoader<'a>,
    #[serde(borrow, default)]
    platforms: Platforms<'a>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Platforms<'a> {
    #[serde(borrow)]
    pub steam: Option<Steam<'a>>,
    #[serde(borrow)]
    pub epic_games: Option<EpicGames<'a>>,
    pub oculus: Option<Oculus>,
    pub origin: Option<Origin>,
    #[serde(borrow)]
    pub xbox_store: Option<XboxStore<'a>>,
}

impl Platforms<'_> {
    pub fn has(&self, platform: Platform) -> bool {
        match platform {
            Platform::Steam => self.steam.is_some(),
            Platform::EpicGames => self.epic_games.is_some(),
            Platform::Oculus => self.oculus.is_some(),
            Platform::Origin => self.origin.is_some(),
            Platform::XboxStore => self.xbox_store.is_some(),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = Platform> + '_ {
        Platform::iter().filter(|platform| self.has(*platform))
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase", from = "JsonGame")]
pub struct GameData<'a> {
    pub name: &'a str,
    pub slug: Cow<'a, str>,
    pub r2_dir_name: Cow<'a, str>,
    pub popular: bool,
    pub server: bool,
    pub mod_loader: ModLoader<'a>,
    pub platforms: Platforms<'a>,
}

impl<'a> From<JsonGame<'a>> for GameData<'a> {
    fn from(value: JsonGame<'a>) -> Self {
        let JsonGame {
            name,
            slug,
            popular,
            server,
            r2_dir_name,
            mod_loader,
            platforms,
        } = value;

        let slug = match slug {
            Some(slug) => Cow::Borrowed(slug),
            None => Cow::Owned(name.to_kebab_case()),
        };

        let r2_dir_name = match r2_dir_name {
            Some(name) => Cow::Borrowed(name),
            None => Cow::Owned(slug.to_pascal_case()),
        };

        Self {
            name,
            slug,
            r2_dir_name,
            popular,
            server,
            mod_loader,
            platforms,
        }
    }
}

impl PartialEq for GameData<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.slug == other.slug
    }
}

impl Eq for GameData<'_> {}

impl Hash for GameData<'_> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.slug.hash(state);
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy, Display, EnumIter)]
#[serde(rename_all = "camelCase")]
pub enum Platform {
    #[default]
    Steam,
    EpicGames,
    Oculus,
    Origin,
    XboxStore,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Steam<'a> {
    pub id: u32,
    #[serde(default)]
    pub dir_name: Option<&'a str>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EpicGames<'a> {
    #[serde(default)]
    pub identifier: Option<&'a str>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Oculus {}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Origin {}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct XboxStore<'a> {
    #[serde(default)]
    pub identifier: Option<&'a str>,
}

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
    pub fn to_str(&self) -> &'static str {
        match &self.kind {
            ModLoaderKind::BepInEx { .. } => "BepInEx",
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

    pub fn log_path(&self) -> &str {
        match &self.kind {
            ModLoaderKind::BepInEx { .. } => "BepInEx/LogOutput.log",
            ModLoaderKind::MelonLoader { .. } => "MelonLoader/Latest.log",
            ModLoaderKind::GDWeave {} => "GDWeave/GDWeave.log",
            ModLoaderKind::Northstar {} => "",
            ModLoaderKind::Shimloader {} => "",
            ModLoaderKind::Lovely {} => "",
            ModLoaderKind::ReturnOfModding { .. } => "",
        }
    }

    pub fn config_path(&self) -> PathBuf {
        match &self.kind {
            ModLoaderKind::BepInEx { .. } => ["BepInEx", "config"].iter().collect(),
            ModLoaderKind::MelonLoader { .. } => PathBuf::new(),
            ModLoaderKind::GDWeave {} => ["GDWeave", "configs"].iter().collect(),
            ModLoaderKind::Northstar {} => PathBuf::new(),
            ModLoaderKind::Shimloader {} => PathBuf::new(),
            ModLoaderKind::Lovely {} => PathBuf::new(),
            ModLoaderKind::ReturnOfModding { .. } => ["ReturnOfModding", "config"].iter().collect(),
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
}
