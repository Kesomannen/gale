use std::{
    borrow::Cow,
    hash::{self, Hash},
};

use heck::{ToKebabCase, ToPascalCase};
use serde::{Deserialize, Serialize};

const JSON: &str = include_str!("../games.json");

lazy_static! {
    static ref GAMES: Vec<GameInner<'static>> = serde_json::from_str(JSON).unwrap();
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct GameData<'a> {
    name: &'a str,
    #[serde(default)]
    slug: Option<&'a str>,
    #[serde(default)]
    popular: bool,
    mod_loader: ModLoader,
    #[serde(default, rename = "r2dirName")]
    r2_dir_name: Option<&'a str>,
    #[serde(default)]
    extra_sub_dirs: Vec<Subdir<'a>>,
    #[serde(borrow)]
    platforms: Platforms<'a>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum ModLoader {
    BepInEx,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Platforms<'a> {
    #[serde(borrow)]
    steam: Steam<'a>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase", untagged)]
enum Steam<'a> {
    Concise(u32),
    #[serde(rename_all = "camelCase")]
    Full {
        id: u32,
        dir_name: &'a str,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase", from = "GameData")]
struct GameInner<'a> {
    name: &'a str,
    slug: Cow<'a, str>,
    steam_name: &'a str,
    steam_id: u32,
    mod_loader: ModLoader,
    r2_dir_name: Cow<'a, str>,
    extra_sub_dirs: Vec<Subdir<'a>>,
    popular: bool,
}

impl<'a> From<GameData<'a>> for GameInner<'a> {
    fn from(value: GameData<'a>) -> Self {
        let GameData {
            name,
            slug,
            popular,
            mod_loader,
            r2_dir_name,
            extra_sub_dirs,
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

        let (steam_id, steam_name) = match platforms.steam {
            Steam::Concise(id) => (id, name),
            Steam::Full { id, dir_name } => (id, dir_name),
        };

        Self {
            name,
            slug,
            steam_name,
            steam_id,
            mod_loader,
            r2_dir_name,
            extra_sub_dirs,
            popular,
        }
    }
}

impl PartialEq for GameInner<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.slug == other.slug
    }
}

impl Hash for GameInner<'_> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.slug.hash(state);
    }
}

fn separate_mods_default() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Subdir<'a> {
    name: &'a str,
    /// Whether to separate mods into `author-name` dirs.
    #[serde(default = "separate_mods_default")]
    separate_mods: bool,
}

impl<'a> Subdir<'a> {
    pub const fn new(name: &'a str) -> Self {
        Self {
            name,
            separate_mods: true,
        }
    }

    pub const fn dont_separate_mods(mut self) -> Self {
        self.separate_mods = false;
        self
    }

    pub fn name(&self) -> &'a str {
        self.name
    }

    pub fn separate_mods(&self) -> bool {
        self.separate_mods
    }
}

impl ModLoader {
    pub fn subdirs(&self) -> impl Iterator<Item = &Subdir<'static>> {
        match self {
            ModLoader::BepInEx => {
                const SUBDIRS: &[Subdir] = &[
                    Subdir::new("plugins"),
                    Subdir::new("patchers"),
                    Subdir::new("monomod"),
                    Subdir::new("core"),
                    Subdir::new("config").dont_separate_mods(),
                ];
                SUBDIRS.iter()
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Hash, Serialize)]
#[serde(transparent)]
pub struct Game(&'static GameInner<'static>);

impl Game {
    pub fn all() -> impl Iterator<Item = Self> {
        GAMES.iter().map(Self)
    }

    pub fn from_slug(slug: &str) -> Option<Self> {
        GAMES.iter().find(|game| game.slug == slug).map(Self)
    }

    pub fn subdirs(self) -> impl Iterator<Item = &'static Subdir<'static>> {
        self.0
            .mod_loader
            .subdirs()
            .chain(self.0.extra_sub_dirs.iter())
    }

    pub fn name(self) -> &'static str {
        self.0.name
    }

    pub fn slug(self) -> &'static str {
        &self.0.slug
    }

    pub fn steam_name(self) -> &'static str {
        self.0.steam_name
    }

    pub fn steam_id(self) -> u32 {
        self.0.steam_id
    }

    pub fn mod_loader(self) -> ModLoader {
        self.0.mod_loader.clone()
    }

    pub fn r2_dir_name(self) -> &'static str {
        &self.0.r2_dir_name
    }

    pub fn is_popular(self) -> bool {
        self.0.popular
    }
}
