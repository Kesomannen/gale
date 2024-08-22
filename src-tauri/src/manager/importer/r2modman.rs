use anyhow::{anyhow, ensure, Context, Result};
use log::{debug, info, warn};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use std::{
    collections::HashMap,
    fs::{self},
    path::PathBuf,
    sync::Mutex,
    time::Duration,
};

use super::ImportData;
use crate::{
    logger,
    manager::{
        downloader::InstallOptions,
        exporter::{ImportSource, R2Mod},
        ModManager,
    },
    prefs::Prefs,
    thunderstore::Thunderstore,
    util::{
        self,
        error::IoResultExt,
        fs::{Overwrite, PathExt},
    },
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagerData<T> {
    r2modman: Option<T>,
    thunderstore: Option<T>,
}

impl<T> ManagerData<T> {
    pub fn and_then<U, F: FnOnce(T) -> Option<U> + Copy>(self, f: F) -> ManagerData<U> {
        ManagerData {
            r2modman: self.r2modman.and_then(f),
            thunderstore: self.thunderstore.and_then(f),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileImportData {
    path: PathBuf,
    profiles: Vec<String>,
}

pub fn gather_info(app: &AppHandle) -> ManagerData<ProfileImportData> {
    find_paths().and_then(|path| {
        let profiles = find_profiles(path.clone(), false, app)
            .ok()?
            .map(util::fs::file_name_lossy)
            .collect();
        Some(ProfileImportData { path, profiles })
    })
}

pub async fn import(path: PathBuf, include: &[bool], app: &AppHandle) -> Result<()> {
    wait_for_mods(app).await;

    info!("importing profiles from {}", path.display());

    for (i, profile_dir) in find_profiles(path, true, app)?.enumerate() {
        if !include[i] {
            continue;
        }

        let name = profile_dir.file_name().unwrap().to_string_lossy();

        let data = match prepare_import(profile_dir.clone(), app) {
            Ok(Some(data)) => data,
            Ok(None) => {
                continue;
            }
            Err(err) => {
                logger::log_js_err(
                    "Error while importing from r2modman",
                    &err.context(format!("Failed to prepare import of profile '{}'", name)),
                    app,
                );
                continue;
            }
        };

        if let Err(err) = import_profile(data, app).await {
            logger::log_js_err(
                "Error while importing from r2modman",
                &err.context(format!("Failed to import profile '{}'", name)),
                app,
            );

            let manager = app.state::<Mutex<ModManager>>();
            let mut manager = manager.lock().unwrap();

            let game = manager.active_game_mut();

            if let Some(index) = game.profile_index(&name) {
                game.delete_profile(index, true).unwrap_or_else(|_| {
                    warn!("failed to delete possibly corrupted profile '{}'", name)
                });
            }
        };
    }

    Ok(())
}

fn find_profiles(
    mut path: PathBuf,
    transfer_cache: bool,
    app: &AppHandle,
) -> Result<impl Iterator<Item = PathBuf>> {
    let manager = app.state::<Mutex<ModManager>>();
    let manager = manager.lock().unwrap();

    let dir_name = ID_TO_R2_DIR
        .get(manager.active_game.id.as_str())
        .ok_or_else(|| anyhow!("current game unsupported"))?;

    path.push(dir_name);

    if transfer_cache {
        if let Err(e) = import_cache(path.clone(), app) {
            logger::log_js_err("failed to transfer r2modman cache", &e, app);
        };
    }

    path.push("profiles");

    ensure!(path.exists(), "no profiles found");

    Ok(path
        .read_dir()
        .fs_context("reading profiles directory", &path)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().unwrap().is_dir())
        .map(|entry| entry.path()))
}

async fn import_profile(data: ImportData, app: &AppHandle) -> Result<()> {
    info!("importing profile '{}'", data.name);
    emit_update(&format!("Importing profile '{}'... 0%", data.name), app);

    let name = data.name.clone();

    super::import_data(
        data,
        InstallOptions::default()
            .can_cancel(false)
            .send_progress(false)
            .on_progress(move |progress, app| {
                let percentage = (progress.total_progress * 100.0).round();
                emit_update(
                    &format!("Importing profile '{}'... {}%", name, percentage),
                    app,
                );
            }),
        app,
    )
    .await
}

fn prepare_import(mut profile_dir: PathBuf, app: &AppHandle) -> Result<Option<ImportData>> {
    let manager = app.state::<Mutex<ModManager>>();
    let thunderstore = app.state::<Mutex<Thunderstore>>();

    let mut manager = manager.lock().unwrap();
    let thunderstore = thunderstore.lock().unwrap();

    let name = util::fs::file_name_lossy(&profile_dir);

    profile_dir.push("mods.yml");

    if !profile_dir.exists() {
        info!("no mods.yml in {}, skipping", profile_dir.display());
        return Ok(None);
    }
    let yaml = fs::read_to_string(&profile_dir).fs_context("reading mods.yml", &profile_dir)?;
    let mods = serde_yaml::from_str::<Vec<R2Mod>>(&yaml).context("failed to parse mods.yml")?;

    profile_dir.pop();

    if mods.is_empty() {
        info!("profile '{}' is empty, skipping", name);
        return Ok(None);
    }

    if let Some(index) = manager.active_game().profile_index(&name) {
        info!("deleting existing profile '{}'", name);

        manager
            .active_game_mut()
            .delete_profile(index, true)
            .context("failed to delete existing profile")?;
    }

    ImportData::from_r2_mods(
        name,
        mods,
        profile_dir,
        false,
        Vec::new(),
        ImportSource::R2,
        &thunderstore,
    )
    .map(Some)
}

async fn wait_for_mods(app: &AppHandle) {
    let thunderstore = app.state::<Mutex<Thunderstore>>();

    loop {
        {
            let thunderstore = thunderstore.lock().unwrap();
            if thunderstore.packages_fetched {
                return;
            }
        }

        emit_update("Fetching mods from Thunderstore...", app);

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

    for package in path.read_dir()? {
        let package = package?;

        if !package.file_type()?.is_dir() {
            continue;
        }

        fs::create_dir_all(prefs.cache_dir.join(package.file_name()))?;

        for version in package.path().read_dir()? {
            let version = version?;

            if !version.file_type()?.is_dir() {
                continue;
            }

            let package_name = util::fs::file_name_lossy(&package.path());
            let version_name = util::fs::file_name_lossy(&version.path());

            let new_path = prefs.cache_dir.join(&package_name).join(&version_name);
            if new_path.exists() {
                continue;
            }

            debug!("transferring cached mod: {}-{}", package_name, version_name);
            util::fs::copy_dir(&version.path(), &new_path, Overwrite::Yes)?;
        }
    }

    Ok(())
}

fn find_paths() -> ManagerData<PathBuf> {
    let parent_dir = match cfg!(target_os = "linux") {
        // r2modman uses the config dir instead of the data dir on linux.
        true => dirs_next::config_dir(),
        false => dirs_next::data_dir(),
    }
    .unwrap();

    ManagerData {
        r2modman: parent_dir.join("r2modmanPlus-local").exists_or_none(),
        thunderstore: parent_dir
            .join("Thunderstore Mod Manager")
            .join("DataFolder")
            .exists_or_none(),
    }
}

fn emit_update(message: &str, app: &AppHandle) {
    app.emit("transfer_update", message).ok();
}
