use std::{
    borrow::Cow,
    fs,
    hash::{self, Hash},
    sync::LazyLock,
};

use eyre::Result;
use heck::{ToKebabCase, ToPascalCase};
use serde::{Deserialize, Serialize};

use mod_loader::ModLoader;
use platform::Platforms;
use tracing::{info, warn};

use crate::util::{self, fs::JsonStyle};

pub mod mod_loader;
pub mod platform;

pub const CACHE_FILE_NAME: &str = "games.json";
const URL: &str =
    "https://raw.githubusercontent.com/Kesomannen/gale/refs/heads/master/src-tauri/games.json";
const FALLBACK_JSON: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", "games.json"));

static GAMES: LazyLock<Vec<GameData<'static>>> = LazyLock::new(|| {
    match (get_remote_games(), get_cached_games()) {
        (Ok(remote), _) => {
            info!(count = remote.len(), "got new games from remote");

            write_cache(&remote).unwrap_or_else(|err| {
                warn!("failed to write games cache: {err}");
            });

            remote
        }
        (Err(err), Ok(cached)) => {
            warn!("failed to get new games from remote, using cache: {err}");

            cached
        }
        (Err(remote_err), Err(cache_err)) => {
            warn!("failed to get new games from remote ({remote_err}) or cache ({cache_err}), using fallback");

            serde_json::from_str(FALLBACK_JSON).unwrap()
        }
    }
});

fn get_cached_games() -> Result<Vec<GameData<'static>>> {
    let path = util::path::default_app_data_dir().join(CACHE_FILE_NAME);
    let str = fs::read_to_string(path)?;
    let games = serde_json::from_str(str.leak())?;

    Ok(games)
}

fn write_cache(games: &Vec<GameData<'static>>) -> Result<()> {
    let path = util::path::default_app_data_dir().join(CACHE_FILE_NAME);
    util::fs::write_json(path, games, JsonStyle::Pretty)
}

fn get_remote_games() -> Result<Vec<GameData<'static>>> {
    let str = reqwest::blocking::get(URL)?.text()?;
    let games = serde_json::from_str(str.leak())?;

    Ok(games)
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
