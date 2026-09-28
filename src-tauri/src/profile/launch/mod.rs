use core::str;
#[cfg(not(target_os = "macos"))]
use std::fs;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

use eyre::{Context, OptionExt, Result, bail, ensure, eyre};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tokio::time::Duration;
use tracing::{info, warn};
use walkdir::WalkDir;

use super::ManagedGame;
#[cfg(not(target_os = "macos"))]
use crate::util::{
    self,
    fs::{Overwrite, UseLinks},
};
use crate::{
    game::Game,
    logger::log_webview_err,
    prefs::{GamePrefs, Prefs},
};

mod custom_args;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
pub(crate) mod macos;
mod mod_loader;
mod platform;

pub mod commands;

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase", tag = "type", content = "content")]
pub enum LaunchMode {
    #[default]
    #[serde(alias = "steam")]
    Launcher,
    #[serde(rename_all = "camelCase")]
    Direct { instances: u32, interval_secs: f32 },
}

impl LaunchMode {
    fn instances(&self) -> u32 {
        match self {
            LaunchMode::Launcher => 1,
            LaunchMode::Direct { instances, .. } => *instances,
        }
    }

    fn interval(&self) -> Duration {
        match self {
            LaunchMode::Launcher => Duration::from_secs_f32(0.0),
            LaunchMode::Direct { interval_secs, .. } => Duration::from_secs_f32(*interval_secs),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct LaunchOption {
    pub arguments: String,
    #[serde(rename = "type")]
    pub launch_type: Option<String>,
    pub description: Option<String>,
}

impl ManagedGame {
    pub fn launch(&self, vanilla: bool, prefs: &Prefs, app: &AppHandle) -> Result<()> {
        self.launch_with_args(vanilla, None, prefs, app)
    }

    pub fn launch_with_args(
        &self,
        vanilla: bool,
        args: Option<String>,
        prefs: &Prefs,
        app: &AppHandle,
    ) -> Result<()> {
        let game_dir =
            locate_game_dir(self.game, prefs).context("failed to locate game directory")?;

        // the launch command has already run `macos::prepare_launch` (launcher,
        // doorstop download, signature check) without holding any lock; this
        // repeats it synchronously for the CLI, which launches while holding
        // the prefs and manager locks, and is a cheap no-op after the command
        #[cfg(target_os = "macos")]
        if !vanilla && self.uses_bepinex() {
            macos::ensure_ready_blocking(self, &game_dir, prefs, app)?;
        }

        if let Err(err) = self.copy_required_files(&game_dir) {
            warn!("failed to copy required files to game directory: {:#}", err);
        }

        let (launch_mode, mut command) = self.launch_command(vanilla, &game_dir, prefs)?;

        if let Some(args) = args {
            command.args(args.split_whitespace());
        }

        info!(game = %self.game.slug, ?command, "launching");

        do_launch(command, app, launch_mode)?;

        Ok(())
    }

    fn launch_command(
        &self,
        vanilla: bool,
        game_dir: &Path,
        prefs: &Prefs,
    ) -> Result<(LaunchMode, Command)> {
        let (launch_mode, mut platform, game_custom_args) =
            prefs.game_prefs.get(&*self.game.slug).map_or_else(
                || {
                    info!("game prefs not set, using default settings");
                    Default::default()
                },
                |prefs| {
                    (
                        prefs.launch_mode.clone(),
                        prefs.platform,
                        prefs.custom_args.as_str(),
                    )
                },
            );

        // if the game has a platform but the setting is unset, fill it in
        platform = platform.or_else(|| self.game.platforms.iter().next());

        let launcher_command = match (&launch_mode, platform) {
            // If the setting is `Launcher` and we have a platform, use the platform-specific
            // launch command (if there is one). Otherwise, fall back to direct execution.
            (LaunchMode::Launcher, Some(platform)) => {
                platform::create_launch_command(game_dir, platform, self.game, prefs).transpose()
            }
            _ => None,
        };

        #[cfg(target_os = "macos")]
        let via_launcher = launcher_command.is_some();

        let mut command =
            launcher_command.unwrap_or_else(|| self.direct_command(vanilla, game_dir))?;

        let profile = self.active_profile();

        // On macOS a launcher (Steam) runs the game through the profile's
        // run_bepinex.sh if the user's launch options say so, no matter who
        // started it, and that script injects doorstop unless told otherwise.
        // Direct vanilla launches never go through the script.
        #[cfg(target_os = "macos")]
        if vanilla && via_launcher && self.uses_bepinex() {
            disable_doorstop_args(&mut command);
        }

        if !vanilla {
            #[cfg(target_os = "linux")]
            let is_proton = {
                use crate::game::platform::Platform;
                use tracing::warn;

                let is_proton = linux::is_proton(game_dir).unwrap_or_else(|err| {
                    warn!("failed to determine if game uses proton: {:#}", err);
                    false
                });

                if is_proton && let Some(proxy_dll) = self.game.mod_loader.proxy_dll() {
                    command.env("WINEDLLOVERRIDES", format!("{proxy_dll}=n,b"));

                    if let Some(steam) = &self.game.platforms.steam
                        && matches!(platform, Some(Platform::Steam))
                        && let Err(err) = linux::ensure_wine_override(steam.id, proxy_dll, game_dir)
                    {
                        warn!("failed to ensure wine dll override: {:#}", err);
                    }
                }

                is_proton
            };

            #[cfg(any(target_os = "windows", target_os = "macos"))]
            let is_proton = false;

            if is_proton {
                info!("game appears to be running under proton, using proton launch method");
            }

            let mut ctx = mod_loader::ArgsContext::new(&mut command, &profile.path, is_proton);
            ctx.add_args(&self.game.mod_loader)?;
        }

        custom_args::add_args(&mut command, game_custom_args)?;
        custom_args::add_args(&mut command, &profile.custom_args)?;

        if matches!(launch_mode, LaunchMode::Direct { .. }) {
            command.current_dir(game_dir);
        }

        Ok((launch_mode, command))
    }

    /// The command for running the game's executable directly, without a launcher.
    #[cfg(not(target_os = "macos"))]
    fn direct_command(&self, _vanilla: bool, game_dir: &Path) -> Result<Command> {
        find_executable(game_dir).map(Command::new)
    }

    /// The command for running the game's executable directly, without a launcher.
    ///
    /// For a modded BepInEx game this runs the profile's launcher script with the
    /// executable as its first argument, since that is what injects doorstop on
    /// macOS. Native games ship as app bundles, so the bundle's executable is
    /// preferred over any script the game directory happens to contain (such as
    /// a BepInExPack start script left there by an older version of Gale or by
    /// another mod manager), for vanilla and modded launches alike.
    #[cfg(target_os = "macos")]
    fn direct_command(&self, vanilla: bool, game_dir: &Path) -> Result<Command> {
        let executable = match find_app_bundle_executable(game_dir) {
            Some(executable) => executable,
            None => find_executable(game_dir)?,
        };

        if vanilla || !self.uses_bepinex() {
            return Ok(Command::new(executable));
        }

        // Run the launcher through the interpreter: macOS refuses to exec a shell
        // script directly from an app process (Steam hits the same restriction).
        let mut command = Command::new("/bin/sh");
        command
            .arg(macos::launcher_path(&self.active_profile().path))
            .arg(executable);

        Ok(command)
    }

    #[cfg(target_os = "macos")]
    fn uses_bepinex(&self) -> bool {
        use crate::game::mod_loader::ModLoaderKind;

        matches!(self.game.mod_loader.kind, ModLoaderKind::BepInEx { .. })
    }

    /// On macOS the launcher and doorstop live in the profile and nothing is
    /// copied into the game directory. Copying BepInExPack's files there is
    /// useless (its dylib is x86_64-only) and interferes with other launchers.
    #[cfg(target_os = "macos")]
    fn copy_required_files(&self, game_dir: &Path) -> Result<()> {
        info!(
            game_dir = %game_dir.display(),
            "not copying mod loader files into the game directory on macOS"
        );

        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    fn copy_required_files(&self, game_dir: &Path) -> Result<()> {
        const INCLUDE_DIRS: [&str; 2] = ["doorstop_libs", "dotnet"];
        const EXCLUDES: [&str; 2] = ["profile.json", "mods.yml"];

        let target_dir = game_dir.join(self.game.mod_loader.file_target.unwrap_or("."));
        ensure!(
            target_dir.exists(),
            "target directory for mod loader files does not exist at {}, please check your settings",
            target_dir.display()
        );

        let entries = self
            .active_profile()
            .path
            .read_dir()?
            .filter_map(std::result::Result::ok)
            .filter(|entry| {
                let name = entry.file_name();

                if EXCLUDES.iter().any(|exclude| name == *exclude) {
                    return false;
                }

                let is_file = entry.file_type().is_ok_and(|ty| ty.is_file());
                let is_included_dir = INCLUDE_DIRS.iter().any(|dir| *dir == name);

                is_file || is_included_dir
            });

        for entry in entries {
            let to_path = target_dir.join(entry.file_name());
            info!(
                file_name = %entry.file_name().to_string_lossy(),
                to_path = %to_path.display(),
                "copying to game directory",
            );

            if entry.file_type()?.is_file() {
                fs::copy(entry.path(), to_path)?;
            } else {
                util::fs::copy_dir(entry.path(), to_path, Overwrite::Yes, UseLinks::No)?;
            }
        }

        Ok(())
    }
}

/// Tells the profile's `run_bepinex.sh` (doorstop 4 argument spelling, which is
/// what the script parses) to leave doorstop disabled. The script strips the
/// pair from the game's arguments; if the launch never reaches the script, the
/// game ignores them like any other doorstop argument.
#[cfg(target_os = "macos")]
fn disable_doorstop_args(command: &mut Command) {
    command.args(["--doorstop-enabled", "false"]);
}

fn do_launch(mut command: Command, app: &AppHandle, mode: LaunchMode) -> Result<()> {
    match mode.instances() {
        0 => bail!("instances must be greater than 0"),
        1 => {
            command.spawn()?;
        }
        instances => {
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                for i in 0..instances {
                    if let Err(err) = command.spawn() {
                        log_webview_err(
                            "Failed to launch game",
                            eyre!("Launch command {} failed: {}.", i, err),
                            &app,
                        );
                    }
                    tokio::time::sleep(mode.interval()).await;
                }
            });
        }
    }

    Ok(())
}

fn locate_game_dir(game: Game, prefs: &Prefs) -> Result<PathBuf> {
    let game_prefs = prefs.game_prefs.get(&*game.slug);

    let path = if let Some(GamePrefs {
        dir_override: Some(path),
        ..
    }) = game_prefs
    {
        info!("using game directory override at {}", path.display());
        path.clone()
    } else {
        let platform = game_prefs
            .and_then(|prefs| prefs.platform)
            .or_else(|| game.platforms.iter().next());

        let path = platform::locate_game_dir(platform, game)?;
        info!(
            "found game directory via platform ({}): {}",
            match &platform {
                Some(platform) => platform.as_ref(),
                None => "none",
            },
            path.display()
        );
        path
    };

    ensure!(
        path.exists(),
        "game directory does not exist, please check your settings (expected at {})",
        path.display()
    );

    Ok(path)
}

const IGNORED_EXES: &[&str] = &[
    "crashpad_handler.exe",
    "UnityCrashHandler32.exe",
    "UnityCrashHandler64.exe",
];

fn find_executable(game_dir: &Path) -> Result<PathBuf> {
    let found = WalkDir::new(game_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .sorted_by(|a, b| a.depth().cmp(&b.depth())) // prefer shallower entries
        .find(|entry| {
            let file_name = PathBuf::from(entry.file_name());
            let file_name_str = file_name.to_string_lossy();
            let extension = file_name.extension().and_then(|ext| ext.to_str());

            let has_correct_extension = if cfg!(windows) {
                matches!(extension, Some("exe"))
            } else {
                matches!(extension, Some("exe" | "sh"))
            };

            has_correct_extension && !IGNORED_EXES.contains(&&*file_name_str)
        })
        .map(walkdir::DirEntry::into_path);

    // native macOS games ship as an app bundle, whose executable has no extension.
    // only fall back to it, since a mod loader's launch script (e.g. BepInEx's
    // run_bepinex.sh) must take precedence to inject the doorstop
    #[cfg(target_os = "macos")]
    let found = found.or_else(|| find_app_bundle_executable(game_dir));

    found.ok_or_eyre("game executable not found")
}

/// Resolves `<bundle>.app/Contents/MacOS/<executable>` for the first app bundle
/// at the top level of `game_dir`. Without parsing Info.plist, the executable is
/// the file named after the bundle (`Valheim.app` -> `Valheim`, case-insensitive),
/// or else the first one sorted by name.
#[cfg(target_os = "macos")]
fn find_app_bundle_executable(game_dir: &Path) -> Option<PathBuf> {
    game_dir
        .read_dir()
        .ok()?
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|ty| ty.is_dir()))
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("app"))
        })
        .sorted()
        .find_map(|bundle| {
            let stem = bundle.file_stem()?;

            let candidates = bundle
                .join("Contents")
                .join("MacOS")
                .read_dir()
                .ok()?
                .filter_map(Result::ok)
                .filter(|entry| entry.file_type().is_ok_and(|ty| ty.is_file()))
                .map(|entry| entry.path())
                .sorted()
                .collect_vec();

            candidates
                .iter()
                .find(|path| {
                    path.file_name()
                        .is_some_and(|name| name.eq_ignore_ascii_case(stem))
                })
                .or_else(|| candidates.first())
                .cloned()
        })
}

