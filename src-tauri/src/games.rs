use heck::ToKebabCase;
use serde::Serialize;
use std::hash::{self, Hash};

#[derive(Serialize, Debug, Clone, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub id: String,
    pub display_name: String,
    pub steam_name: String,
    pub aliases: Vec<String>,
    pub url: String,
    pub steam_id: u32,
}

impl PartialEq for Game {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for Game {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

fn default_package_url(id: &str) -> String {
    format!("https://thunderstore.io/c/{}/api/v1/package/", id)
}

impl Game {
    fn new(display_name: &str, steam_id: u32) -> Self {
        let id = display_name.to_kebab_case();

        Self {
            url: default_package_url(&id),
            display_name: display_name.to_owned(),
            steam_name: display_name.to_owned(),
            aliases: Vec::new(),
            steam_id,
            id,
        }
    }

    fn url(mut self, url: &str) -> Self {
        url.clone_into(&mut self.url);
        self
    }

    fn id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.id);
        self.url = default_package_url(id);
        self
    }

    fn on_steam(mut self, name: &str) -> Self {
        name.clone_into(&mut self.steam_name);
        self
    }

    fn aka(mut self, alias: &str) -> Self {
        self.aliases.push(alias.to_owned());
        self
    }
}

lazy_static! {
    pub static ref GAMES: [Game; 88] = [
        Game::new("Risk of Rain 2", 632360)
            .id("ror2")
            .url("https://thunderstore.io/api/v1/package/")
            .aka("ror2"),
        Game::new("Dyson Sphere Program", 1366540).aka("dsp"),
        Game::new("Valheim", 892970),
        Game::new("GTFO", 493520).id("gtfo"),
        Game::new("Outward", 794260),
        Game::new("TaleSpire", 720620).id("talespire"),
        Game::new("H3VR", 450540).id("h3vr"),
        Game::new("ROUNDS", 1557740),
        Game::new("Mechanica", 1226990),
        Game::new("Muck", 1625450),
        Game::new("Lethal League Blaze", 553310)
            .on_steam("LLBlaze")
            .aka("llb"),
        Game::new("Timberborn", 1062090),
        Game::new("TABS", 508440)
            .on_steam("Totally Accurate Battle Simulator")
            .id("totally-accurate-battle-simulator"),
        Game::new("Nickelodeon All-Star Brawl", 1414850)
            .id("nasb")
            .aka("nasb"),
        Game::new("Inscryption", 1092790),
        Game::new("Starsand", 1380220),
        Game::new("Cats are Liquid - A Better Place", 1188080).id("cats-are-liquid"),
        Game::new("Potion Craft", 1210320),
        Game::new("Nearly Dead", 1268900),
        Game::new("AGAINST", 1584840)
            .id("against")
            .on_steam("AGAINST_steam"),
        Game::new("Rogue Tower", 1843760),
        Game::new("House of the Dying Sun", 283160)
            .id("hotds")
            .on_steam("DyingSun"),
        Game::new("For The King", 527230),
        Game::new("Subnautica", 264710),
        Game::new("Subnautica: Below Zero", 848450)
            .id("belowzero")
            .on_steam("SubnauticaZero"),
        Game::new("Core Keeper", 1621690),
        Game::new("Peglin", 1296610),
        Game::new("V Rising", 1604030).on_steam("VRising"),
        Game::new("20 Minutes Till Dawn", 1966900).on_steam("20MinuteTillDawn"),
        Game::new("Green Hell VR", 1782330),
        Game::new("VTOL VR", 667970),
        Game::new("Stacklands", 1948280),
        Game::new("Enter the Gungeon", 311690),
        Game::new("Ravenfield", 636480),
        Game::new("Aloft", 2051980).on_steam("Aloft Demo"),
        Game::new("Cult of the Lamb", 1313140),
        Game::new("Chrono Ark", 1188930),
        Game::new("BONELAB", 1592190),
        Game::new("Trombone Champ", 1059990).on_steam("TromboneChamp"),
        Game::new("Rogue : Genesia", 2067920)
            .id("rogue-genesia")
            .on_steam("Rogue Genesia"),
        Game::new("Across the Obelisk", 1385380),
        Game::new("ULTRAKILL", 1229490),
        Game::new("Ultimate Chicken Horse", 386940),
        Game::new("Atrio: The Dark Wild", 1125390)
            .id("atrio-the-dark-wild")
            .on_steam("Atrio The Dark Wild"),
        Game::new("Ancient Dungeon VR", 1125240),
        Game::new("RUMBLE", 890550),
        Game::new("Skul: The Hero Slayer", 1147560)
            .id("skul-the-hero-slayer")
            .on_steam("Skul"),
        Game::new("Sons Of The Forest", 1326470),
        Game::new("The Ouroboros King", 2096510),
        Game::new("Wrestling Empire", 1620340),
        Game::new("Receiver 2", 1129310),
        Game::new("The Planet Crafter", 1284190),
        Game::new("Patch Quest", 1347970),
        Game::new("Shadows Over Loathing", 1939160),
        Game::new("West of Loathing", 597220),
        Game::new("Sun Haven", 1432860),
        Game::new("Wildfrost", 1811990),
        Game::new("Shadows of Doubt", 986130),
        Game::new("Garfield Kart - Furious Racing", 1085510),
        Game::new("Techtonica", 1457320),
        Game::new("Thronefall", 2239150),
        Game::new("We Love Katamari REROLL+ Royal Reverie", 1730700)
            .id("we-love-katamari-reroll-royal-reverie")
            .on_steam("WLKRR"),
        Game::new("Wizard of Legend", 445980),
        Game::new("Bomb Rush Cyberfunk", 1353230).on_steam("BombRushCyberfunk"),
        Game::new("TouhouLostBranchOfLegend", 1140150).on_steam("LBoL"),
        Game::new("Wizard With A Gun", 1150530),
        Game::new("Sunkenland", 2080690),
        Game::new("Atomicrops", 757320),
        Game::new("Erenshor", 2382520),
        Game::new("Last Train Outta' Wormtown", 2318480).id("last-train-outta-wormtown"),
        Game::new("DREDGE", 1562430),
        Game::new("Cities: Skylines II", 949230)
            .id("cities-skylines-ii")
            .on_steam("Cities Skylines II"),
        Game::new("Lethal Company", 1966720).aka("lc"),
        Game::new("Meeple Station", 900010),
        Game::new("Void Crew", 1063420),
        Game::new("Sailwind", 1764530),
        Game::new("Plasma", 1409160),
        Game::new("Content Warning", 2881650).aka("cw"),
        Game::new("Bopl Battle", 1686940),
        Game::new("Vertigo 2", 843390),
        Game::new("Against the Storm", 1336490),
        Game::new("Lycans", 2596100),
        Game::new("Castle Story", 227860),
        Game::new("Panicore", 2695940),
        Game::new("Magicraft", 2103140),
        Game::new("Another Crab's Treasure", 1887840)
            .id("another-crabs-treasure")
            .on_steam("AnotherCrabsTreasure"),
        Game::new("Gladio Mori", 2908480).on_steam("Gladio Mori Demo"),
        Game::new("Slipstream: Rogue Space", 2765860)
            .id("slipstream-rogue-space")
            .on_steam("Slipstream Rogue Space"),
    ];
}

pub fn from_id(id: &str) -> Option<&'static Game> {
    GAMES.iter().find(|game| game.id == id)
}
