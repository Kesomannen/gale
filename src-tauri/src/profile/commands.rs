use eyre::{Context, OptionExt};
use itertools::Itertools;
use log::warn;
use serde::Serialize;
use tauri::{command, AppHandle};
use uuid::Uuid;

use super::{actions::ActionResult, Dependant, Profile};
use crate::{
    game::{self, Game, Platform},
    state::ManagerExt,
    thunderstore::{query::QueryModsArgs, FrontendProfileMod, Thunderstore, VersionIdent},
    util::cmd::Result,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontendGame {
    name: &'static str,
    slug: &'static str,
    popular: bool,
    mod_loader: &'static str,
    platforms: Vec<Platform>,
}

impl From<Game> for FrontendGame {
    fn from(value: Game) -> Self {
        let platforms = value.platforms.iter().collect();

        Self {
            name: value.name,
            slug: &*value.slug,
            popular: value.popular,
            mod_loader: value.mod_loader.to_str(),
            platforms,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameInfo {
    all: Vec<FrontendGame>,
    active: FrontendGame,
    favorites: Vec<&'static str>,
}

#[command]
pub fn get_game_info(app: AppHandle) -> GameInfo {
    let manager = app.lock_manager();

    let favorites = manager
        .games
        .iter()
        .filter_map(|(game, managed_game)| match managed_game.favorite {
            true => Some(&*game.slug),
            false => None,
        })
        .collect();

    GameInfo {
        all: game::all().map_into().collect(),
        active: manager.active_game.into(),
        favorites,
    }
}

#[command]
pub fn favorite_game(slug: String, app: AppHandle) -> Result<()> {
    let prefs = app.lock_prefs();
    let mut manager = app.lock_manager();

    let game = game::from_slug(&slug).ok_or_eyre("unknown game")?;
    let managed_game = manager.ensure_game(game, &prefs, app.db())?;
    managed_game.favorite = !managed_game.favorite;

    managed_game.save(app.db())?;

    Ok(())
}

#[command]
pub fn set_active_game(slug: &str, app: AppHandle) -> Result<()> {
    let mut manager = app.lock_manager();

    let game = game::from_slug(slug).ok_or_eyre("unknown game")?;

    manager.set_active_game(game, &app)?;
    manager.save(app.db())?;

    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilesInfo {
    profiles: Vec<ProfileInfo>,
    active_id: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileInfo {
    id: i64,
    name: String,
    mod_count: usize,
}

#[command]
pub fn get_profile_info(app: AppHandle) -> ProfilesInfo {
    let manager = app.lock_manager();
    let game = manager.active_game();

    ProfilesInfo {
        profiles: game
            .profiles
            .iter()
            .map(|profile| ProfileInfo {
                id: profile.id,
                name: profile.name.clone(),
                mod_count: profile.mods.len(),
            })
            .collect(),
        active_id: game.active_profile_id,
    }
}

#[command]
pub fn set_active_profile(index: usize, app: AppHandle) -> Result<()> {
    let mut manager = app.lock_manager();

    let game = manager.active_game_mut();
    game.set_active_profile(index)?;
    game.save(app.db())?;

    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontendAvailableUpdate {
    full_name: VersionIdent,
    ignore: bool,
    package_uuid: Uuid,
    version_uuid: Uuid,
    old: semver::Version,
    new: semver::Version,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileQuery {
    mods: Vec<FrontendProfileMod>,
    total_mod_count: usize,
    updates: Vec<FrontendAvailableUpdate>,
    unknown_mods: Vec<Dependant>,
}

#[command]
pub fn query_profile(args: QueryModsArgs, app: AppHandle) -> Result<ProfileQuery> {
    let manager = app.lock_manager();
    let thunderstore = app.lock_thunderstore();

    let profile = manager.active_profile();

    let (mods, unknown_mods) = profile.query_mods(&args, &thunderstore);
    let total_mod_count = profile.mods.len();

    let updates = profile
        .mods
        .iter()
        .filter_map(|profile_mod| {
            profile
                .check_update(profile_mod.uuid(), false, &thunderstore)
                .transpose()
        })
        .map_ok(|update| {
            let ignore = profile.ignored_updates.contains(&update.latest.uuid);

            FrontendAvailableUpdate {
                full_name: update.latest.ident.clone(),
                package_uuid: update.package.uuid,
                version_uuid: update.latest.uuid,
                old: update.current.parsed_version().clone(),
                new: update.latest.parsed_version().clone(),
                ignore,
            }
        })
        .collect::<eyre::Result<Vec<_>>>()
        .unwrap_or_else(|err| {
            warn!("failed to check for updates: {:#}", err);
            Vec::new()
        });

    Ok(ProfileQuery {
        mods,
        total_mod_count,
        updates,
        unknown_mods,
    })
}

#[command]
pub fn is_mod_installed(uuid: Uuid, app: AppHandle) -> Result<bool> {
    let manager = app.lock_manager();

    let result = manager.active_profile().has_mod(uuid);

    Ok(result)
}

#[command]
pub fn create_profile(name: String, app: AppHandle) -> Result<()> {
    let mut manager = app.lock_manager();

    manager.active_game_mut().create_profile(name, app.db())?;
    manager.save_all(app.db())?;

    Ok(())
}

#[command]
pub fn delete_profile(index: usize, app: AppHandle) -> Result<()> {
    let mut manager = app.lock_manager();

    let game = manager.active_game_mut();
    game.delete_profile(index, false)?;
    game.save(app.db())?;

    Ok(())
}

#[command]
pub fn rename_profile(name: String, app: AppHandle) -> Result<()> {
    let mut manager = app.lock_manager();

    let profile = manager.active_profile_mut();
    profile.rename(name)?;
    profile.save(app.db())?;

    Ok(())
}

#[command]
pub fn duplicate_profile(name: String, app: AppHandle) -> Result<()> {
    let mut manager = app.lock_manager();

    let game = manager.active_game_mut();
    game.duplicate_profile(name, game.active_profile_id, app.db())?;
    manager.save_all(app.db())?;

    Ok(())
}

#[command]
pub fn remove_mod(uuid: Uuid, app: AppHandle) -> Result<ActionResult> {
    mod_action_command(app, |profile, thunderstore| {
        profile.remove_mod(uuid, thunderstore)
    })
}

#[command]
pub fn toggle_mod(uuid: Uuid, app: AppHandle) -> Result<ActionResult> {
    mod_action_command(app, |profile, thunderstore| {
        profile.toggle_mod(uuid, thunderstore)
    })
}

fn mod_action_command<F>(app: AppHandle, action: F) -> Result<ActionResult>
where
    F: FnOnce(&mut Profile, &Thunderstore) -> eyre::Result<ActionResult>,
{
    let mut manager = app.lock_manager();
    let thunderstore = app.lock_thunderstore();

    let profile = manager.active_profile_mut();
    let response = action(profile, &thunderstore)?;

    if let ActionResult::Done = response {
        profile.save(app.db())?;
    }

    Ok(response)
}

#[command]
pub fn force_remove_mods(uuids: Vec<Uuid>, app: AppHandle) -> Result<()> {
    let mut manager = app.lock_manager();

    let profile = manager.active_profile_mut();
    for package_uuid in uuids {
        profile.force_remove_mod(package_uuid)?;
    }

    profile.save(app.db())?;

    Ok(())
}

#[command]
pub fn set_all_mods_state(enable: bool, app: AppHandle) -> Result<usize> {
    let mut manager = app.lock_manager();

    let profile = manager.active_profile_mut();
    let uuids = profile
        .mods
        .iter()
        .filter(|profile_mod| profile_mod.enabled != enable)
        .map(|profile_mod| profile_mod.uuid())
        .collect_vec();

    let count = uuids.len();

    for uuid in uuids {
        profile.force_toggle_mod(uuid)?;
    }

    profile.save(app.db())?;

    Ok(count)
}

#[command]
pub fn remove_disabled_mods(app: AppHandle) -> Result<usize> {
    let mut manager = app.lock_manager();

    let profile = manager.active_profile_mut();
    let uuids = profile
        .mods
        .iter()
        .filter(|profile_mod| !profile_mod.enabled)
        .map(|profile_mod| profile_mod.uuid())
        .collect_vec();

    let len = uuids.len();

    for uuid in uuids {
        profile.force_remove_mod(uuid)?;
    }

    profile.save(app.db())?;

    Ok(len)
}

#[command]
pub fn force_toggle_mods(uuids: Vec<Uuid>, app: AppHandle) -> Result<()> {
    let mut manager = app.lock_manager();

    let profile = manager.active_profile_mut();
    for package_uuid in uuids {
        profile.force_toggle_mod(package_uuid)?;
    }

    profile.save(app.db())?;

    Ok(())
}

#[command]
pub fn get_dependants(uuid: Uuid, app: AppHandle) -> Result<Vec<VersionIdent>> {
    let manager = app.lock_manager();
    let thunderstore = app.lock_thunderstore();

    let dependants = manager
        .active_profile()
        .dependants(uuid, &thunderstore)
        .map(|profile_mod| profile_mod.ident().into_owned())
        .collect();

    Ok(dependants)
}

#[command]
pub fn open_profile_dir(app: AppHandle) -> Result<()> {
    let manager = app.lock_manager();

    let path = &manager.active_profile().path;
    open::that(path).context("failed to open directory")?;

    Ok(())
}

#[command]
pub fn open_mod_dir(uuid: Uuid, app: AppHandle) -> Result<()> {
    let manager = app.lock_manager();

    manager.active_profile().open_mod_dir(uuid)?;

    Ok(())
}

#[command]
pub fn open_game_log(app: AppHandle) -> Result<()> {
    let manager = app.lock_manager();

    let path = manager.active_profile().log_path()?;
    open::that_detached(path).context("failed to open log file")?;

    Ok(())
}
