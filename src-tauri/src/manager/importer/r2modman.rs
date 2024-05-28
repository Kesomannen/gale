use anyhow::{anyhow, ensure, Context, Result};
use log::{debug, info};
use tauri::{AppHandle, Manager};

use std::{
    collections::HashMap,
    fs::{self},
    path::PathBuf,
    sync::Mutex,
    time::Duration,
};

use super::ImportData;
use crate::{
    fs_util,
    manager::{downloader, exporter::R2Mod, ModManager},
    prefs::Prefs,
    thunderstore::Thunderstore,
    util::{self, IoResultExt},
};

lazy_static! {
    static ref ID_TO_R2_DIR: HashMap<&'static str, &'static str> = HashMap::from([
        ("ror2", "RiskOfRain2"),
        ("dsp", "DysonSphereProgram"),
        ("valheim", "Valheim"),
        ("gtfo", "GTFO"),
        ("outward", "Outward"),
        ("talespire", "TaleSpire"),
        ("h3vr", "H3VR"),
        ("rounds", "ROUNDS"),
        ("mechanica", "Mechanica"),
        ("muck", "Muck"),
        ("boneworks", "BONEWORKS"),
        ("lethal-league-blaze", "LethalLeagueBlaze"),
        ("timberborn", "Timberborn"),
        ("totally-accurate-battle-simulator", "TABS"),
        ("nasb", "NASB"),
        ("inscryption", "Inscryption"),
        ("starsand", "Starsand"),
        ("cats-are-liquid", "CatsAreLiquidABP"),
        ("potion-craft", "PotionCraft"),
        ("nearly-dead", "NearlyDead"),
        ("against", "AGAINST"),
        ("rogue-tower", "RogueTower"),
        ("hotds", "HOTDS"),
        ("for-the-king", "ForTheKing"),
        ("subnautica", "Subnautica"),
        ("belowzero", "SubnauticaBZ"),
        ("core-keeper", "CoreKeeper"),
        ("northstar", "Titanfall2"),
        ("peglin", "Peglin"),
        ("v-rising", "VRising"),
        ("hard-bullet", "HardBullet"),
        ("20-minutes-till-dawn", "20MinutesTillDawn"),
        ("green-hell-vr", "GreenHellVR"),
        ("vtol-vr", "VTOL_VR"),
        ("backpack-hero", "BackpackHero"),
        ("stacklands", "Stacklands"),
        ("enter-the-gungeon", "ETG"),
        ("ravenfield", "Ravenfield"),
        ("aloft", "Aloft"),
        ("cult-of-the-lamb", "COTL"),
        ("chrono-ark", "ChronoArk"),
        ("bonelab", "BONELAB"),
        ("trombone-champ", "TromboneChamp"),
        ("rogue-genesia", "RogueGenesia"),
        ("across-the-obelisk", "AcrossTheObelisk"),
        ("ultrakill", "ULTRAKILL"),
        ("ultimate-chicken-horse", "UltimateChickenHorse"),
        ("atrio-the-dark-wild", "AtrioTheDarkWild"),
        ("brotato", "Brotato"),
        ("ancient-dungeon-vr", "AncientDungeonVR"),
        ("rumble", "RUMBLE"),
        ("dome-keeper", "DomeKeeper"),
        ("skul-the-hero-slayer", "SkulTheHeroSlayer"),
        ("sons-of-the-forest", "SonsOfTheForest"),
        ("the-ouroboros-king", "TheOuroborosKing"),
        ("wrestling-empire", "WrestlingEmpire"),
        ("receiver-2", "Receiver2"),
        ("the-planet-crafter", "ThePlanetCrafter"),
        ("patch-quest", "PatchQuest"),
        ("shadows-over-loathing", "ShadowsOverLoathing"),
        ("west-of-loathing", "WestofLoathing"),
        ("sun-haven", "SunHaven"),
        ("wildfrost", "Wildfrost"),
        ("shadows-of-doubt", "ShadowsofDoubt"),
        ("garfield-kart-furious-racing", "GarfieldKartFuriousRacing"),
        ("techtonica", "Techtonica"),
        ("thronefall", "Thronefall"),
        (
            "we-love-katamari-reroll-royal-reverie",
            "WeLoveKatamariRerollRoyalReverie"
        ),
        ("wizard-of-legend", "WizardOfLegend"),
        ("bomb-rush-cyberfunk", "BombRushCyberfunk"),
        ("touhou-lost-branch-of-legend", "TouhouLostBranchOfLegend"),
        ("wizard-with-a-gun", "WizardWithAGun"),
        ("sunkenland", "Sunkenland"),
        ("atomicrops", "Atomicrops"),
        ("erenshor", "Erenshor"),
        ("last-train-outta-wormtown", "LastTrainOuttaWormtown"),
        ("dredge", "Dredge"),
        ("cities-skylines-ii", "CitiesSkylines2"),
        ("lethal-company", "LethalCompany"),
        ("meeple-station", "MeepleStation"),
        ("void-crew", "VoidCrew"),
        ("sailwind", "Sailwind"),
        ("voices-of-the-void", "VotV"),
        ("palworld", "Palworld"),
        ("plasma", "Plasma"),
        ("content-warning", "ContentWarning"),
    ]);
}

