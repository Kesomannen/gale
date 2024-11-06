use std::{
    borrow::Cow,
    hash::{self, Hash},
    marker::PhantomData,
    path::PathBuf,
};

use heck::{ToKebabCase, ToPascalCase};
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use crate::profile::install::{
    BepinexInstaller, MelonLoaderInstaller, PackageInstaller, Subdir, SubdirInstaller,
};

const JSON: &str = include_str!("../games.json");

lazy_static! {
    static ref GAMES: Vec<GameData<'static>> = serde_json::from_str(JSON).unwrap();
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
    #[serde(default, rename = "r2dirName")]
    r2_dir_name: Option<&'a str>,
    #[serde(borrow)]
    mod_loader: ModLoader<'a>,
    #[serde(borrow)]
    platforms: Platforms<'a>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Platforms<'a> {
    #[serde(borrow)]
    pub steam: Option<Steam<'a>>,
    #[serde(borrow)]
    pub epic_games: Option<EpicGames<'a>>,
    pub oculus: Option<Oculus>,
    pub origin: Option<Origin>,
    #[serde(borrow)]
    pub xbox_game_pass: Option<XboxGamePass<'a>>,
}

impl Platforms<'_> {
    pub fn has(&self, platform: Platform) -> bool {
        match platform {
            Platform::Steam => self.steam.is_some(),
            Platform::EpicGames => self.epic_games.is_some(),
            Platform::Oculus => self.oculus.is_some(),
            Platform::Origin => self.origin.is_some(),
            Platform::XboxGamePass => self.xbox_game_pass.is_some(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase", from = "JsonGame")]
pub struct GameData<'a> {
    pub name: &'a str,
    pub slug: Cow<'a, str>,
    pub r2_dir_name: Cow<'a, str>,
    pub popular: bool,
    pub mod_loader: ModLoader<'a>,
    pub platforms: Platforms<'a>,
}

impl<'a> From<JsonGame<'a>> for GameData<'a> {
    fn from(value: JsonGame<'a>) -> Self {
        let JsonGame {
            name,
            slug,
            popular,
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

#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy, EnumIter)]
#[serde(rename_all = "camelCase")]
pub enum Platform {
    #[default]
    Steam,
    EpicGames,
    Oculus,
    Origin,
    XboxGamePass,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Steam<'a> {
    pub id: u32,
    #[serde(default)]
    pub dir_name: Cow<'a, str>,
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
pub struct XboxGamePass<'a> {
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
        extra_sub_dirs: Vec<Subdir<'a>>,
        #[serde(skip)]
        lifetime: PhantomData<&'a ()>,
    },
    MelonLoader {
        #[serde(default, borrow, rename = "subdirs")]
        extra_sub_dirs: Vec<Subdir<'a>>,
    },
}

impl<'a> ModLoader<'a> {
    pub fn installer(&self, package_name: &str) -> Box<dyn PackageInstaller> {
        match (self.is_loader_package(package_name), &self.kind) {
            (false, ModLoaderKind::BepInEx { extra_sub_dirs, .. }) => {
                const SUBDIRS: &[Subdir] = &[
                    Subdir::separate_flatten("plugins", "BepInEx/plugins"),
                    Subdir::separate_flatten("patchers", "BepInEx/patchers"),
                    Subdir::separate_flatten("monomod", "BepInEx/monomod").extension(".mm.dll"),
                    Subdir::separate_flatten("core", "BepInEx/core"),
                    Subdir::untracked("config", "BepInEx/config").mutable(),
                ];

                const DEFAULT: &Subdir = &Subdir::separate_flatten("plugins", "BepInEx/plugins");

                const IGNORED: &[&str] = &[];

                Box::new(SubdirInstaller::new(SUBDIRS, Some(DEFAULT), IGNORED))
            }
            (true, ModLoaderKind::BepInEx { .. }) => Box::new(BepinexInstaller),
            (false, ModLoaderKind::MelonLoader { extra_sub_dirs }) => {
                const SUBDIRS: &[Subdir] = &[
                    Subdir::track("UserLibs", "UserLibs").extension(".lib.dll"),
                    Subdir::track("Managed", "MelonLoader/Managed").extension(".managed.dll"),
                    Subdir::track("Mods", "Mods").extension(".dll"),
                    Subdir::separate("ModManager", "UserData/ModManager"),
                    Subdir::track("MelonLoader", "MelonLoader"),
                    Subdir::track("Libs", "MelonLoader/Libs"),
                ];

                const DEFAULT: &Subdir = &Subdir::track("Mods", "Mods");

                const IGNORED: &[&str] = &["manifest.json", "icon.png", "README.md"];

                Box::new(SubdirInstaller::new(SUBDIRS, Some(DEFAULT), IGNORED))
            }
            (true, ModLoaderKind::MelonLoader { .. }) => Box::new(MelonLoaderInstaller),
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
            }
        }
    }

    pub fn log_path(&self) -> PathBuf {
        match &self.kind {
            ModLoaderKind::BepInEx { .. } => &["BepInEx", "LogOutput.log"],
            ModLoaderKind::MelonLoader { .. } => &["MelonLoader", "Latest.log"],
        }
        .iter()
        .collect()
    }
}
