use std::{
    collections::HashMap,
    fs,
    ops::Deref,
    path::{Path, PathBuf},
};

use eyre::{bail, ensure, Context, Result};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tracing::{debug, info, warn};

use crate::{
    db::{self, Db},
    game::{self, Platform},
    logger,
    profile::launch::LaunchMode,
    state::ManagerExt,
    util::{
        self,
        error::IoResultExt,
        fs::{Overwrite, PathExt, UseLinks},
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
    pub steam_exe_path: Option<PathBuf>,
    pub data_dir: DirPref,

    pub send_telemetry: bool,
    pub fetch_mods_automatically: bool,
    pub zoom_factor: f32,
    pub pull_before_launch: bool,

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

#[cfg(target_os = "windows")]
fn default_steam_exe_path() -> PathBuf {
    match read_steam_registry() {
        Ok(path) => {
            info!(
                "read steam installation path from registry: {}",
                path.display()
            );
            path.join("steam.exe")
        }
        Err(err) => {
            warn!(
                "failed to read steam installation path from registry: {:#}, using default",
                err
            );
            r"C:\Program Files (x86)\Steam\steam.exe".into()
        }
    }
}

#[cfg(target_os = "linux")]
fn default_steam_exe_path() -> PathBuf {
    "/usr/bin/steam".into()
}

impl Default for Prefs {
    fn default() -> Self {
        let steam_exe_path = default_steam_exe_path().exists_or_none();

        Self {
            steam_exe_path,
            data_dir: DirPref::new(util::path::default_app_data_dir())
                .keep(logger::FILE_NAME)
                .keep(db::FILE_NAME)
                .keep(db::SHM_FILE_NAME)
                .keep(db::WAL_FILE_NAME),

            send_telemetry: true,
            fetch_mods_automatically: true,
            pull_before_launch: true,

            zoom_factor: 1.0,

            game_prefs: HashMap::new(),
        }
    }
}

impl Prefs {
    pub fn init(&mut self, db: &Db, app: &AppHandle) -> Result<()> {
        self.data_dir.keep_files.extend(&[
            logger::FILE_NAME,
            db::FILE_NAME,
            db::SHM_FILE_NAME,
            db::WAL_FILE_NAME,
        ]);

        let window = app.get_webview_window("main").unwrap();
        window.zoom(self.zoom_factor as f64).ok();

        self.save(db)?;

        Ok(())
    }

    fn save(&self, db: &Db) -> Result<()> {
        db.save_prefs(self)
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
        self.pull_before_launch = value.pull_before_launch;

        self.save(app.db()).context("failed save prefs")
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
}
