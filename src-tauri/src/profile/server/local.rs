use std::{
    path::PathBuf,
    process::{Child, Command},
};

use eyre::{Context, OptionExt, Result};
use tracing::info;

use crate::{
    game::platform::Platform,
    prefs::Prefs,
    profile::{ManagedGame, launch, server::config::DedicatedServerSettings},
};

pub struct LocalServerProcess {
    pub child: Child,
    pub server_dir: PathBuf,
    pub profile_id: i64,
    pub game_slug: String,
}

pub fn launch(
    game: &ManagedGame,
    settings: &DedicatedServerSettings,
    password: &str,
    prefs: &Prefs,
) -> Result<LocalServerProcess> {
    settings.validate_local(password)?;

    let dedicated = game
        .game
        .dedicated_server
        .as_ref()
        .ok_or_eyre("this game does not define a dedicated server")?;

    let (server_dir, server_platform) = locate_server_dir(game, prefs)?;
    let executable = launch::find_executable(&server_dir)
        .context("failed to locate dedicated server executable")?;

    game.copy_required_files(&server_dir)
        .context("failed to prepare mod loader files for dedicated server")?;

    let profile = game.active_profile();
    let mut command = Command::new(&executable);

    if matches!(server_platform, Platform::Steam)
        && let Some(steam) = &game.game.platforms.steam
    {
        command.env("SteamAppId", steam.id.to_string());
    }

    command
        .current_dir(&server_dir)
        .arg("-nographics")
        .arg("-batchmode")
        .arg("-name")
        .arg(settings.server_name.trim())
        .arg("-port")
        .arg(settings.port.to_string())
        .arg("-world")
        .arg(settings.world.trim())
        .arg("-public")
        .arg(if settings.public_server { "1" } else { "0" });

    if !password.is_empty() {
        command.arg("-password").arg(password);
    }

    if settings.crossplay {
        command.arg("-crossplay");
    }

    game.apply_mod_loader_args(
        &mut command,
        &server_dir,
        Some(server_platform),
        &dedicated.platforms,
    )
    .context("failed to configure mod loader")?;

    if !settings.extra_args.trim().is_empty() {
        launch::apply_custom_args(&mut command, &settings.extra_args)
            .context("failed to apply dedicated server launch arguments")?;
    }

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;

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

    let child = command
        .spawn()
        .context("failed to start dedicated server")?;

    Ok(LocalServerProcess {
        child,
        server_dir,
        profile_id: profile.id,
        game_slug: game.game.slug.to_string(),
    })
}

pub(crate) fn locate_server_dir(game: &ManagedGame, prefs: &Prefs) -> Result<(PathBuf, Platform)> {
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
