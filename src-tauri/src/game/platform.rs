use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, EnumIter};

#[derive(
    Serialize, Deserialize, Debug, Default, Clone, Copy, strum_macros::Display, EnumIter, AsRefStr,
)]
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
pub struct Steam {
    pub id: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EpicGames {
    #[serde(default)]
    pub identifier: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Oculus {}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Origin {}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct XboxStore {
    #[serde(default)]
    pub identifier: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Platforms {
    pub steam: Option<Steam>,
    pub epic_games: Option<EpicGames>,
    pub oculus: Option<Oculus>,
    pub origin: Option<Origin>,
    pub xbox_store: Option<XboxStore>,
}

impl Platforms {
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
