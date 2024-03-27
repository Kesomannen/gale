use core::fmt::Debug;
use std::{fmt::Display, fs, path::Path, str::FromStr};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

mod parser;
mod fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    name: String,
    entries: Vec<ConfigEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigEntry {
    Config {
        name: String,
        description: String,
        default_value: Option<String>,
        value: ConfigValue,
    },
    Section {
        name: String,
        entries: Vec<ConfigEntry>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigValue {
    Boolean(bool),
    String(String),
    Enum {
        value: String,
        options: Vec<String>,
        type_name: String,
    },
    Flags {
        values: Vec<String>,
        options: Vec<String>,
        type_name: String,
    },
    Int32(ConfigRange<i32>),
    Single(ConfigRange<f32>),
    Double(ConfigRange<f64>),
    Other {
        type_name: String,
        value: String,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigRange<T>
where 
    T: Debug + Display + Clone + Serialize
{
    pub value: T,
    pub range: Option<(T, T)>,
}

impl<T, E> ConfigRange<T>
where 
    T: Debug + Display + Clone + Serialize + FromStr<Err = E>,
    E: std::error::Error + Send + Sync + 'static
{
    pub fn parse(value_str: &str, range_str: Option<(&str, &str)>) -> Result<Self> {
        let value = value_str.parse()?;
        let range = range_str.map(|range_str| {
            let min = range_str.0.replace(',', ".").parse::<T>()?;
            let max = range_str.1.replace(',', ".").parse::<T>()?;
            Ok::<(T, T), E>((min, max))
        }).transpose()?;

        Ok(Self { value, range })
    }
}

impl<T> ConfigRange<T>
where 
    T: Debug + Display + Clone + Serialize,
{
    fn comment(&self) -> Option<String> {
        self.range.as_ref().map(|(min, max)| {
            format!("# Acceptable value range: From {} to {}", min, max)
        })
    }
}

pub fn parse_config_files(profile_path: &Path) -> Result<Vec<ConfigFile>> {
    let config_path = profile_path.join("BepInEx").join("config");
    WalkDir::new(&config_path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "cfg"))
        .map(|entry| {
            let name = entry.path()
                .strip_prefix(&config_path)
                .unwrap()
                .with_extension("")
                .to_string_lossy()
                .to_string();

            let content = fs::read_to_string(entry.path())
                .with_context(|| format!("failed to read config file '{}'", name))?;

            let entries = parser::parse(&content)
                .with_context(|| format!("failed to parse config file '{}'", name))?;

            Ok(ConfigFile { name, entries })
        })
        .collect()
}