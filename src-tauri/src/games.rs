use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub name: &'static str,
    pub display_name: &'static str,
    pub steam_id: u32,
}

impl Game {
    pub const fn new(name: &'static str, display_name: &'static str, steam_id: u32) -> Self {
        Self {
            name,
            display_name,
            steam_id,
        }
    }
}

pub const GAMES: [Game; 8] = [
    Game::new("RiskOfRain2", "Risk of Rain 2", 632360),
    Game::new("DysonSphereProgram", "Dyson Sphere Program", 1366540),
    Game::new("LethalCompany", "Lethal Company", 1966720),
    Game::new("ContentWarning", "Content Warning", 2881650),
    Game::new("Rounds", "Rounds", 1557740),
    Game::new("Inscryption", "Inscryption", 1092790),
    Game::new("Muck", "Muck", 1625450),
    Game::new("Subnautica", "Subnautica", 264710),
];

pub fn from_name(name: &str) -> Option<&'static Game> {
    GAMES.iter().find(|g| g.name == name)
}

pub fn from_steam_id(steam_id: u32) -> Option<&'static Game> {
    GAMES.iter().find(|g| g.steam_id == steam_id)
}