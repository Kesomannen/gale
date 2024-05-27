use anyhow::{Context, Result};
use tauri::{AppHandle, Manager};

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

use super::ImportData;
use crate::{
    fs_util, games,
    manager::{exporter::R2Mod, ModManager},
    prefs::Prefs,
    thunderstore::Thunderstore,
    util::IoResultExt,
};
use log::{debug, info};

lazy_static! {
    static ref DIR_NAME_TO_ID: HashMap<&'static str, &'static str> =
        HashMap::from([("LethalCompany", "lethal-company"),]);
}

pub async fn import_r2modman(path: &Path, app: &AppHandle) -> Result<()> {
    info!("importing r2modman profiles from {}", path.display());

    for game_dir in fs_util::read_dir(path)
        .fs_context("reading data directory", path)?
        .filter(|entry| entry.file_type().unwrap().is_dir())
    {
        let mut path = game_dir.path();
        let game_name = fs_util::file_name(&path);
        let id = match DIR_NAME_TO_ID.get(game_name.as_str()) {
            Some(id) => *id,
            None => {
                debug!("skipping unknown directory {}", game_name);
                continue;
            }
        };

        let game = games::from_name(id).context("invalid game id")?;

        {
            let manager = app.state::<Mutex<ModManager>>();
            let prefs = app.state::<Mutex<Prefs>>();

            let mut manager = manager.lock().unwrap();
            let prefs = prefs.lock().unwrap();

            manager.ensure_game(game, &prefs)?;
            manager.active_game = game;
        }

        path.push("profiles");

        for profile_dir in
            fs_util::read_dir(&path).fs_context("reading profiles directory", &path)?
        {
            let path = profile_dir.path();
            match import_profile(path.clone(), app)
                .await
                .with_context(|| format!("failed to import profile from {}", path.display()))?
            {
                true => info!("imported profile {}", path.display()),
                false => info!("skipped profile {}", path.display()),
            }
        }
    }

    Ok(())
}

async fn import_profile(mut path: PathBuf, app: &AppHandle) -> Result<bool> {
    let profile_name = fs_util::file_name(&path);

    path.push("mods.yml");

    if !path.exists() {
        return Ok(false);
    }

    let yaml = fs::read_to_string(&path).fs_context("reading mods.yml", &path)?;
    path.pop();

    let mods =
        serde_yaml::from_str::<Vec<R2Mod>>(&yaml).context("failed to parse parse mods.yml")?;

    if mods.is_empty() {
        return Ok(false);
    }

    let import_data = {
        let manager = app.state::<Mutex<ModManager>>();
        let thunderstore = app.state::<Mutex<Thunderstore>>();

        let mut manager = manager.lock().unwrap();
        let thunderstore = thunderstore.lock().unwrap();

        if let Some(index) = manager
            .active_game()
            .profiles
            .iter()
            .position(|p| p.name == profile_name)
        {
            info!("deleting existing profile {}", profile_name);

            manager
                .active_game_mut()
                .delete_profile(index, true)
                .context("failed to delete existing profile")?;
        }

        let mods = super::resolve_r2mods(mods.into_iter(), &thunderstore)?;

        ImportData {
            mods,
            name: profile_name.clone(),
            config_path: path.join("BepInEx").join("config"),
        }
    };

    super::import_data(import_data, app).await?;

    Ok(true)
}
