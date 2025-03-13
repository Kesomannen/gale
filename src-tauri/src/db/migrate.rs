use std::{collections::HashSet, fs};

use eyre::Result;
use log::info;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    game,
    prefs::Prefs,
    profile::{export::modpack::ModpackArgs, ProfileMod},
    util,
};

// old types

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManagerSaveData {
    active_game: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManagedGameSaveData {
    favorite: bool,
    active_profile_index: usize,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProfileSaveData {
    mods: Vec<ProfileMod>,

    #[serde(default)]
    modpack: Option<ModpackArgs>,

    #[serde(default)]
    ignored_updates: HashSet<Uuid>,
}

pub fn should_migrate(prefs: &Prefs) -> bool {
    prefs.data_dir.join("manager.json").exists()
}

pub fn migrate(prefs: &Prefs) -> Result<super::SaveData> {
    info!("migrating save data from {}", prefs.data_dir.display());

    let manifest_path = prefs.data_dir.join("manager.json");
    let manager_data: ManagerSaveData = util::fs::read_json(&manifest_path)?;
    fs::rename(&manifest_path, manifest_path.with_extension("old"))?;

    let manager = super::ManagerData {
        id: 1,
        active_game_slug: Some(manager_data.active_game),
    };

    let mut games = Vec::new();
    let mut profiles = Vec::new();

    let game_dirs = prefs
        .data_dir
        .read_dir()?
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|ty| ty.is_dir()))
        .filter_map(|entry| {
            game::from_slug(&entry.file_name().to_string_lossy()).map(|game| (game, entry.path()))
        });

    for (game, path) in game_dirs {
        let data: ManagedGameSaveData = util::fs::read_json(path.join("game.json"))?;

        let mut active_profile_id: i64 = 1;

        let profile_dirs = path
            .join("profiles")
            .read_dir()?
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_ok_and(|ty| ty.is_dir()))
            .map(|entry| entry.path());

        for (index, path) in profile_dirs.enumerate() {
            let name = util::fs::file_name_owned(&path);
            let profile_data: ProfileSaveData = util::fs::read_json(path.join("profile.json"))?;

            let id = (profiles.len() + 1) as i64;

            profiles.push(super::ProfileData {
                id,
                name,
                path: path.to_string_lossy().into_owned(),
                game_slug: game.slug.to_string(),
                mods: profile_data.mods,
                modpack: profile_data.modpack,
                ignored_updates: Some(profile_data.ignored_updates),
            });

            if data.active_profile_index == index {
                active_profile_id = id;
            }
        }

        games.push(super::ManagedGameData {
            id: (games.len() + 1) as i64,
            slug: game.slug.to_string(),
            favorite: data.favorite,
            active_profile_id,
        });
    }

    Ok(super::SaveData {
        manager,
        games,
        profiles,
    })
}