pub fn parse_steam_launch_options(steam_id: u32) -> Result<Vec<LaunchOption>> {
    let raw_options = platform::get_steam_launch_options(steam_id)
        .context("failed to get Steam launch options")?;

    let mut launch_options = Vec::new();

    if let Some(options_obj) = raw_options.as_object() {
        for (_, option_value) in options_obj {
            if let Some(option) = option_value.as_object() {
                // TODO: Figure out how to properly filter by active beta branch.
                // Need to find where Steam stores info about which beta branch is active for an app.
                if let Some(config) = option.get("config")
                    && config.get("BetaKey").is_some()
                {
                    continue;
                }

                let launch_type = option
                    .get("type")
                    .and_then(|t| t.as_str())
                    .map(std::string::ToString::to_string);

                let arguments = option
                    .get("arguments")
                    .and_then(|a| a.as_str())
                    .unwrap_or("")
                    .to_string();

                let description = option
                    .get("description")
                    .and_then(|d| d.as_str())
                    .map(std::string::ToString::to_string);

                launch_options.push(LaunchOption {
                    arguments,
                    launch_type,
                    description,
                });
            }
        }
    }

    Ok(launch_options)
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use std::fs;

    use tempfile::{TempDir, tempdir};

    use super::*;

    fn touch(path: PathBuf) -> PathBuf {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, b"").unwrap();
        path
    }

    fn game_dir() -> TempDir {
        tempdir().unwrap()
    }

    #[test]
    fn app_bundle_single_executable() {
        let dir = game_dir();
        let exe = touch(dir.path().join("Foo.app/Contents/MacOS/Foo"));

        assert_eq!(find_executable(dir.path()).unwrap(), exe);
    }

    #[test]
    fn app_bundle_prefers_executable_named_after_bundle() {
        let dir = game_dir();
        // sorts before "valheim" but is not the main executable
        touch(dir.path().join("Valheim.app/Contents/MacOS/UnityPlayer"));
        let exe = touch(dir.path().join("Valheim.app/Contents/MacOS/valheim"));

        assert_eq!(find_executable(dir.path()).unwrap(), exe);
    }

    #[test]
    fn app_bundle_falls_back_to_first_sorted() {
        let dir = game_dir();
        let exe = touch(dir.path().join("Foo.app/Contents/MacOS/a_game"));
        touch(dir.path().join("Foo.app/Contents/MacOS/b_helper"));

        assert_eq!(find_executable(dir.path()).unwrap(), exe);
    }

    #[test]
    fn app_bundle_skips_directory_named_like_executable() {
        let dir = game_dir();
        fs::create_dir_all(dir.path().join("Foo.app/Contents/MacOS/Foo")).unwrap();
        let exe = touch(dir.path().join("Foo.app/Contents/MacOS/Bar"));

        assert_eq!(find_executable(dir.path()).unwrap(), exe);
    }

    #[test]
    fn shell_script_at_top_level() {
        let dir = game_dir();
        let script = touch(dir.path().join("start_game.sh"));

        assert_eq!(find_executable(dir.path()).unwrap(), script);
    }

    #[test]
    fn shell_script_preferred_over_app_bundle() {
        let dir = game_dir();
        let script = touch(dir.path().join("run_bepinex.sh"));
        touch(dir.path().join("Foo.app/Contents/MacOS/Foo"));

        assert_eq!(find_executable(dir.path()).unwrap(), script);
    }

    #[test]
    fn nothing_found() {
        let dir = game_dir();
        touch(dir.path().join("readme.txt"));
        // a .app that is a plain file, not a bundle
        touch(dir.path().join("Foo.app"));

        assert!(find_executable(dir.path()).is_err());
    }

    fn managed_game(slug: &str, profile_dir: &Path) -> ManagedGame {
        use std::collections::{HashMap, HashSet};

        use crate::{config::ConfigCache, profile::Profile};

        let game = crate::game::from_slug(slug).unwrap();

        let profile = Profile {
            id: 1,
            name: "test".to_owned(),
            path: profile_dir.to_path_buf(),
            mods: Vec::new(),
            game,
            ignored_version_updates: HashSet::new(),
            ignored_package_updates: HashSet::new(),
            config_cache: ConfigCache::default(),
            linked_config: HashMap::new(),
            modpack: None,
            sync: None,
            custom_args: String::new(),
            missing: false,
        };

        ManagedGame {
            id: 1,
            game,
            path: profile_dir.parent().unwrap().to_path_buf(),
            profiles: vec![profile],
            favorite: false,
            active_profile_id: 1,
        }
    }

    #[test]
    fn direct_command_wraps_bundle_executable_with_launcher() {
        let dir = game_dir();
        let exe = touch(dir.path().join("Valheim.app/Contents/MacOS/Valheim"));
        // left behind in the game directory by an older version or another manager
        touch(dir.path().join("start_game_bepinex.sh"));
        let profile = tempdir().unwrap();
        let game = managed_game("valheim", profile.path());

        let command = game.direct_command(false, dir.path()).unwrap();

        assert_eq!(command.get_program(), "/bin/sh");
        assert_eq!(
            command.get_args().collect_vec(),
            vec![
                macos::launcher_path(profile.path()).as_os_str(),
                exe.as_os_str()
            ]
        );
    }

    #[test]
    fn direct_command_vanilla_runs_executable_itself() {
        let dir = game_dir();
        let exe = touch(dir.path().join("Valheim.app/Contents/MacOS/Valheim"));
        let profile = tempdir().unwrap();
        let game = managed_game("valheim", profile.path());

        let command = game.direct_command(true, dir.path()).unwrap();

        assert_eq!(command.get_program(), exe.as_os_str());
        assert_eq!(command.get_args().count(), 0);
    }

    #[test]
    fn direct_command_vanilla_ignores_stray_bepinex_script() {
        let dir = game_dir();
        let exe = touch(dir.path().join("Valheim.app/Contents/MacOS/Valheim"));
        // copied into the game directory by pre-macOS-port launches; running it
        // would inject BepInExPack's doorstop instead of starting the vanilla game
        touch(dir.path().join("start_game_bepinex.sh"));
        touch(dir.path().join("run_bepinex.sh"));
        let profile = tempdir().unwrap();
        let game = managed_game("valheim", profile.path());

        let command = game.direct_command(true, dir.path()).unwrap();

        assert_eq!(command.get_program(), exe.as_os_str());
        assert_eq!(command.get_args().count(), 0);
    }

    #[test]
    fn direct_command_vanilla_falls_back_to_script_without_bundle() {
        let dir = game_dir();
        let script = touch(dir.path().join("start_game.sh"));
        let profile = tempdir().unwrap();
        let game = managed_game("valheim", profile.path());

        let command = game.direct_command(true, dir.path()).unwrap();

        assert_eq!(command.get_program(), script.as_os_str());
    }

    fn prefs_with(slug: &str, launch_mode: LaunchMode) -> Prefs {
        let mut prefs = Prefs::default();
        prefs.game_prefs.insert(
            slug.to_owned(),
            GamePrefs {
                launch_mode,
                ..Default::default()
            },
        );
        prefs
    }

    #[test]
    fn disable_doorstop_args_uses_doorstop_4_spelling() {
        let mut command = Command::new("steam_osx");
        disable_doorstop_args(&mut command);

        assert_eq!(
            command.get_args().collect_vec(),
            vec!["--doorstop-enabled", "false"]
        );
    }

    #[test]
    fn launch_command_direct_vanilla_has_no_doorstop_args() {
        let dir = game_dir();
        let exe = touch(dir.path().join("Valheim.app/Contents/MacOS/Valheim"));
        let profile = tempdir().unwrap();
        let game = managed_game("valheim", profile.path());
        let prefs = prefs_with(
            "valheim",
            LaunchMode::Direct {
                instances: 1,
                interval_secs: 0.0,
            },
        );

        let (_, command) = game.launch_command(true, dir.path(), &prefs).unwrap();

        assert_eq!(command.get_program(), exe.as_os_str());
        assert_eq!(command.get_args().count(), 0);
    }

    #[test]
    fn launch_command_steam_vanilla_disables_doorstop() {
        // needs a Steam install to build the launcher command
        if platform::create_launch_command(
            Path::new("/"),
            crate::game::platform::Platform::Steam,
            crate::game::from_slug("valheim").unwrap(),
            &Prefs::default(),
        )
        .is_err()
        {
            eprintln!("skipping: Steam is not installed");
            return;
        }

        let dir = game_dir();
        touch(dir.path().join("Valheim.app/Contents/MacOS/Valheim"));
        let profile = tempdir().unwrap();
        let game = managed_game("valheim", profile.path());
        let prefs = prefs_with("valheim", LaunchMode::Launcher);

        let (_, command) = game.launch_command(true, dir.path(), &prefs).unwrap();

        let args = command.get_args().collect_vec();
        assert!(args.ends_with(&[
            std::ffi::OsStr::new("--doorstop-enabled"),
            std::ffi::OsStr::new("false")
        ]));

        // and a modded launch through the same launcher still enables it
        touch(profile.path().join("BepInEx/core/BepInEx.Preloader.dll"));
        fs::write(profile.path().join(".doorstop_version"), "4").unwrap();

        let (_, modded) = game.launch_command(false, dir.path(), &prefs).unwrap();

        let args = modded.get_args().collect_vec();
        assert!(args.contains(&std::ffi::OsStr::new("--doorstop-enabled")));
        assert!(args.contains(&std::ffi::OsStr::new("true")));
        assert!(!args.contains(&std::ffi::OsStr::new("false")));
    }
}
