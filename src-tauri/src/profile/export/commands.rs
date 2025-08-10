use std::{
    fs,
    io::{BufWriter, Cursor},
    path::PathBuf,
};

use eyre::{anyhow, Context};
use gale_util::{cmd::Result, fs::PathExt};
use itertools::Itertools;
use tauri::{command, AppHandle};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tracing::{debug, warn};
use uuid::Uuid;

use super::{
    changelog,
    modpack::{self, ModpackArgs},
};
use crate::{
    profile::ProfileModKind,
    state::ManagerExt,
    thunderstore::{self},
};

#[command]
pub async fn export_code(app: AppHandle) -> Result<Uuid> {
    let key = super::export_code(&app).await?;

    Ok(key)
}

#[command]
pub fn export_file(dir: PathBuf, app: AppHandle) -> Result<()> {
    let manager = app.lock_manager();

    let game = manager.active_game().game;
    let profile = manager.active_profile();

    let mut path = dir;
    path.push(&profile.name);
    path.set_extension("r2z");

    let file = fs::File::create(&path).map_err(|err| anyhow!(err))?;
    let writer = BufWriter::new(file);
    super::export_zip(manager.active_profile(), writer, game)?;

    open::that(path.parent().unwrap()).ok();

    Ok(())
}

#[command]
pub fn get_pack_args(app: AppHandle) -> Result<Option<ModpackArgs>> {
    let mut manager = app.lock_manager();
    let profile = manager.active_profile_mut();

    modpack::refresh_args(profile);

    Ok(profile.modpack.clone())
}

#[command]
pub fn set_pack_args(args: ModpackArgs, app: AppHandle) -> Result<()> {
    let mut manager = app.lock_manager();

    let profile = manager.active_profile_mut();
    profile.modpack = Some(args);
    profile.save(&app, true)?;

    Ok(())
}

#[command]
pub fn export_pack(dir: PathBuf, args: ModpackArgs, app: AppHandle) -> Result<()> {
    let mut manager = app.lock_manager();
    let thunderstore = app.lock_thunderstore();

    let profile = manager.active_profile_mut();

    let mut path = dir;
    path.push(format!("{}-{}", args.name, args.version_number));
    path.add_ext("zip");

    debug!("exporting pack to {:?}", path);

    let file = fs::File::create(&path)
        .map(BufWriter::new)
        .context("failed to create file")?;
    profile.export_pack(&args, file, &thunderstore)?;

    debug!("taking snapshot of profile");

    if let Err(err) = profile.take_snapshot(&args) {
        warn!("failed to take profile snapshot: {}", err);
    }

    open::that(path).ok();

    Ok(())
}

#[command]
pub async fn upload_pack(args: ModpackArgs, app: AppHandle) -> Result<()> {
    let (data, game, args, token) = {
        let manager = app.lock_manager();
        let thunderstore = app.lock_thunderstore();

        let token = thunderstore::token::get()
            .context("failed to get thunderstore API token")?
            .ok_or(anyhow!("no thunderstore API token found"))?;

        let profile = manager.active_profile();

        let mut data = Cursor::new(Vec::new());
        profile.export_pack(&args, &mut data, &thunderstore)?;

        if let Err(err) = profile.take_snapshot(&args) {
            warn!("failed to take profile snapshot: {}", err);
        }

        (data, manager.active_game, args, token)
    };

    let client = app.http().clone();
    modpack::publish(data.into_inner().into(), game, args, token, client).await?;

    Ok(())
}

#[command]
pub fn copy_dependency_strings(app: AppHandle) -> Result<()> {
    let manager = app.lock_manager();

    let content = manager
        .active_profile()
        .mods
        .iter()
        .map(|profile_mod| profile_mod.ident())
        .join("\n");

    app.clipboard()
        .write_text(content)
        .context("failed to write to clipboard")?;

    Ok(())
}

#[command]
pub fn copy_debug_info(app: AppHandle) -> Result<()> {
    let manager = app.lock_manager();
    let profile = manager.active_profile();

    let log = profile
        .log_path()
        .and_then(|path| fs::read_to_string(path).map_err(|err| anyhow!(err)));

    let content = format!(
        "OS: {}\nGale version: {}\n\nMods ({}):\n{}\n\nLatest log:\n{}",
        std::env::consts::OS,
        env!("CARGO_PKG_VERSION"),
        profile.mods.len(),
        profile
            .mods
            .iter()
            .map(|profile_mod| {
                let ty = match &profile_mod.kind {
                    ProfileModKind::Thunderstore(_) => "thunderstore",
                    ProfileModKind::Local(_) => "local",
                };

                format!("{} [{}]", profile_mod.ident(), ty)
            })
            .join("\n"),
        match log {
            Ok(log) => log,
            Err(err) => format!("failed to read log: {err}"),
        }
    );

    app.clipboard()
        .write_text(content)
        .context("failed to write to clipboard")?;

    Ok(())
}

#[command]
pub fn generate_changelog(mut args: ModpackArgs, all: bool, app: AppHandle) -> Result<String> {
    let manager = app.lock_manager();
    let thunderstore = app.lock_thunderstore();

    if all {
        let changelog = changelog::generate_all(
            &args,
            manager.active_profile(),
            manager.active_game().game,
            &thunderstore,
        )?;

        Ok(changelog)
    } else {
        changelog::generate_latest(
            &mut args,
            manager.active_profile(),
            manager.active_game().game,
            &thunderstore,
        )?;

        Ok(args.changelog)
    }
}
