use std::{
    borrow::Cow,
    fs,
    hash::{self, Hash},
    sync::LazyLock,
};

use chrono::{DateTime, Utc};
use eyre::{OptionExt, Result};
use heck::{ToKebabCase, ToPascalCase};
use serde::{Deserialize, Serialize};

use mod_loader::ModLoader;
use platform::Platforms;
use tauri::AppHandle;
use tracing::{info, warn};

use crate::{
    state::ManagerExt,
    util::{self, fs::JsonStyle},
};

pub mod mod_loader;
pub mod platform;

pub const CACHE_FILE_NAME: &str = "games.json";

const GITHUB_API_URL: &str =
    "https://api.github.com/repos/Kesomannen/gale/commits?path=src-tauri/games.json&per_page=1";
const GAMES_JSON_URL: &str =
    "https://raw.githubusercontent.com/Kesomannen/gale/refs/heads/master/src-tauri/games.json";

const BUNDLED_GAMES_JSON: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", "games.json"));

const BUILD_TIME: &str = env!("BUILD_TIME");

static GAMES: LazyLock<(DateTime<Utc>, Vec<GameData<'static>>)> = LazyLock::new(|| {
    if cfg!(debug_assertions) {
        match get_cached_games() {
            Ok(cache) => {
                info!("using cached games list, last commit at {}", cache.date);
                return (cache.date, cache.games);
            }
            Err(err) => {
                warn!("failed to cached games list: {err}");
            }
        }
    }

    let updated_at = DateTime::parse_from_rfc3339(BUILD_TIME).unwrap().to_utc();

    info!("using bundled games list (built at {updated_at})");

    (
        updated_at,
        serde_json::from_str(BUNDLED_GAMES_JSON).unwrap(),
    )
});

#[derive(Debug, Serialize, Deserialize)]
struct GamesCache<'a> {
    date: DateTime<Utc>,
    #[serde(borrow)]
    games: Vec<GameData<'a>>,
}

fn get_cached_games() -> Result<GamesCache<'static>> {
    let path = util::path::default_app_data_dir().join(CACHE_FILE_NAME);

    let str = fs::read_to_string(path)?;
    let games = serde_json::from_str(str.leak())?;

    Ok(games)
}

pub async fn update_list_task(app: &AppHandle) -> Result<()> {
    let str = app
        .http()
        .get(GAMES_JSON_URL)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    let games: Vec<GameData<'_>> = serde_json::from_str(&str)?;

    let date = get_last_commit_date(app).await.unwrap_or_else(|err| {
        warn!("failed to get last commit date: {err}");
        Utc::now()
    });

    let cache = GamesCache {
        date: date.clone(),
        games,
    };

    let path = util::path::default_app_data_dir().join(CACHE_FILE_NAME);
    util::fs::write_json(path, &cache, JsonStyle::Pretty)?;

    info!("updated games list from github, last commit at {date}");

    Ok(())
}

async fn get_last_commit_date(app: &AppHandle) -> Result<DateTime<Utc>> {
    #[derive(Debug, Deserialize)]
    struct ResponseEntry {
        commit: Commit,
    }

    #[derive(Debug, Deserialize)]
    struct Commit {
        author: Author,
    }

    #[derive(Debug, Deserialize)]
    struct Author {
        date: DateTime<Utc>,
    }

    let response: Vec<ResponseEntry> = app
        .http()
        .get(GITHUB_API_URL)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let date = response
        .get(0)
        .ok_or_eyre("github api response contained no entries")?
        .commit
        .author
        .date;

    Ok(date)
}

pub type Game = &'static GameData<'static>;

pub fn list() -> impl Iterator<Item = Game> {
    GAMES.1.iter()
}

pub fn from_slug(slug: &str) -> Option<Game> {
    GAMES.1.iter().find(|game| game.slug == slug)
}

pub fn last_updated() -> DateTime<Utc> {
    return GAMES.0;
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
