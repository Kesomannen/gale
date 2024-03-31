use std::{fmt::Display, fs, io, ops::Range, path::Path, str::FromStr};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use walkdir::WalkDir;

use crate::fs_util;

use super::Profile;

pub mod commands;
mod de;
mod ser;

#[typeshare]
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct File {
    name: String,
    sections: Vec<Section>,
}

impl File {
    fn new(name: &str, sections: Vec<Section>) -> Self {
        Self {
            name: name.to_owned(),
            sections,
        }
    }

    pub fn save(&self, dir: &Path) -> io::Result<()> {
        let mut path = dir.join(&self.name);
        fs_util::add_extension(&mut path, "cfg");
        let contents = ser::to_string(self);
        fs::write(path, contents)
    }

    pub fn entries(&self) -> impl Iterator<Item = &Entry> {
        self.sections.iter().flat_map(|s| s.entries.iter())
    }
}

#[typeshare]
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Section {
    name: String,
    entries: Vec<Entry>,
}

impl Section {
    fn new(name: &str, entries: Vec<Entry>) -> Self {
        Self {
            name: name.to_owned(),
            entries,
        }
    }
}

#[typeshare]
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    name: String,
    description: String,
    type_name: String,
    default_value: Option<Value>,
    value: Value,
}

impl Entry {
    fn new(name: &str, description: &str, default_value: Option<Value>, value: Value) -> Self {
        let type_name = match &value {
            Value::Boolean(_) => "Boolean",
            Value::String(_) => "String",
            Value::Int32(_) => "Int32",
            Value::Single(_) => "Single",
            Value::Double(_) => "Double",
            _ => panic!("cannot determine type name"),
        };

        Self::new_typed(name, description, type_name, default_value, value)
    }

    fn new_typed(
        name: &str,
        description: &str,
        type_name: &str,
        default_value: Option<Value>,
        value: Value,
    ) -> Entry {
        Self {
            name: name.to_owned(),
            description: description.to_owned(),
            type_name: type_name.to_owned(),
            default_value,
            value,
        }
    }

    fn reset(&mut self) -> Result<()> {
        self.value = self
            .default_value
            .clone()
            .ok_or_else(|| anyhow!("no default value"))?;
        Ok(())
    }
}

#[typeshare]
#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "content", rename_all = "camelCase")]
pub enum Value {
    Boolean(bool),
    String(String),
    Enum {
        value: String,
        options: Vec<String>,
    },
    Flags {
        values: Vec<String>,
        options: Vec<String>,
    },
    Int32(Num<i32>),
    Single(Num<f32>),
    Double(Num<f64>),
    Other(String),
}

impl Value {
    fn options(&self) -> Option<&Vec<String>> {
        match self {
            Self::Enum { options, .. } => Some(options),
            Self::Flags { options, .. } => Some(options),
            _ => None,
        }
    }
}

#[typeshare]
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Num<T>
where
    T: Serialize + ToString,
{
    pub value: T,
    pub range: Option<Range<T>>,
}

impl Profile {
    fn refresh_config(&mut self) {
        self.config = load_config(&self.path).collect();
    }

    fn modify_config<F, R>(&mut self, file: &str, section: &str, entry: &str, f: F) -> Result<R>
    where
        F: FnOnce(&mut Entry) -> R,
    {
        let file = self
            .config
            .iter_mut()
            .filter_map(|f| f.as_mut().ok())
            .find(|f| f.name == file)
            .ok_or_else(|| anyhow!("config file {} not found", file))?;

        let section = file
            .sections
            .iter_mut()
            .find(|s| s.name == section)
            .ok_or_else(|| anyhow!("section {} not found in {}", section, self.name))?;

        let mut entry = section
            .entries
            .iter_mut()
            .find(|e| e.name == entry)
            .ok_or_else(|| anyhow!("entry {} not found in section {}", entry, self.name))?;

        let result = f(&mut entry);

        let config_dir = self.path.join("BepInEx").join("config");
        file.save(&config_dir).context("failed to save file")?;

        Ok(result)
    }
}

pub type LoadedFile = anyhow::Result<File, (String, anyhow::Error)>;

pub fn load_config(profile_path: &Path) -> impl Iterator<Item = LoadedFile> {
    let config_path = profile_path.join("BepInEx").join("config");
    WalkDir::new(&config_path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "cfg"))
        .map(move |entry| {
            let name = entry
                .path()
                .strip_prefix(&config_path)
                .unwrap()
                .with_extension("")
                .to_string_lossy()
                .to_string();

            let content = fs::read_to_string(entry.path())
                .map_err(|err| (name.clone(), anyhow!(err)))?;

            let sections = de::from_str(&content)
                .map_err(|err| (name.clone(), anyhow!(err)))?;

            Ok(File { name, sections })
        })
}
