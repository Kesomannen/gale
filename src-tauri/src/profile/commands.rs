use std::path::PathBuf;

use eyre::{Context, OptionExt};
use itertools::Itertools;
use serde::Serialize;
use tauri::{command, AppHandle};
use tracing::warn;
use uuid::Uuid;

use super::{actions::ActionResult, Dependant, Profile};
use crate::{
    game::{self, platform::Platform, Game},
    profile::FrontendManagedGame,
    state::ManagerExt,
    thunderstore::{
        cache::MarkdownKind, query::QueryModsArgs, FrontendProfileMod, Thunderstore, VersionIdent,
    },
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
            mod_loader: value.mod_loader.as_str(),
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
    let managed_game = manager.ensure_game(game, false, &prefs, app.db())?;
    managed_game.favorite = !managed_game.favorite;

    managed_game.save(&app)?;

    Ok(())
}

#[command]
pub fn set_active_game(slug: &str, app: AppHandle) -> Result<()> {
    let mut manager = app.lock_manager();

    let game = game::from_slug(slug).ok_or_eyre("unknown game")?;

    app.sync_socket().unsubscribe(manager.active_profile());

    let managed_game = manager.set_active_game(game, &app)?;

    app.sync_socket().subscribe(managed_game.active_profile());

    managed_game.update_window_title(&app)?;
    manager.save_all(&app)?;

    Ok(())
}

#[command]
pub fn get_profile_info(app: AppHandle) -> FrontendManagedGame {
    let manager = app.lock_manager();

    manager.active_game().to_frontend()
}

#[command]
pub async fn set_active_profile(index: usize, app: AppHandle) -> Result<()> {
    let mut manager = app.lock_manager();

    let game = manager.active_game_mut();

    app.sync_socket().unsubscribe(game.active_profile());

    game.set_active_profile(index)?;

    app.sync_socket().subscribe(game.active_profile());

    game.save(&app)?;
    game.update_window_title(&app)?;

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
    let install_queue = app.install_queue().handle();

    let profile = manager.active_profile();

    let (mods, unknown_mods) = profile.query_mods(&args, &thunderstore);
    let total_mod_count = profile.mods.len();

    let updates = profile
        .mods
        .iter()
        .filter_map(|profile_mod| {
            profile
                .check_update(profile_mod.uuid(), false, &thunderstore, &install_queue)
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
    let profile = manager.active_profile();

    let result = profile.has_mod(uuid) || app.install_queue().handle().has_mod(uuid, profile.id);

    Ok(result)
}

#[command]
pub fn create_profile(name: String, override_path: Option<PathBuf>, app: AppHandle) -> Result<()> {
    let mut manager = app.lock_manager();
    let game = manager.active_game_mut();

    let profile = game.create_profile(name, override_path, app.db())?;

    profile.save(&app, false)?;
    game.save(&app)?;

    game.update_window_title(&app)?;

    Ok(())
}

#[command]
pub fn delete_profile(index: usize, app: AppHandle) -> Result<()> {
    let mut manager = app.lock_manager();
    let game = manager.active_game_mut();

    game.delete_profile(index, false, app.db())?;
    game.save(&app)?;

    game.update_window_title(&app)?;

    Ok(())
}

#[command]
pub fn rename_profile(name: String, app: AppHandle) -> Result<()> {
    let mut manager = app.lock_manager();
    let game = manager.active_game_mut();

    let profile = game.active_profile_mut();

    profile.rename(name)?;
    profile.save(&app, true)?;

    game.update_window_title(&app)?;

    Ok(())
}

#[command]
pub fn duplicate_profile(name: String, app: AppHandle) -> Result<()> {
    let mut manager = app.lock_manager();
    let game = manager.active_game_mut();

    let profile = game.duplicate_profile(name, game.active_profile_id, app.db())?;

    profile.save(&app, false)?;
    game.save(&app)?;

    game.update_window_title(&app)?;

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
        profile.save(&app, true)?;
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

    profile.save(&app, true)?;

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

    profile.save(&app, true)?;

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

    profile.save(&app, true)?;

    Ok(len)
}

#[command]
pub fn force_toggle_mods(uuids: Vec<Uuid>, app: AppHandle) -> Result<()> {
    let mut manager = app.lock_manager();

    let profile = manager.active_profile_mut();
    for package_uuid in uuids {
        profile.force_toggle_mod(package_uuid)?;
    }

    profile.save(&app, true)?;

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

#[command]
pub fn create_desktop_shortcut(app: AppHandle) -> Result<()> {
    let manager = app.lock_manager();

    manager.active_game().create_desktop_shortcut()?;

    Ok(())
}

#[command]
pub async fn get_local_markdown(
    uuid: Uuid,
    kind: MarkdownKind,
    app: AppHandle,
) -> Result<Option<String>> {
    let manager = app.lock_manager();
    let profile = manager.active_profile();

    let local_mod = profile.get_mod(uuid).and_then(|profile_mod| {
        profile_mod
            .as_local()
            .map(|(local_mod, _)| local_mod)
            .ok_or_eyre("mod is not local")
    })?;

    let str = match kind {
        MarkdownKind::Readme => &local_mod.readme,
        MarkdownKind::Changelog => &local_mod.changelog,
    };

    Ok(str.clone())
}

#[command]
pub fn set_custom_args(custom_args: Vec<String>, enabled: bool, app: AppHandle) -> Result<()> {
    let mut manager = app.lock_manager();
    let profile = manager.active_profile_mut();
    profile.custom_args = custom_args;
    profile.custom_args_enabled = enabled;
    profile.save(&app, false)?;
    manager.save_active_game(&app)?;
    Ok(())
}
