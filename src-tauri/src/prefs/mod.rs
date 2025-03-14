use std::{
    collections::HashMap,
    fs,
    ops::Deref,
    path::{Path, PathBuf},
};

use eyre::{anyhow, bail, ensure, Context, Result};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::{
    game::{self, Platform},
    profile::launch::LaunchMode,
    state::ManagerExt,
    util::{
        self,
        error::IoResultExt,
        fs::{JsonStyle, Overwrite, PathExt, UseLinks},
        window::WindowExt,
    },
};

pub mod commands;

#[derive(Serialize, Deserialize, Clone, Debug, Eq)]
#[serde(transparent)]
pub struct DirPref {
    value: PathBuf,
    #[serde(skip)]
    keep_files: Vec<&'static str>,
}

impl DirPref {
    fn new(value: PathBuf) -> Self {
        Self {
            value,
            keep_files: Vec::new(),
        }
    }

    fn keep(mut self, file: &'static str) -> Self {
        self.keep_files.push(file);
        self
    }

    pub fn get(&self) -> &Path {
        &self.value
    }

    pub fn set(&mut self, new_value: PathBuf) -> Result<bool> {
        if self.value == new_value {
            return Ok(false);
        }

        ensure!(new_value.is_dir(), "new value is not a directory");
        ensure!(
            !new_value.starts_with(&self.value),
            "value cannot be a subdirectory of the current directory"
        );
        ensure!(
            new_value.read_dir()?.next().is_none(),
            "new directory is not empty"
        );

        info!(
            "attempting to rename directory: {} -> {}",
            self.value.display(),
            new_value.display()
        );

        // on windows fs::rename requires the target directory to not exist
        #[cfg(windows)]
        fs::remove_dir(&new_value)?;

        match fs::rename(&self.value, &new_value) {
            Ok(_) => {
                info!("renaming succeeded");

                if !self.keep_files.is_empty() {
                    // move files back to the original directory
                    fs::create_dir_all(&self.value)?;

                    for file in &self.keep_files {
                        let moved_path = new_value.join(file);
                        let original_path = self.value.join(file);

                        if !moved_path.exists() {
                            continue;
                        }

                        if moved_path.is_dir() {
                            util::fs::copy_dir(
                                &moved_path,
                                &original_path,
                                Overwrite::Yes,
                                UseLinks::No,
                            )?;
                        } else {
                            fs::copy(&moved_path, &original_path)?;
                        }
                    }
                }
            }
            Err(err) => {
                info!("renaming failed, falling back to copying: {}", err);

                fs::create_dir_all(&new_value).fs_context("creating new directory", &new_value)?;

                for entry in self
                    .value
                    .read_dir()
                    .fs_context("reading old directory", &self.value)?
                {
                    let entry = entry.context("failed to read file in old directory")?;
                    let file_name = entry.file_name();

                    if self.keep_files.iter().any(|file| file_name == *file) {
                        info!("skipping {}", file_name.to_string_lossy());
                        continue;
                    }

                    let old_path = entry.path();
                    let new_path = new_value.join(file_name);

                    if entry.file_type()?.is_dir() {
                        debug!("copying dir {:?} -> {:?}", old_path, new_path);

                        util::fs::copy_dir(&old_path, &new_path, Overwrite::Yes, UseLinks::No)
                            .context("failed to copy subdirectory")?;
                        fs::remove_dir_all(&old_path)
                            .fs_context("removing old subdirectory", &old_path)?;
                    } else {
                        debug!("copying file {:?} -> {:?}", old_path, new_path);

                        fs::copy(&old_path, &new_path).fs_context("copying file", &new_path)?;
                        fs::remove_file(&old_path).fs_context("removing old file", &old_path)?;
                    }
                }
            }
        }

        // remove only if empty
        fs::remove_dir(&self.value).ok();

        self.value = new_value;

        Ok(true)
    }
}

impl AsRef<Path> for DirPref {
    fn as_ref(&self) -> &Path {
        self.get()
    }
}

impl Deref for DirPref {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl PartialEq for DirPref {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl From<PathBuf> for DirPref {
    fn from(value: PathBuf) -> Self {
        Self::new(value)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default, rename_all = "camelCase")]
pub struct Prefs {
    #[serde(skip)]
    pub is_first_run: bool,

    pub steam_exe_path: Option<PathBuf>,
    pub steam_library_dir: Option<PathBuf>,
    pub data_dir: DirPref,

    #[serde(alias = "sendTelementary")] // old typo (oops)
    pub send_telemetry: bool,
    pub fetch_mods_automatically: bool,
    pub zoom_factor: f32,

    pub game_prefs: HashMap<String, GamePrefs>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct GamePrefs {
    pub dir_override: Option<PathBuf>,
    pub custom_args: Option<Vec<String>>,
    pub launch_mode: LaunchMode,
    pub platform: Option<Platform>,
}

#[cfg(target_os = "windows")]
fn read_steam_registry() -> Result<PathBuf> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hklm.open_subkey(r"SOFTWARE\WOW6432Node\Valve\Steam")?;

    let path: String = key.get_value("InstallPath")?;
    Ok(PathBuf::from(path))
}

fn default_steam_exe_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        match read_steam_registry() {
            Ok(path) => path.join("Steam.exe"),
            _ => r"C:\Program Files (x86)\Steam\Steam.exe".into(),
        }
    }

    #[cfg(target_os = "linux")]
    {
        "/usr/bin/steam".into()
    }
}

