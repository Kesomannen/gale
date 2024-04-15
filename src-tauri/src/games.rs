use serde::Serialize;

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub id: String,
    pub display_name: String,
    pub url: String,
    pub steam_id: u32,
}

impl Game {
    fn new(id: &'static str, display_name: &'static str, steam_id: u32) -> Self {
        Self {
            id: id.to_owned(),
            display_name: display_name.to_owned(),
            url: format!("https://thunderstore.io/c/{}/api/v1/package/", id),
            steam_id,
        }
    }

    fn with_url(mut self, url: &'static str) -> Self {
        self.url = url.to_owned();
        self
    }
}

lazy_static! {
    pub static ref GAMES: [Game; 8] = [
        Game::new("lethal-company", "Lethal Company", 1966720),
        Game::new("ror2", "Risk of Rain 2", 632360).with_url("https://thunderstore.io/api/v1/package/"),
        Game::new("dyson-sphere-program", "Dyson Sphere Program", 1366540),
        Game::new("content-warning", "Content Warning", 2881650),
        Game::new("rounds", "Rounds", 1557740),
        Game::new("inscryption", "Inscryption", 1092790),
        Game::new("muck", "Muck", 1625450),
        Game::new("subnautica", "Subnautica", 264710),
    ];
}

pub fn from_name(name: &str) -> Option<&'static Game> {
    GAMES.iter().find(|g| g.id == name)
}