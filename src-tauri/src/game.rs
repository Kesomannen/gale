use std::{
    borrow::Cow,
    hash::{self, Hash},
    path::PathBuf,
};

use heck::{ToKebabCase, ToPascalCase};
use serde::{Deserialize, Serialize};

const JSON: &str = include_str!("../games.json");

lazy_static! {
    static ref GAMES: Vec<GameData<'static>> = serde_json::from_str(JSON).unwrap();
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
    platforms: JsonPlatforms<'a>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct JsonPlatforms<'a> {
    steam: Option<Steam<'a>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase", from = "JsonGame")]
pub struct GameData<'a> {
    pub name: &'a str,
    pub slug: Cow<'a, str>,
    pub r2_dir_name: Cow<'a, str>,
    pub popular: bool,
    pub mod_loader: ModLoader<'a>,
    pub platforms: Vec<Platform<'a>>,
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

        let platforms = {
            let mut vec = Vec::new();

            if let Some(steam) = platforms.steam {
                vec.push(Platform::Steam(steam));
            }

            vec
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase", tag = "name")]
pub enum Platform<'a> {
    Steam(Steam<'a>),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Steam<'a> {
    pub id: u32,
    #[serde(default)]
    pub dir_name: Cow<'a, str>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(tag = "name")]
pub enum ModLoader<'a> {
    #[serde(rename_all = "camelCase")]
    BepInEx {
        #[serde(borrow)]
        extra_sub_dirs: Vec<Subdir<'a>>,
    },
    MelonLoader,
}

fn default_true() -> bool {
    true
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Subdir<'a> {
    pub name: &'a str,
    pub target: &'a str,
    /// Whether to separate mods into `author-name` dirs.
    #[serde(default = "default_true")]
    pub separate_mods: bool,
    #[serde(default)]
    pub mutable: bool,
    #[serde(default)]
    pub extension: Option<&'a str>,
}

impl<'a> Subdir<'a> {
    pub const fn new(name: &'a str, target: &'a str) -> Self {
        Self {
            name,
            target,
            separate_mods: true,
            mutable: false,
            extension: None,
        }
    }

    pub const fn dont_separate_mods(mut self) -> Self {
        self.separate_mods = false;
        self
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

impl<'a> ModLoader<'a> {
    pub fn log_path(&self) -> PathBuf {
        match self {
            ModLoader::BepInEx { .. } => &["BepInEx", "LogOutput.log"],
            ModLoader::MelonLoader => &["MelonLoader", "Latest.log"],
        }
        .iter()
        .collect()
    }

    pub fn default_subdir(&self) -> Option<&Subdir> {
        match self {
            ModLoader::BepInEx { .. } => {
                const SUBDIR: &Subdir = &Subdir::new("plugins", "BepInEx/plugins");
                Some(SUBDIR)
            }
            ModLoader::MelonLoader => None,
        }
    }

    pub fn subdirs(&self) -> Box<dyn Iterator<Item = &Subdir> + '_> {
        match self {
            ModLoader::BepInEx { extra_sub_dirs } => {
                const SUBDIRS: &[Subdir] = &[
                    Subdir::new("plugins", "BepInEx/plugins"),
                    Subdir::new("patchers", "BepInEx/patchers"),
                    Subdir::new("monomod", "BepInEx/monomod").extension(".mm.dll"),
                    Subdir::new("core", "BepInEx/core"),
                    Subdir::new("config", "BepInEx/config")
                        .dont_separate_mods()
                        .mutable(),
                ];
                Box::new(SUBDIRS.iter().chain(extra_sub_dirs))
            }
            ModLoader::MelonLoader => {
                const SUBDIRS: &[Subdir] = &[
                    Subdir::new("Mods", "Mods")
                        .dont_separate_mods()
                        .extension(".dll"),
                    Subdir::new("Plugins", "Plugins")
                        .dont_separate_mods()
                        .extension(".plugin.dll"),
                    Subdir::new("MelonLoader", "MelonLoader").dont_separate_mods(),
                    Subdir::new("Managed", "MelonLoader/Managed")
                        .dont_separate_mods()
                        .extension(".managed.dll"),
                    Subdir::new("Libs", "MelonLoader/Libs")
                        .dont_separate_mods()
                        .extension(".lib.dll"),
                    Subdir::new("UserData", "UserData").dont_separate_mods(),
                    Subdir::new("CustomItems", "UserData/CustomItems")
                        .dont_separate_mods()
                        .extension(".melon"),
                    Subdir::new("CustomMaps", "UserData/CustomMaps")
                        .dont_separate_mods()
                        .extension(".bcm"),
                    Subdir::new("PlayerModels", "UserData/PlayerModels")
                        .dont_separate_mods()
                        .extension(".body"),
                    Subdir::new("CustomLoadScreens", "UserData/CustomLoadScreens")
                        .dont_separate_mods()
                        .extension(".load"),
                    Subdir::new("Music", "UserData/Music")
                        .dont_separate_mods()
                        .extension(".wav"),
                    Subdir::new("Food", "UserData/Food")
                        .dont_separate_mods()
                        .extension(".food"),
                    Subdir::new("Scoreworks", "UserData/Scoreworks")
                        .dont_separate_mods()
                        .extension(".sw"),
                    Subdir::new("CustomSkins", "UserData/CustomSkins")
                        .dont_separate_mods()
                        .extension(".png"),
                    Subdir::new("Grenades", "UserData/Grenades")
                        .dont_separate_mods()
                        .extension(".grenade"),
                ];

                Box::new(SUBDIRS.iter())
            }
        }
    }

    pub fn match_subdir(&self, name: &str) -> Option<&Subdir> {
        self.subdirs().find(|subdir| {
            subdir.name == name || subdir.extension.is_some_and(|ext| name.ends_with(ext))
        })
    }
}

pub type Game = &'static GameData<'static>;

pub fn all() -> impl Iterator<Item = Game> {
    GAMES.iter()
}

pub fn from_slug(slug: &str) -> Option<Game> {
    GAMES.iter().find(|game| game.slug == slug)
}
