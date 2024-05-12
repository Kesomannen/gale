use std::{
    fmt::Display,
    fs, io,
    ops::Range,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use walkdir::WalkDir;

use crate::fs_util;

use super::Profile;
use tauri::AppHandle;

pub mod commands;
pub mod de;
pub mod ser;
#[cfg(test)]
mod tests;

pub fn setup(_app: &AppHandle) -> Result<()> {
    Ok(())
}

#[typeshare]
#[derive(Serialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase", tag = "type", content = "content")]
pub enum LoadFileResult {
    Ok(File),
    Err {
        name: String,
        error: String,
    },
}

impl LoadFileResult {
    pub fn name(&self) -> &str {
        match self {
            Self::Ok(f) => &f.name,
            Self::Err { name, .. } => name,
        }
    }

    pub fn path_relative(&self) -> PathBuf {
        path_relative(self.name())
    }

    pub fn path_from(&self, root: &Path) -> PathBuf {
        path_from(self.name(), root)
    }

    pub fn ok_ref(&self) -> Option<&File> {
        match self {
            Self::Ok(f) => Some(f),
            Self::Err { .. } => None,
        }
    }

    pub fn ok_mut(&mut self) -> Option<&mut File> {
        match self {
            Self::Ok(f) => Some(f),
            Self::Err { .. } => None,
        }
    }
}

impl From<std::result::Result<File, (String, anyhow::Error)>> for LoadFileResult {
    fn from(value: std::result::Result<File, (String, anyhow::Error)>) -> Self {
        match value {
            Ok(f) => Self::Ok(f),
            Err((name, error)) => Self::Err { name, error: format!("{:#}", error) },
        }
    }
}

#[typeshare]
#[derive(Serialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct File {
    name: String,
    metadata: Option<FileMetadata>,
    sections: Vec<Section>,
}

impl File {
    pub fn path_relative(&self) -> PathBuf {
        path_relative(&self.name)
    }

    pub fn path_from(&self, root: &Path) -> PathBuf {
        path_from(&self.name, root)
    }

    pub fn save(&self, root: &Path) -> io::Result<()> {
        fs::write(self.path_from(root), ser::to_string(self))
    }
}

#[typeshare]
#[derive(Serialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FileMetadata {
    plugin_name: String,
    plugin_version: semver::Version,
    plugin_guid: String,
}

pub fn path_relative(name: &str) -> PathBuf {
    let mut path = ["BepInEx", "config", name].iter().collect();
    fs_util::add_extension(&mut path, "cfg");
    path
}

pub fn path_from(name: &str, root: &Path) -> PathBuf {
    root.join(path_relative(name))
}

#[typeshare]
#[derive(Serialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Section {
    name: String,
    entries: Vec<Entry>,
}

#[typeshare]
#[derive(Serialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase", tag = "type", content = "content")]
pub enum Entry {
    Tagged(TaggedEntry),
    Untagged { name: String, value: String },
}

impl Entry {
    pub fn name(&self) -> &str {
        match self {
            Self::Tagged(e) => &e.name,
            Self::Untagged { name, .. } => name,
        }
    }

    fn as_tagged_mut(&mut self) -> Result<&mut TaggedEntry> {
        match self {
            Self::Tagged(e) => Ok(e),
            Self::Untagged { .. } => Err(anyhow!("entry is not tagged")),
        }
    }

    fn as_untagged_mut(&mut self) -> Result<&mut String> {
        match self {
            Self::Tagged(_) => Err(anyhow!("entry is not untagged")),
            Self::Untagged { value, .. } => Ok(value),
        }
    }
}

impl From<TaggedEntry> for Entry {
    fn from(e: TaggedEntry) -> Self {
        Self::Tagged(e)
    }
}

#[typeshare]
#[derive(Serialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TaggedEntry {
    name: String,
    description: String,
    type_name: String,
    default_value: Option<Value>,
    value: Value,
}

impl TaggedEntry {
    fn reset(&mut self) -> Result<()> {
        self.value = self
            .default_value
            .clone()
            .ok_or_else(|| anyhow!("no default value"))?;
        Ok(())
    }
}

#[typeshare]
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
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
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Num<T>
where
    T: Serialize + ToString,
{
    pub value: T,
    pub range: Option<Range<T>>,
}

impl Profile {
    pub fn refresh_config(&mut self) {
        self.config = load_config(self.path.clone()).collect();
    }

    pub fn ok_config(&self) -> impl Iterator<Item = &File> {
        self.config.iter().filter_map(|res| res.ok_ref())
    }

    fn find_config_file<'a>(&'a self, name: &str) -> Result<&'a LoadFileResult> {
        self.config
            .iter()
            .find(|f| f.name() == name)
            .ok_or_else(|| anyhow!("config file {} not found in profile {}", name, self.name))
    }

    fn modify_config<F, R>(&mut self, file: &str, section: &str, entry: &str, f: F) -> Result<R>
    where
        F: FnOnce(&mut Entry) -> Result<R>,
    {
        let file = self
            .config
            .iter_mut()
            .filter_map(|f| f.ok_mut())
            .find(|f| f.name == file)
            .ok_or_else(|| anyhow!("config file {} not found in profile {}", file, self.name))?;

        let section = file
            .sections
            .iter_mut()
            .find(|s| s.name == section)
            .ok_or_else(|| anyhow!("section {} not found in file {}", section, self.name))?;

        let entry = section
            .entries
            .iter_mut()
            .find(|e| e.name() == entry)
            .ok_or_else(|| anyhow!("entry {} not found in section {}", entry, self.name))?;

        let result = f(entry);

        file.save(&self.path)
            .context("failed to save config file")?;

        result
    }
}

pub fn load_config(mut path: PathBuf) -> impl Iterator<Item = LoadFileResult> {
    path.push("BepInEx");
    path.push("config");

    WalkDir::new(&path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "cfg"))
        .map(move |entry| {
            let name = entry
                .path()
                .strip_prefix(&path)
                .unwrap()
                .with_extension("")
                .to_string_lossy()
                .to_string();

            match fs::read_to_string(entry.path()) {
                Ok(content) => de::from_str(&content, name),
                Err(err) => LoadFileResult::Err {
                    name,
                    error: format!("failed to read file: {:#}", err),
                },
            }
        })
}
