use heck::ToKebabCase;
use serde::{Deserialize, Serialize};
use std::hash::{self, Hash};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct GameData {
    name: String,
    #[serde(default)]
    slug: Option<String>,
    #[serde(default)]
    popular: bool,
    #[allow(unused)]
    mod_loader: ModLoader,
    #[serde(default, rename = "subdirs")]
    extra_sub_dirs: Vec<String>,
    platforms: Platforms,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum ModLoader {
    BepInEx,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Platforms {
    steam: Steam,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase", untagged)]
enum Steam {
    Concise(u32),
    Full { id: u32, dir_name: String },
}

#[derive(Serialize, Deserialize, Debug, Eq)]
#[serde(rename_all = "camelCase", from = "GameData")]
pub struct Game {
    pub name: String,
    pub slug: String,
    pub steam_name: String,
    pub steam_id: u32,
    pub mod_loader: ModLoader,
    pub r2_dir_name: String,
    pub extra_sub_dirs: Vec<String>,
    pub popular: bool,
}

impl Game {
    pub fn subdirs(&self) -> impl Iterator<Item = &str> {
        self.mod_loader
            .subdirs()
            .chain(self.extra_sub_dirs.iter().map(String::as_str))
    }
}

impl ModLoader {
    fn subdirs(&self) -> impl Iterator<Item = &str> {
        match self {
            ModLoader::BepInEx => {
                const SUBDIRS: [&str; 5] = ["plugins", "config", "patchers", "monomod", "core"];
                SUBDIRS.into_iter()
            }
        }
    }
}

impl From<GameData> for Game {
    fn from(value: GameData) -> Self {
        let GameData {
            name,
            slug,
            popular,
            mod_loader,
            extra_sub_dirs,
            platforms,
        } = value;

        let slug = slug.unwrap_or_else(|| name.to_kebab_case());

        let (steam_id, steam_name) = match platforms.steam {
            Steam::Concise(id) => (id, name.clone()),
            Steam::Full { id, dir_name } => (id, dir_name),
        };

        Self {
            name,
            slug,
            steam_name,
            steam_id,
            mod_loader,
            r2_dir_name: Default::default(),
            extra_sub_dirs,
            popular,
        }
    }
}

impl PartialEq for Game {
    fn eq(&self, other: &Self) -> bool {
        self.slug == other.slug
    }
}

impl Hash for Game {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.slug.hash(state);
    }
}

pub fn from_slug(slug: &str) -> Option<&'static Game> {
    GAMES.iter().find(|game| game.slug == slug)
}

const JSON: &str = include_str!("../games.json");

lazy_static! {
    pub static ref GAMES: Vec<Game> = serde_json::from_str(JSON).unwrap();
}
