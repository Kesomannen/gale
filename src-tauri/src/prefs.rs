use std::{
    collections::HashMap,
    fs,
    ops::Deref,
    path::{Path, PathBuf},
    sync::Mutex,
};

use anyhow::{anyhow, ensure, Result};
use log::debug;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tauri_plugin_fs::FsExt;

use crate::{
    logger,
    manager::{launcher::LaunchMode, ModManager},
    util::{
        self,
        fs::{JsonStyle, Overwrite, PathExt},
        window::WindowExt,
    },
};

pub mod commands;

pub fn setup(app: &AppHandle) -> Result<()> {
    let prefs = Prefs::create(app)?;

    app.manage(Mutex::new(prefs));

    Ok(())
}

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

        debug!(
            "attempting to rename directory: {} -> {}",
            self.value.display(),
            new_value.display()
        );

        // on windows fs::rename requires the target directory to not exist
        #[cfg(windows)]
        fs::remove_dir(&new_value)?;

        match fs::rename(&self.value, &new_value) {
            Ok(_) => {
                debug!("renaming succeeded");

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
                            util::fs::copy_dir(&moved_path, &original_path, Overwrite::Yes)?;
                        } else {
                            fs::copy(&moved_path, &original_path)?;
                        }
                    }
                }
            }
            Err(err) => {
                debug!("renaming failed, falling back to copying: {}", err);

                fs::create_dir_all(&new_value)?;

                for entry in self.value.read_dir()? {
                    let entry = entry?;
                    let file_name = entry.file_name();

                    if self.keep_files.iter().any(|file| file_name == *file) {
                        continue;
                    }

                    let old_path = entry.path();
                    let new_path = new_value.join(file_name);

                    if entry.file_type()?.is_dir() {
                        util::fs::copy_dir(&old_path, &new_path, Overwrite::Yes)?;
                        fs::remove_dir_all(old_path)?;
                    } else {
                        fs::copy(&old_path, &new_path)?;
                        fs::remove_file(old_path)?;
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
    is_first_run: bool,

    // aliases are for backwards compatibility
    // will be removed in the future
    #[serde(alias = "steam_exe_path")]
    pub steam_exe_path: Option<PathBuf>,
    #[serde(alias = "steam_game_dir")]
    pub steam_library_dir: Option<PathBuf>,

    #[serde(alias = "data_dir")]
    pub data_dir: DirPref,

    enable_mod_cache: bool,
    fetch_mods_automatically: bool,

    #[serde(alias = "zoom_factor")]
    zoom_factor: f32,

    pub game_prefs: HashMap<String, GamePrefs>,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
#[serde(default, rename_all = "camelCase")]
pub struct GamePrefs {
    pub dir_override: Option<PathBuf>,
    pub launch_mode: LaunchMode,
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

    #[cfg(target_os = "macos")]
    {
        "/Applications/Steam.app/Contents/MacOS/Steam".into()
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

    #[cfg(target_os = "macos")]
    {
        Some("~/Library/Application Support/Steam".into())
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
                .keep("logs"),

            enable_mod_cache: true,
            fetch_mods_automatically: true,

            zoom_factor: 1.0,

            game_prefs: HashMap::new(),
        }
    }
}

impl Prefs {
    fn path() -> PathBuf {
        util::path::default_app_config_dir().join("prefs.json")
    }

    pub fn create(app: &AppHandle) -> Result<Self> {
        let path = Self::path();
        fs::create_dir_all(path.parent().unwrap())?;

        let is_first_run = !path.exists();
        let prefs = match is_first_run {
            true => {
                let prefs = Prefs {
                    is_first_run,
                    ..Default::default()
                };

                prefs.save()?;
                prefs
            }
            false => {
                let mut prefs: Prefs = util::fs::read_json(&path).map_err(|err| {
                    anyhow!("failed to read settings: {} (at {})\n\nThe file might be corrupted or too old to run with your version of Gale.", err, path.display())
                })?;

                prefs.data_dir.keep_files.extend(&["prefs.json", "logs"]);

                let window = app.get_webview_window("main").unwrap();
                window.zoom(prefs.zoom_factor as f64).ok();

                prefs
            }
        };

        app.fs_scope().allow_directory(prefs.data_dir.get(), true);

        Ok(prefs)
    }

    fn save(&self) -> Result<()> {
        util::fs::write_json(Self::path(), self, JsonStyle::Pretty)
            .map_err(|err| err.context("failed to save settings"))
    }

    fn set(&mut self, value: Self, app: &AppHandle) -> Result<()> {
        self.steam_exe_path = value.steam_exe_path;
        self.steam_library_dir = value.steam_library_dir;
        self.game_prefs = value.game_prefs;

        if self.data_dir != value.data_dir {
            let scope = app.fs_scope();

            scope.forbid_directory(&self.data_dir, true);
            scope.allow_directory(&value.data_dir, true);

            // move profile paths
            let manager = app.state::<Mutex<ModManager>>();
            let mut manager = manager.lock().unwrap();

            let mut path = value.data_dir.to_path_buf();
            for (id, game) in &mut manager.games {
                path.push(id);
                path.push("profiles");

                for profile in &mut game.profiles {
                    profile.path = path.join(&profile.name);
                }

                path.pop();
                path.pop();
            }
        }

        self.data_dir.set(value.data_dir.value)?;

        if self.zoom_factor != value.zoom_factor {
            let window = app.get_webview_window("main").unwrap();
            if let Err(err) = window.zoom(value.zoom_factor as f64) {
                logger::log_js_err(
                    "Error while updating settings",
                    &anyhow!("failed to set zoom level: {}", err),
                    app,
                );
            }
        }
        self.zoom_factor = value.zoom_factor;

        self.fetch_mods_automatically = value.fetch_mods_automatically;

        self.save()?;
        Ok(())
    }

    pub fn mod_cache_enabled(&self) -> bool {
        self.enable_mod_cache
    }

    pub fn cache_dir(&self) -> PathBuf {
        self.data_dir.get().join("cache")
    }

    pub fn fetch_mods_automatically(&self) -> bool {
        self.fetch_mods_automatically
    }
}
