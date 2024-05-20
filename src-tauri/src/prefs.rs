use std::{collections::HashMap, env, fs, path::PathBuf, sync::Mutex};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::{manager::launcher::LaunchMode, util::IoResultExt};

pub mod commands;

pub fn setup(app: &AppHandle) -> Result<()> {
    let prefs = Prefs::create(app)?;

    app.manage(Mutex::new(prefs));

    Ok(())
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged,rename_all="camelCase")]
pub enum PrefValue {
    Float(f32),
    Path(PathBuf),
    LaunchMode(LaunchMode),
}

impl PrefValue {
    pub fn as_path(&self) -> Option<&PathBuf> {
        match self {
            PrefValue::Path(path) => Some(path),
            _ => None,
        }
    }
}

pub struct Prefs {
    path: PathBuf,
    map: HashMap<String, PrefValue>,
}

impl Prefs {
    pub fn create(app: &AppHandle) -> Result<Self> {
        let path_resolver = app.path_resolver();
        let config_path = path_resolver
            .app_config_dir()
            .context("failed to resolve preference directory")?;
        fs::create_dir_all(&config_path)?;

        let path = config_path.join("prefs.json");
        let mut map = path
            .exists()
            .then(|| -> Result<HashMap<String, PrefValue>> {
                let json = fs::read_to_string(&path)?;
                let map = serde_json::from_str(&json)?;
                Ok(map)
            })
            .transpose()?
            .unwrap_or_default();

        if map.get("steam_exe_path").is_none() {
            let steam_path = PathBuf::from(match env::consts::OS {
                "windows" => r"C:\Program Files (x86)\Steam\steam.exe",
                "macos" => "/Applications/Steam.app/Contents/MacOS/Steam",
                "linux" => "/usr/bin/steam",
                _ => "",
            });

            if steam_path.exists() {
                map.insert("steam_exe_path".to_owned(), PrefValue::Path(steam_path));
            }
        }

        insert_default_path(&mut map, "data_dir", || {
            path_resolver
                .app_data_dir()
                .context("failed to resolve app data dir")
        })?;

        insert_default_path(&mut map, "cache_dir", || {
            path_resolver
                .app_cache_dir()
                .context("failed to resolve app cache dir")
                .map(|cache_dir| cache_dir.join("cache"))
        })?;

        insert_default_path(&mut map, "temp_dir", || {
            path_resolver
                .app_cache_dir()
                .context("failed to resolve app temp dir")
                .map(|cache_dir| cache_dir.join("temp"))
        })?;
        
        map.entry("launch_mode".to_owned())
            .or_insert(PrefValue::LaunchMode(LaunchMode::Steam));

        map.entry("zoom_factor".to_owned())
            .or_insert(PrefValue::Float(1.0));

        let prefs = Self { path, map };

        prefs.save()?;

        return Ok(prefs);

        fn insert_default_path<F>(
            map: &mut HashMap<String, PrefValue>,
            key: &str,
            get_default: F,
        ) -> Result<()>
        where
            F: FnOnce() -> Result<PathBuf>,
        {
            if map.get(key).is_none() {
                map.insert(key.to_owned(), PrefValue::Path(get_default()?));
            }

            Ok(())
        }
    }

    pub fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.map)?;
        fs::write(&self.path, json).fs_context("saving preferences", &self.path)?;
        Ok(())
    }

    pub fn get_path_or_err(&self, key: &str) -> Result<&PathBuf> {
        self.get_or_err(key)?
            .as_path()
            .ok_or_else(|| anyhow!("pref {} is not a path", key))
    }

    pub fn get_or_err(&self, key: &str) -> Result<&PrefValue> {
        self.get(key)
            .ok_or_else(|| anyhow!("pref {} not found", key))
    }

    pub fn get<'a>(&'a self, key: &str) -> Option<&'a PrefValue> {
        self.map.get(key)
    }

    pub fn set(&mut self, key: impl Into<String>, value: PrefValue) -> Result<()> {
        self.map.insert(key.into(), value);
        self.save()?;
        Ok(())
    }
}