fn default_steam_library_dir(exe_path: Option<&Path>) -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        exe_path.and_then(|exe| exe.parent().map(|path| path.to_path_buf()))
    }

    #[cfg(target_os = "linux")]
    {
        dirs_next::data_dir().map(|data_dir| data_dir.join("Steam"))
    }
}

impl Default for Prefs {
    fn default() -> Self {
        let steam_exe_path = default_steam_exe_path().exists_or_none();

        let steam_library_dir = default_steam_library_dir(steam_exe_path.as_deref())
            .and_then(|path| path.exists_or_none());

        Self {
            is_first_run: false,

            steam_exe_path,
            steam_library_dir,
            data_dir: DirPref::new(util::path::default_app_data_dir())
                .keep("prefs.json")
                .keep("telementary.json")
                .keep("latest.log")
                .keep("data.sqlite3"),

            send_telemetry: true,
            fetch_mods_automatically: true,

            zoom_factor: 1.0,

            game_prefs: HashMap::new(),
        }
    }
}

impl Prefs {
    pub fn create(app: &AppHandle) -> Result<Self> {
        let path = Self::path();
        fs::create_dir_all(path.parent().unwrap())
            .context("failed to create settings directory")?;

        info!("loading settings from {}", path.display());

        let is_first_run = !path.exists();
        let prefs = match is_first_run {
            true => {
                info!("no settings file found, creating new default");

                let prefs = Prefs {
                    is_first_run,
                    ..Default::default()
                };

                prefs.save().context("failed to write initial settings")?;

                prefs
            }
            false => {
                let mut prefs: Prefs = util::fs::read_json(&path).map_err(|err| {
                    anyhow!("failed to read settings: {} (at {})\n\nThe file might be corrupted or too old to run with your version of Gale.", err, path.display())
                })?;

                prefs.data_dir.keep_files.extend(&[
                    "prefs.json",
                    "telementary.json",
                    "latest.log",
                    "data.sqlite3",
                ]);

                let window = app.get_webview_window("main").unwrap();
                window.zoom(prefs.zoom_factor as f64).ok();

                prefs
            }
        };

        Ok(prefs)
    }

    fn path() -> PathBuf {
        util::path::default_app_config_dir().join("prefs.json")
    }

    fn save(&self) -> Result<()> {
        util::fs::write_json(Self::path(), self, JsonStyle::Pretty)
            .context("failed to save settings")
    }

    fn set(&mut self, value: Self, app: &AppHandle) -> Result<()> {
        // prevent the user from setting the steam exe to the game's exe, for example
        let is_valid_steam_exe = value.steam_exe_path.as_ref().is_some_and(|path| {
            path.file_name()
                .is_some_and(|name| name.to_string_lossy().to_lowercase().contains("steam"))
        });

        if is_valid_steam_exe {
            self.steam_exe_path = value.steam_exe_path;
        } else {
            bail!(
                "Steam executable path is invalid. Maybe you entered the game's location instead?",
            );
        }

        self.steam_library_dir = value.steam_library_dir;

        self.game_prefs = value.game_prefs;
        self.validate_game_prefs()?;

        if self.data_dir != value.data_dir {
            // move profile paths
            let mut manager = app.lock_manager();

            let mut path = value.data_dir.to_path_buf();
            for (key, game) in &mut manager.games {
                path.push(&*key.slug);

                game.path = path.clone();

                path.push("profiles");

                for profile in &mut game.profiles {
                    profile.path = path.join(&profile.name);
                }

                path.pop();
                path.pop();
            }

            manager.save_all(app.db())?;
        }

        self.data_dir.set(value.data_dir.value)?;

        if self.zoom_factor != value.zoom_factor {
            let window = app.get_webview_window("main").unwrap();
            window
                .zoom(value.zoom_factor as f64)
                .context("failed to set zoom level")?;
        }
        self.zoom_factor = value.zoom_factor;

        self.send_telemetry = value.send_telemetry;
        self.fetch_mods_automatically = value.fetch_mods_automatically;

        self.save().context("failed write to settings file")
    }

    fn validate_game_prefs(&mut self) -> Result<()> {
        for (slug, value) in &mut self.game_prefs {
            let Some(game) = game::from_slug(slug) else {
                warn!("game prefs key {} is invalid", slug);
                continue;
            };

            if let Some(platform) = game.platforms.iter().next() {
                value.platform.get_or_insert(platform);
            } else {
                value.platform = None;
                if let LaunchMode::Launcher = value.launch_mode {
                    value.launch_mode = LaunchMode::Direct {
                        instances: 1,
                        interval_secs: 10.0,
                    };
                }
            }

            // make sure people don't select the steam library
            if value.dir_override.as_ref().is_some_and(|path| {
                path.file_name().is_some_and(|name| {
                    let name = name.to_string_lossy().to_lowercase();
                    name.contains("steam") || name.contains("common") || name.contains("steamapps")
                })
            }) {
                value.dir_override = None;
                bail!(
                    "Location override for {} is invalid. Please ensure you selected the game's directory.",
                    slug
                );
            }
        }

        Ok(())
    }

    pub fn cache_dir(&self) -> PathBuf {
        self.data_dir.join("cache")
    }

    pub fn fetch_mods_automatically(&self) -> bool {
        self.fetch_mods_automatically
    }

    pub fn send_telemetry(&self) -> bool {
        self.send_telemetry
    }
}
