use std::{
    path::{Path, PathBuf},
    process::Command,
};

use eyre::{Context, OptionExt, Result, ensure};
use tokio::process::Child;
use tracing::info;

use crate::{
    game::{Game, platform::Platform},
    prefs::Prefs,
    profile::{ManagedGame, launch, server::config::LocalServerSettings},
};

pub struct LocalServerProcess {
    pub child: Child,
    pub server_dir: PathBuf,
    pub profile_id: i64,
    pub game: Game,
}

pub fn launch(
    game: &ManagedGame,
    settings: &LocalServerSettings,
    password: &str,
    prefs: &Prefs,
) -> Result<LocalServerProcess> {
    settings.validate(password)?;

    game.game
        .dedicated_server
        .as_ref()
        .ok_or_eyre("this game does not define a dedicated server")?;

    ensure!(
        matches!(
            (game.game.slug.as_ref(), settings),
            ("valheim", LocalServerSettings::Valheim { .. })
        ),
        "dedicated server settings do not match the active game"
    );

    let (server_dir, server_platform) = locate_server_dir(game, prefs)?;
    let executable = launch::find_executable(&server_dir)
        .context("failed to locate dedicated server executable")?;

    let profile = game.active_profile();
    let mut command = Command::new(&executable);

    match settings {
        LocalServerSettings::Valheim {
            server_name,
            world,
            port,
            public_server,
            crossplay,
            ..
        } => {
            if matches!(server_platform, Platform::Steam)
                && let Some(steam) = &game.game.platforms.steam
            {
                // Valheim's dedicated server still checks the client app ID.
                command.env("SteamAppId", steam.id.to_string());
            }

            command
                .arg("-nographics")
                .arg("-batchmode")
                .arg("-name")
                .arg(server_name.trim())
                .arg("-port")
                .arg(port.to_string())
                .arg("-world")
                .arg(world.trim())
                .arg("-public")
                .arg(if *public_server { "1" } else { "0" });

            if !password.is_empty() {
                command.arg("-password").arg(password);
            }

            if *crossplay {
                command.arg("-crossplay");
            }
        }
    }

    command.current_dir(&server_dir);
    configure_mod_loader(game, &mut command, &server_dir, server_platform)?;

    launch::custom_args::add_args(&mut command, settings.extra_args())
        .context("failed to apply dedicated server launch arguments")?;

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

fn configure_mod_loader(
    game: &ManagedGame,
    command: &mut Command,
    server_dir: &Path,
    server_platform: Platform,
) -> Result<()> {
    if game
        .active_profile()
        .mods
        .iter()
        .any(|profile_mod| profile_mod.enabled)
    {
        game.copy_required_files(server_dir)
            .context("failed to prepare mod loader files for dedicated server")?;

        return game
            .apply_mod_loader_args(command, server_dir, Some(server_platform))
            .context("failed to configure mod loader");
    }

    info!("active profile has no enabled mods, launching server without mod loader");
    Ok(())
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

#[cfg(test)]
mod tests {
    use std::{
        collections::{HashMap, HashSet},
        fs,
        path::{Path, PathBuf},
        process::Command,
    };

    use chrono::Utc;
    use tempfile::tempdir;

    use super::configure_mod_loader;
    use crate::{
        config::ConfigCache,
        game::{self, platform::Platform},
        profile::{ManagedGame, Profile, ProfileMod, ProfileModKind},
    };

    fn profile_mod(enabled: bool) -> ProfileMod {
        ProfileMod {
            enabled,
            install_time: Utc::now(),
            kind: ProfileModKind::Local(Box::default()),
        }
    }

    fn managed_game(profile_dir: &Path, mods: Vec<ProfileMod>) -> ManagedGame {
        let game = game::from_slug("valheim").unwrap();

        ManagedGame {
            id: 1,
            game,
            path: PathBuf::new(),
            profiles: vec![Profile {
                id: 1,
                name: "Test".to_owned(),
                path: profile_dir.to_owned(),
                mods,
                game,
                ignored_version_updates: HashSet::new(),
                ignored_package_updates: HashSet::new(),
                config_cache: ConfigCache::default(),
                linked_config: HashMap::new(),
                modpack: None,
                sync: None,
                custom_args: String::new(),
                server_settings: None,
                missing: false,
                excluded_export_files: HashSet::new(),
            }],
            favorite: false,
            active_profile_id: 1,
        }
    }

    #[test]
    fn empty_profile_launches_without_mod_loader() {
        let profile = tempdir().unwrap();
        let server = tempdir().unwrap();
        let game = managed_game(profile.path(), Vec::new());
        let mut command = Command::new("server");

        configure_mod_loader(&game, &mut command, server.path(), Platform::Steam).unwrap();

        assert_eq!(command.get_args().count(), 0);
    }

    #[test]
    fn fully_disabled_profile_launches_without_mod_loader() {
        let profile = tempdir().unwrap();
        let server = tempdir().unwrap();
        let game = managed_game(profile.path(), vec![profile_mod(false)]);
        let mut command = Command::new("server");

        configure_mod_loader(&game, &mut command, server.path(), Platform::Steam).unwrap();

        assert_eq!(command.get_args().count(), 0);
    }

    #[test]
    fn enabled_profile_requires_mod_loader() {
        let profile = tempdir().unwrap();
        let server = tempdir().unwrap();
        let game = managed_game(profile.path(), vec![profile_mod(true)]);
        let mut command = Command::new("server");

        let error =
            configure_mod_loader(&game, &mut command, server.path(), Platform::Steam).unwrap_err();

        assert!(format!("{error:#}").contains("failed to read BepInEx core directory"));
    }

    #[test]
    fn enabled_profile_configures_mod_loader() {
        let profile = tempdir().unwrap();
        let server = tempdir().unwrap();
        let core = profile.path().join("BepInEx/core");
        fs::create_dir_all(&core).unwrap();
        fs::write(core.join("BepInEx.Unity.Mono.Preloader.dll"), []).unwrap();
        fs::write(profile.path().join(".doorstop_version"), "4.0.0").unwrap();
        let game = managed_game(profile.path(), vec![profile_mod(true)]);
        let mut command = Command::new("server");

        configure_mod_loader(&game, &mut command, server.path(), Platform::Steam).unwrap();

        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        assert_eq!(args[0], "--doorstop-enabled");
        assert_eq!(args[1], "true");
        assert_eq!(args[2], "--doorstop-target-assembly");
        assert!(args[3].ends_with("BepInEx.Unity.Mono.Preloader.dll"));
    }
}
