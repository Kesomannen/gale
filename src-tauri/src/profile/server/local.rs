use std::path::PathBuf;

use eyre::{Context, OptionExt, Result};
use tracing::info;

use super::{args, settings::LocalServerSettings};
use crate::{
    game::{Game, platform::Platform},
    prefs::Prefs,
    profile::{ManagedGame, Profile, launch},
};

pub struct LocalServerProcess {
    pub child: tokio::process::Child,
    pub server_dir: PathBuf,
    pub profile_id: i64,
    pub game: Game,
}

/// Launches a dedicated server on this machine for `profile`.
///
/// The caller is expected to have validated `settings` with
/// [`args::validate_game_args`] already; this only resolves the installation,
/// applies arguments and spawns the process.
pub fn launch(
    game: &ManagedGame,
    profile: &Profile,
    settings: &LocalServerSettings,
    password: &str,
    prefs: &Prefs,
) -> Result<LocalServerProcess> {
    if game.game.dedicated_server.is_none() {
        eyre::bail!("this game does not define a dedicated server");
    }

    let (server_dir, server_platform) = locate_server_dir(game, prefs)?;
    let executable = launch::find_executable(&server_dir)
        .context("failed to locate dedicated server executable")?;

    game.copy_required_files(&server_dir, &profile.path)
        .context("failed to prepare mod loader files for dedicated server")?;

    let mut command = std::process::Command::new(&executable);

    // The dedicated server executable runs outside Steam, so it needs the
    // app id in the environment for Steamworks to initialize.
    if matches!(server_platform, Platform::Steam)
        && let Some(steam) = &game.game.platforms.steam
    {
        command.env("SteamAppId", steam.id.to_string());
    }

    command.current_dir(&server_dir);

    args::apply_game_args(&mut command, game.game, settings, password)?;

    game.apply_mod_loader_args(
        &mut command,
        &server_dir,
        Some(server_platform),
        &profile.path,
    )
    .context("failed to configure mod loader")?;

    if !settings.extra_args.is_empty() {
        launch::custom_args::add_args(&mut command, &settings.extra_args)
            .context("failed to apply dedicated server launch arguments")?;
    }

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;

        // Give the server its own console window so its log stays visible
        // instead of dying silently with Gale.
        const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;

        command.creation_flags(CREATE_NEW_CONSOLE);
    }

    info!(
        game = %game.game.slug,
        profile = %profile.name,
        server_dir = %server_dir.display(),
        executable = %executable.display(),
        "launching dedicated server"
    );

    let child = tokio::process::Command::from(command)
        .spawn()
        .context("failed to start dedicated server")?;

    Ok(LocalServerProcess {
        child,
        server_dir,
        profile_id: profile.id,
        game: game.game,
    })
}

pub fn locate_server_dir(game: &ManagedGame, prefs: &Prefs) -> Result<(PathBuf, Platform)> {
    let dedicated = game
        .game
        .dedicated_server
        .as_ref()
        .ok_or_eyre("this game does not define a dedicated server")?;

    let preferred_platform = prefs
        .game_prefs
        .get(&*game.game.slug)
        .and_then(|prefs| prefs.platform)
        .filter(|platform| dedicated.platforms.has(*platform));

    let server_platform = preferred_platform
        .or_else(|| dedicated.platforms.iter().next())
        .ok_or_eyre("dedicated server has no configured platforms")?;

    let server_dir = launch::platform::locate_dir(
        Some(server_platform),
        &dedicated.platforms,
        &format!("{} Dedicated Server", game.game.name),
    )
    .context("failed to locate dedicated server installation")?;

    info!(
        platform = server_platform.as_ref(),
        server_dir = %server_dir.display(),
        "found dedicated server directory via platform"
    );

    Ok((server_dir, server_platform))
}
