use std::{fs, path::PathBuf, sync::Mutex};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use crate::util::IoResultExt;

pub mod commands;

pub fn setup(app: &AppHandle) -> Result<()> {
    let prefs = Prefs::create(app)?;

    app.manage(Mutex::new(prefs));

    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all="camelCase")]
pub struct Prefs {
    #[serde(skip)]
    pub prefs_path: PathBuf,
    pub steam_exe_path: Option<PathBuf>,
    pub cache_path: PathBuf,
    pub temp_path: PathBuf,
    pub data_path: PathBuf,
}

impl Prefs {
    pub fn create(app: &AppHandle) -> Result<Self> {
        let path_resolver = app.path_resolver();
        let config_path = path_resolver
            .app_config_dir()
            .context("failed to resolve preference directory")?;
        fs::create_dir_all(&config_path)?;

        let prefs_path = config_path.join("prefs.json");
        let prefs = match prefs_path.try_exists()? {
            true => {
                let data = fs::read_to_string(&prefs_path)?;
                let mut prefs: Prefs = serde_json::from_str(&data)?;
                prefs.prefs_path = prefs_path;

                prefs
            }
            false => {
                let mut cache_path = path_resolver.app_cache_dir()
                    .context("failed to resolve cache directory")?;
                fs::create_dir_all(&cache_path)?;

                let temp_path = cache_path.join("temp");
                fs::create_dir_all(&temp_path)?;

                cache_path.push("cache");

                let data_path = path_resolver.app_data_dir()
                    .context("failed to resolve data directory")?;
                fs::create_dir_all(&data_path)?;

                Prefs {
                    prefs_path,
                    cache_path,
                    temp_path,
                    data_path,
                    ..Default::default()
                }
            }
        };

        prefs.save()?;

        Ok(prefs)
    }

    pub fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(&self.prefs_path, json)
            .fs_context("writing preferences", &self.prefs_path)?;
        Ok(())
    }
}