use std::process::Command;

use eyre::{Result, bail, ensure};

use super::settings::LocalServerSettings;
use crate::game::Game;

const MIN_VALHEIM_PASSWORD_LENGTH: usize = 5;

/// Applies the game's dedicated-server launch arguments to `command`.
///
/// This is the game-specific half of local launching: which settings map to
/// which arguments, and which validation rules apply. Games without a builder
/// here are not supported.
pub fn apply_game_args(
    command: &mut Command,
    game: Game,
    settings: &LocalServerSettings,
    password: &str,
) -> Result<()> {
    match &*game.slug {
        "valheim" => valheim_args(command, settings, password),
        slug => bail!("dedicated server launch is not supported for {slug}"),
    }
}

/// Validates `settings` the way the game's launch-arg builder will use them.
/// Runs the game-independent checks first.
pub fn validate_game_args(
    game: Game,
    settings: &LocalServerSettings,
    password: &str,
) -> Result<()> {
    settings.validate()?;

    match &*game.slug {
        "valheim" => validate_valheim(settings, password),
        slug => bail!("dedicated server launch is not supported for {slug}"),
    }
}

fn validate_valheim(settings: &LocalServerSettings, password: &str) -> Result<()> {
    ensure!(
        !settings.world.trim().is_empty(),
        "world name cannot be empty"
    );

    if !password.is_empty() {
        ensure!(
            password.chars().count() >= MIN_VALHEIM_PASSWORD_LENGTH,
            "server password must contain at least {MIN_VALHEIM_PASSWORD_LENGTH} characters"
        );

        ensure!(
            !settings
                .server_name
                .to_lowercase()
                .contains(&password.to_lowercase()),
            "server password cannot be contained in the server name"
        );
    }

    Ok(())
}

fn valheim_args(
    command: &mut Command,
    settings: &LocalServerSettings,
    password: &str,
) -> Result<()> {
    validate_valheim(settings, password)?;

    command
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

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game;

    fn settings() -> LocalServerSettings {
        LocalServerSettings {
            server_name: "My Server".into(),
            world: "Dedicated".into(),
            port: 2456,
            ..Default::default()
        }
    }

    #[test]
    fn valheim_args_cover_all_settings() {
        let game = game::from_slug("valheim").unwrap();
        let mut command = Command::new("valheim_server");
        let settings = LocalServerSettings {
            crossplay: true,
            ..settings()
        };

        apply_game_args(&mut command, game, &settings, "secret1").unwrap();

        let args: Vec<_> = command
            .get_args()
            .map(|arg| arg.to_str().unwrap().to_owned())
            .collect();
        assert_eq!(
            args,
            [
                "-nographics",
                "-batchmode",
                "-name",
                "My Server",
                "-port",
                "2456",
                "-world",
                "Dedicated",
                "-public",
                "0",
                "-password",
                "secret1",
                "-crossplay"
            ]
        );
    }

    #[test]
    fn valheim_rejects_invalid_settings() {
        let game = game::from_slug("valheim").unwrap();

        // Empty world.
        assert!(
            validate_game_args(
                game,
                &LocalServerSettings {
                    world: " ".into(),
                    ..settings()
                },
                ""
            )
            .is_err()
        );

        // Short password.
        assert!(validate_game_args(game, &settings(), "abc").is_err());

        // Password contained in server name (case-insensitive).
        let mut named = settings();
        named.server_name = "My Secret Server".into();
        assert!(validate_game_args(game, &named, "secret").is_err());

        // Valid.
        assert!(validate_game_args(game, &settings(), "secret1").is_ok());
    }

    #[test]
    fn rejects_games_without_a_builder() {
        let game = game::from_slug("h3vr").unwrap();

        assert!(validate_game_args(game, &settings(), "").is_err());
    }
}
