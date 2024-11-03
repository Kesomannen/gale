use super::{
    changelog,
    modpack::{self, ModpackArgs},
};
use crate::{
    prefs::Prefs,
    profile::{ModManager, ProfileModKind},
    thunderstore::{self, Thunderstore},
    util::{
        cmd::{Result, StateMutex},
        fs::PathExt,
    },
    NetworkClient,
};
use anyhow::{anyhow, Context};
use itertools::Itertools;
use log::{debug, warn};
use std::{
    fs,
    io::{BufWriter, Cursor},
    path::PathBuf,
};
use tauri::{AppHandle, State};
use tauri_plugin_clipboard_manager::ClipboardExt;
use uuid::Uuid;

#[tauri::command]
pub async fn export_code(
    client: State<'_, NetworkClient>,
    manager: StateMutex<'_, ModManager>,
    thunderstore: StateMutex<'_, Thunderstore>,
) -> Result<Uuid> {
    let key = super::export_code(&client.0, manager, thunderstore).await?;

    Ok(key)
}

#[tauri::command]
pub fn export_file(
    dir: PathBuf,
    manager: StateMutex<'_, ModManager>,
    thunderstore: StateMutex<'_, Thunderstore>,
) -> Result<()> {
    let manager = manager.lock().unwrap();
    let thunderstore = thunderstore.lock().unwrap();

    let profile = manager.active_profile();

    let mut path = dir;
    path.push(&profile.name);
    path.set_extension("r2z");

    let file = fs::File::create(&path).map_err(|err| anyhow!(err))?;
    let writer = BufWriter::new(file);
    super::export_zip(manager.active_profile(), writer, &thunderstore)?;

    open::that(path.parent().unwrap()).ok();

    Ok(())
}

#[tauri::command]
pub fn get_pack_args(manager: StateMutex<'_, ModManager>) -> Result<Option<ModpackArgs>> {
    let mut manager = manager.lock().unwrap();
    let profile = manager.active_profile_mut();

    modpack::refresh_args(profile);

    Ok(profile.modpack.clone())
}

#[tauri::command]
pub fn set_pack_args(
    args: ModpackArgs,
    manager: StateMutex<'_, ModManager>,
    prefs: StateMutex<'_, Prefs>,
) -> Result<()> {
    let mut manager = manager.lock().unwrap();
    let prefs = prefs.lock().unwrap();

    manager.active_profile_mut().modpack = Some(args);
    manager.save(&prefs)?;

    Ok(())
}

#[tauri::command]
pub fn export_pack(
    dir: PathBuf,
    args: ModpackArgs,
    manager: StateMutex<'_, ModManager>,
    thunderstore: StateMutex<'_, Thunderstore>,
) -> Result<()> {
    let mut manager = manager.lock().unwrap();
    let thunderstore = thunderstore.lock().unwrap();

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

#[tauri::command]
pub async fn upload_pack(
    args: ModpackArgs,
    manager: StateMutex<'_, ModManager>,
    thunderstore: StateMutex<'_, Thunderstore>,
    client: tauri::State<'_, NetworkClient>,
) -> Result<()> {
    let (data, game, args, token) = {
        let manager = manager.lock().unwrap();
        let thunderstore = thunderstore.lock().unwrap();

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

    let client = client.0.clone();
    modpack::publish(data.into_inner().into(), game, args, token, client).await?;

    Ok(())
}

#[tauri::command]
pub fn copy_dependency_strings(app: AppHandle, manager: StateMutex<ModManager>) -> Result<()> {
    let manager = manager.lock().unwrap();

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

#[tauri::command]
pub fn copy_debug_info(app: AppHandle, manager: StateMutex<ModManager>) -> Result<()> {
    let manager = manager.lock().unwrap();
    let profile = manager.active_profile();

    let log = profile
        .bepinex_log_path()
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
            Err(err) => format!("failed to read log: {}", err),
        }
    );

    app.clipboard()
        .write_text(content)
        .context("failed to write to clipboard")?;

    Ok(())
}

#[tauri::command]
pub fn generate_changelog(
    mut args: ModpackArgs,
    all: bool,
    manager: StateMutex<ModManager>,
    thunderstore: StateMutex<Thunderstore>,
) -> Result<String> {
    let manager = manager.lock().unwrap();
    let thunderstore = thunderstore.lock().unwrap();

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
