use std::{fs, path::Path};

use anyhow::{Context, Result};
use walkdir::WalkDir;

mod parse;
mod fmt;

pub struct ConfigFile {
    name: String,
    entries: Vec<ConfigEntry>,
}

pub enum ConfigEntry {
    Config {
        name: String,
        description: String,
        default_value: String,
        value: ConfigValue,
    },
    Section {
        name: String,
        entries: Vec<ConfigEntry>,
    },
}

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
    Int32(i32),
    Single(f32),
    Double(f64),
    Other {
        type_name: String,
        value: String,
    }
}

pub fn parse_configs(profile_path: &Path) -> Result<Vec<ConfigFile>> {
    let config_path = profile_path.join("config");
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

            let entries = parse::parse(&content)
                .with_context(|| format!("failed to parse config file '{}'", name))?;

            Ok(ConfigFile { name, entries })
        })
        .collect()
}