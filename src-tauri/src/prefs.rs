use std::{fs, path::PathBuf, sync::Mutex};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

pub mod commands;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Prefs {
    pub steam_exe_path: Option<PathBuf>,
    pub cache_path: PathBuf,
    pub data_path: PathBuf,
    pub auto_start: bool
}

pub struct PrefsState {
    path: PathBuf,
    inner: Mutex<Prefs>,
}

type Result<T> = anyhow::Result<T>;

impl PrefsState {
    pub fn init(app: &AppHandle) -> Result<Self> {
        println!("initiating preferences");

        let path_resolver = app.path_resolver();
        let config_path = path_resolver
            .app_config_dir()
            .context("failed to get config directory")?;
        fs::create_dir_all(&config_path)?;

        let save_path = config_path.join("config.json");
        let save_data = match save_path.try_exists()? {
            true => {
                let data = fs::read_to_string(&save_path)?;
                serde_json::from_str(&data)?
            }
            false => {
                let cache_path = path_resolver.app_cache_dir().unwrap().join("cache");
                fs::create_dir_all(&cache_path)?;

                let data_path = path_resolver.app_data_dir().unwrap();
                fs::create_dir_all(&data_path)?;

                Prefs {
                    cache_path,
                    data_path,
                    ..Default::default()
                }
            }
        };

        let options = Self {
            path: save_path,
            inner: Mutex::new(save_data),
        };

        options.save()?;

        Ok(options)
    }

    pub fn lock(&self) -> std::sync::MutexGuard<Prefs> {
        self.inner.lock().unwrap()
    }

    pub fn save(&self) -> Result<()> {
        let json = serde_json::to_string(&*self.lock())?;
        fs::write(&self.path, json)?;
        Ok(())
    }
}