pub async fn import(app: &AppHandle) -> Result<()> {
    wait_for_mods(app).await;

    let path = find_path()?;

    info!("importing r2modman profiles from {}", path.display());

    for profile_dir in find_profiles(path, app)? {
        let name = profile_dir.file_name().unwrap();

        if let Err(err) = import_profile(profile_dir.clone(), app).await {
            util::print_err(
                "Error while importing from r2modman",
                &err.context(format!("Failed to import profile {:?}", name)),
                app,
            );
        };
    }

    Ok(())
}

fn find_profiles(mut path: PathBuf, app: &AppHandle) -> Result<impl Iterator<Item = PathBuf>> {
    let manager = app.state::<Mutex<ModManager>>();
    let manager = manager.lock().unwrap();

    let dir_name = ID_TO_R2_DIR
        .get(manager.active_game.id.as_str())
        .ok_or_else(|| anyhow!("current game unsupported"))?;

    path.push(dir_name);

    if let Err(e) = import_cache(path.clone(), app) {
        util::print_err("failed to transfer r2modman cache", &e, app);
    };

    path.push("profiles");

    ensure!(path.exists(), "no profiles found");

    Ok(fs_util::read_dir(&path)
        .fs_context("reading profiles directory", &path)?
        .filter(|entry| entry.file_type().unwrap().is_dir())
        .map(|entry| entry.path()))
}

async fn import_profile(path: PathBuf, app: &AppHandle) -> Result<bool> {
    let data = match read_data(path, app)? {
        Some(data) => data,
        None => return Ok(false),
    };

    info!("importing profile '{}'", data.name);
    emit_update(&format!("Importing profile '{}'...", data.name), app);
    super::import_data(data, false, app).await?;

    Ok(true)
}

fn read_data(mut path: PathBuf, app: &AppHandle) -> Result<Option<ImportData>> {
    let manager = app.state::<Mutex<ModManager>>();
    let thunderstore = app.state::<Mutex<Thunderstore>>();

    let mut manager = manager.lock().unwrap();
    let thunderstore = thunderstore.lock().unwrap();

    let name = fs_util::file_name(&path);

    if !path.exists() {
        info!("no mods.yml in {}, skipping", path.display());
        return Ok(None);
    }

    path.push("mods.yml");
    let yaml = fs::read_to_string(&path).fs_context("reading mods.yml", &path)?;
    let mods = serde_yaml::from_str::<Vec<R2Mod>>(&yaml).context("failed to parse mods.yml")?;
    path.pop();

    if mods.is_empty() {
        info!("profile '{}' is empty, skipping", name);
        return Ok(None);
    }

    if let Some(index) = manager
        .active_game()
        .profiles
        .iter()
        .position(|p| p.name == name)
    {
        info!("deleting existing profile '{}'", name);

        manager
            .active_game_mut()
            .delete_profile(index, true)
            .context("failed to delete existing profile")?;
    }

    let mods = super::resolve_r2mods(mods.into_iter(), &thunderstore)?;

    Ok(Some(ImportData {
        mods,
        name,
        config_path: path.join("BepInEx").join("config"),
    }))
}

async fn wait_for_mods(app: &AppHandle) {
    let thunderstore = app.state::<Mutex<Thunderstore>>();

    loop {
        {
            let thunderstore = thunderstore.lock().unwrap();
            if thunderstore.finished_loading {
                return;
            }
        }

        emit_update("Waiting for mod fecthing...", app);

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

fn import_cache(mut path: PathBuf, app: &AppHandle) -> Result<()> {
    path.push("cache");

    if !path.exists() {
        debug!("no cache directory found at {}", path.display());
        return Ok(());
    }

    emit_update("Transferring cached mods...", app);

    let prefs = app.state::<Mutex<Prefs>>();
    let prefs = prefs.lock().unwrap();

    let cache_dir = prefs.get_path_or_err("cache_dir")?;

    for package in fs_util::read_dir(&path)
        .fs_context("reading cache directory", &path)?
        .filter(|entry| entry.file_type().unwrap().is_dir())
    {
        fs::create_dir_all(cache_dir.join(package.file_name()))?;

        for version in fs_util::read_dir(&package.path())
            .fs_context("reading cache directory", &path)?
            .filter(|entry| entry.file_type().unwrap().is_dir())
        {
            let package_name = package.file_name().into_string().unwrap();
            let version_name = version.file_name().into_string().unwrap();

            let mut new_path = cache_dir.join(&package_name).join(&version_name);
            if new_path.exists() {
                continue;
            }

            debug!("transferring cached mod: {}-{}", package_name, version_name);
            fs_util::copy_dir(&version.path(), &new_path)?;
            downloader::normalize_mod_structure(&mut new_path)?;
        }
    }

    Ok(())
}

fn find_path() -> Result<PathBuf> {
    let mut path = tauri::api::path::data_dir().unwrap();

    path.push("r2modmanPlus-local");

    if path.exists() {
        return Ok(path);
    }

    path.pop();
    path.push("Thunderstore Mod Manager");
    path.push("DataFolder");

    ensure!(
        path.exists(),
        "r2modman directory not found at {}",
        path.display()
    );

    Ok(path)
}

fn emit_update(message: &str, app: &AppHandle) {
    app.emit_all("transfer_update", message).ok();
}
