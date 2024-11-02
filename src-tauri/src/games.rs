use heck::ToKebabCase;
use serde::{Deserialize, Serialize};
use std::{
    convert::Infallible,
    hash::{self, Hash},
};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct GameData {
    name: String,
    #[serde(default)]
    slug: Option<String>,
    #[serde(default)]
    popular: bool,
    platforms: Platforms,
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
#[serde(rename_all = "camelCase", try_from = "GameData")]
pub struct Game {
    pub name: String,
    pub slug: String,
    pub steam_name: String,
    pub steam_id: u32,
    pub popular: bool,
}

impl TryFrom<GameData> for Game {
    type Error = Infallible;

    fn try_from(value: GameData) -> Result<Self, Self::Error> {
        let GameData {
            name,
            slug,
            popular,
            platforms,
        } = value;

        let slug = slug.unwrap_or_else(|| name.to_kebab_case());

        let (steam_id, steam_name) = match platforms.steam {
            Steam::Concise(id) => (id, name.clone()),
            Steam::Full { id, dir_name } => (id, dir_name),
        };

        Ok(Self {
            name,
            slug,
            steam_name,
            steam_id,
            popular,
        })
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
