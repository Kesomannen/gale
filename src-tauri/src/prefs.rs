use std::{
    collections::HashMap,
    env, fs,
    ops::Deref,
    path::{Path, PathBuf},
    sync::Mutex,
};

use anyhow::{anyhow, bail, ensure, Result};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::{
    manager::launcher::LaunchMode,
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

#[derive(Serialize, Deserialize, Clone, Debug)]
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

    pub fn set(&mut self, value: PathBuf) -> Result<()> {
        if self.value == value {
            return Ok(());
        }

        ensure!(value.is_dir(), "value is not a directory");
        ensure!(
            !value.starts_with(&self.value),
            "value cannot be a subdirectory of the current directory"
        );

        for entry in value.read_dir()? {
            let entry = entry?;

            if !self
                .keep_files
                .iter()
                .any(|file| entry.file_name() == *file)
            {
                bail!("new directory is not empty");
            }
        }

        for entry in self.value.read_dir()? {
            let entry = entry?;
            let file_name = entry.file_name();

            if self
                .keep_files
                .iter()
                .any(|file| file_name == *file)
            {
                continue;
            }

            let old_path = entry.path();
            let new_path = value.join(file_name);
            
            // can't do fs::rename because it doesn't work across drives
            if entry.file_type()?.is_dir() {
                util::fs::copy_dir(&old_path, &new_path, Overwrite::Yes)?;
                fs::remove_dir_all(old_path)?;
            } else {
                fs::copy(&old_path, &new_path)?;
                fs::remove_file(old_path)?;
            }
        }

        // remove only if empty
        fs::remove_dir(&self.value).ok();

        self.value = value;

        Ok(())
    }
}

impl Deref for DirPref {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.value
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
    #[serde(alias = "cache_dir")]
    pub cache_dir: DirPref,
    #[serde(alias = "temp_dir")]
    pub temp_dir: DirPref,

    #[serde(alias = "enable_mod_cache")]
    enable_mod_cache: bool,
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

impl Default for Prefs {
    fn default() -> Self {
        let steam_exe_path = PathBuf::from(match env::consts::OS {
            "windows" => r"C:\Program Files (x86)\Steam\steam.exe",
            "macos" => "/Applications/Steam.app/Contents/MacOS/Steam",
            "linux" => "/usr/bin/steam",
            _ => "",
        })
        .exists_or_none();

        let steam_library_dir = match env::consts::OS {
            "windows" => steam_exe_path
                .as_ref()
                .and_then(|exe| exe.parent().map(|parent| parent.to_path_buf())),
            "macos" => Some("~/Library/Application Support/Steam/steamapps/common".into()),
            "linux" => dirs_next::data_dir().map(|data_dir| data_dir.join("Steam")),
            _ => None,
        }
        .and_then(|path| path.exists_or_none());

        Self {
            is_first_run: false,

            steam_exe_path,
            steam_library_dir,

            data_dir: DirPref::new(util::path::app_data_dir())
                .keep("prefs.json")
                .keep("logs"),
            cache_dir: util::path::app_cache_dir().join("cache").into(),
            temp_dir: util::path::app_cache_dir().join("temp").into(),

            enable_mod_cache: true,
            zoom_factor: 1.0,

            game_prefs: HashMap::new(),
        }
    }
}

impl Prefs {
    fn path() -> PathBuf {
        util::path::app_config_dir().join("prefs.json")
    }

    pub fn create(app: &AppHandle) -> Result<Self> {
        let path = Self::path();
        fs::create_dir_all(path.parent().unwrap())?;

        let is_first_run = !path.exists();
        let prefs = match is_first_run {
            true => Prefs {
                is_first_run,
                ..Default::default()
            },
            false => {
                let mut prefs: Prefs = util::fs::read_json(&path).map_err(|err| {
                    anyhow!(
                        "Failed to read settings: {} (at {})",
                        err,
                        path.display()
                    )
                })?;

                prefs.data_dir.keep_files.extend(&["prefs.json", "logs"]);

                let window = app.get_webview_window("main").unwrap();
                window.zoom(prefs.zoom_factor as f64).ok();

                prefs
            }
        };

        prefs.save()?;

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

        self.data_dir.set(value.data_dir.value)?;
        self.cache_dir.set(value.cache_dir.value)?;
        self.temp_dir.set(value.temp_dir.value)?;

        if self.zoom_factor != value.zoom_factor {
            let window = app.get_webview_window("main").unwrap();
            if let Err(err) = window.zoom(value.zoom_factor as f64) {
                util::error::log(
                    "Error while updating settings",
                    &anyhow!("failed to set zoom level: {}", err),
                    app,
                );
            }
        }
        self.zoom_factor = value.zoom_factor;

        if self.enable_mod_cache && !value.enable_mod_cache {
            fs::remove_dir_all(&*self.cache_dir)?;
            fs::create_dir_all(&*self.cache_dir)?;
        }
        self.enable_mod_cache = value.enable_mod_cache;

        self.save()?;
        Ok(())
    }

    pub fn mod_cache_enabled(&self) -> bool {
        self.enable_mod_cache
    }
}
