use std::{
    borrow::Cow,
    fs,
    hash::{self, Hash},
    sync::{Arc, LazyLock},
};

use eyre::Result;
use heck::{ToKebabCase, ToPascalCase};
use serde::Deserialize;

use mod_loader::JsonModLoader;
use platform::Platforms;
use tracing::{info, warn};

use crate::util::{self};

pub mod mod_loader;
pub mod platform;

pub const CACHE_FILE_NAME: &str = "games.json";
const URL: &str =
    "https://raw.githubusercontent.com/Kesomannen/gale/refs/heads/master/src-tauri/games.json";
const FALLBACK_JSON: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", "games.json"));

static GAMES: LazyLock<Vec<GameData>> = LazyLock::new(|| {
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

fn get_cached_games() -> Result<Vec<GameData>> {
    let path = util::path::default_app_data_dir().join(CACHE_FILE_NAME);
    let str = fs::read_to_string(path)?;
    let games = serde_json::from_str(str.leak())?;

    Ok(games)
}

fn write_cache(games: &Vec<GameData>) -> Result<()> {
    let path = util::path::default_app_data_dir().join(CACHE_FILE_NAME);
    // util::fs::write_json(path, games, JsonStyle::Pretty)
    Ok(())
}

fn get_remote_games() -> Result<Vec<GameData>> {
    let str = reqwest::blocking::get(URL)?.text()?;
    let games = serde_json::from_str(str.leak())?;

    Ok(games)
}

pub type Game = &'static GameData;

pub fn all() -> impl Iterator<Item = Game> {
    GAMES.iter()
}

pub fn from_slug(slug: &str) -> Option<Game> {
    GAMES.iter().find(|game| game.slug == slug)
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct JsonGame {
    name: String,

    #[serde(default)]
    slug: Option<String>,

    #[serde(default)]
    popular: bool,

    #[serde(default)]
    server: bool,

    #[serde(default, rename = "r2dirName")]
    r2_dir_name: Option<String>,

    mod_loader: JsonModLoader,

    #[serde(default)]
    platforms: Platforms,
}

#[derive(Deserialize)]
#[serde(from = "JsonGame")]
pub struct GameData {
    pub name: String,
    pub slug: String,
    pub r2_dir_name: String,
    pub popular: bool,
    pub server: bool,
    pub mod_loader: ModLoader,
    pub platforms: Platforms,
}

impl From<JsonGame> for GameData {
    fn from(value: JsonGame) -> Self {
        let JsonGame {
            name,
            slug,
            popular,
            server,
            r2_dir_name,
            mod_loader,
            platforms,
        } = value;

        let slug = slug.unwrap_or_else(|| name.to_kebab_case());
        let r2_dir_name = r2_dir_name.unwrap_or_else(|| slug.to_pascal_case());

        let mod_loader = mod_loader.into_loadsmith();

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

impl PartialEq for GameData {
    fn eq(&self, other: &Self) -> bool {
        self.slug == other.slug
    }
}

impl Eq for GameData {}

impl Hash for GameData {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.slug.hash(state);
    }
}

pub struct ModLoader {
    name_matcher: PackageNameMatcher,
    inner: Arc<dyn loadsmith::ModLoader>,
}

impl ModLoader {
    pub fn inner(&self) -> &dyn loadsmith::ModLoader {
        &*self.inner
    }

    pub fn installer_for(&self, package_name: &str) -> &dyn loadsmith::PackageInstaller {
        if self.name_matcher.matches(package_name) {
            self.inner.loader_installer()
        } else {
            self.inner.package_installer()
        }
    }
}

enum PackageNameMatcher {
    Exact(Cow<'static, str>),
    StartsWith(Cow<'static, str>),
}

impl PackageNameMatcher {
    pub fn matches(&self, package_name: &str) -> bool {
        match self {
            PackageNameMatcher::Exact(str) => package_name == &*str,
            PackageNameMatcher::StartsWith(str) => package_name.starts_with(&**str),
        }
    }
}
