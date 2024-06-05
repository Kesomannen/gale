use std::{
    fmt::Display,
    fs, io,
    ops::Range,
    path::{Path, PathBuf},
    str::FromStr,
    time::SystemTime,
};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use typeshare::typeshare;
use walkdir::WalkDir;

use crate::{manager::Profile, thunderstore::Thunderstore, util};

use log::debug;
use tauri::AppHandle;

pub mod commands;
pub mod de;
pub mod ser;

#[cfg(test)]
mod tests;

pub fn setup(_app: &AppHandle) -> Result<()> {
    Ok(())
}

#[derive(Error, Debug)]
#[error("failed to load config file: {error}")]
pub struct LoadFileError {
    name: String,
    error: anyhow::Error,
}

pub type LoadFileResult = std::result::Result<File, LoadFileError>;

pub trait LoadFileResultExt {
    fn name(&self) -> &str;
    fn path_relative(&self) -> PathBuf;
    fn path_from(&self, root: &Path) -> PathBuf;
}

impl LoadFileResultExt for LoadFileResult {
    fn name(&self) -> &str {
        match self {
            Ok(file) => &file.name,
            Err(err) => &err.name,
        }
    }

    fn path_relative(&self) -> PathBuf {
        file_path_relative(self.name())
    }

    fn path_from(&self, root: &Path) -> PathBuf {
        file_path_from(self.name(), root)
    }
}

#[typeshare]
#[derive(Serialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct File {
    name: String,
    #[serde(skip)]
    read_time: SystemTime,
    metadata: Option<FileMetadata>,
    sections: Vec<Section>,
}

impl File {
    pub fn new(name: String, sections: Vec<Section>, metadata: Option<FileMetadata>) -> Self {
        Self {
            name,
            read_time: SystemTime::now(),
            metadata,
            sections,
        }
    }

    pub fn path_from(&self, root: &Path) -> PathBuf {
        file_path_from(&self.name, root)
    }

    pub fn save(&self, root: &Path) -> io::Result<()> {
        let mut file = fs::File::create(self.path_from(root))?;
        ser::to_writer(self, &mut file)
    }
}

#[typeshare]
#[derive(Serialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FileMetadata {
    plugin_name: String,
    plugin_version: String,
    plugin_guid: String,
}

pub fn file_path_relative(name: &str) -> PathBuf {
    let mut path = ["BepInEx", "config", name].iter().collect();
    util::fs::add_extension(&mut path, "cfg");
    path
}

pub fn file_path_from(name: &str, root: &Path) -> PathBuf {
    root.join(file_path_relative(name))
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
        index: usize,
        options: Vec<String>,
    },
    Flags {
        indicies: Vec<usize>,
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
    pub fn refresh_config(&mut self, thunderstore: Option<&Thunderstore>) {
        load_config(self.path.clone(), &mut self.config);
        if let Some(thunderstore) = thunderstore {
            self.link_config(thunderstore);
        }
    }

    fn link_config(&mut self, thunderstore: &Thunderstore) {
        for profile_mod in &self.mods {
            let name = match profile_mod.kind.name(thunderstore) {
                Ok(name) => name,
                Err(_) => {
                    continue;
                }
            };

            let file = self.config.iter().find(|file| matches(file, name));

            if let Some(file) = file {
                debug!("linked {} to config file", name);
                self.linked_config
                    .insert(*profile_mod.uuid(), file.name().to_owned());
            }
        }

        fn matches(file: &LoadFileResult, mod_name: &str) -> bool {
            if file.name() == mod_name {
                return true;
            }

            let file = match file {
                Ok(file) => file,
                Err(_) => return false,
            };

            let meta = match &file.metadata {
                Some(meta) => meta,
                None => return false,
            };

            mod_name == meta.plugin_name
                || mod_name == meta.plugin_guid
                || mod_name.replace('_', "") == meta.plugin_name.replace(' ', "")
        }
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
            .filter_map(|f| f.as_mut().ok())
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

pub fn load_config(mut root: PathBuf, vec: &mut Vec<LoadFileResult>) {
    root.push("BepInEx");
    root.push("config");

    let files = WalkDir::new(&root)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "cfg"));

    for entry in files {
        load_config_file(entry, &root, vec);
    }
}

fn load_config_file(entry: walkdir::DirEntry, root: &Path, vec: &mut Vec<LoadFileResult>) {
    let name = entry
        .path()
        .strip_prefix(root)
        .unwrap()
        .with_extension("")
        .to_string_lossy()
        .to_string();

    let curr_index = vec.iter().position(|f| f.name() == name);

    if let Some(curr_index) = curr_index {
        if let Ok(curr_file) = &vec[curr_index] {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    if modified <= curr_file.read_time {
                        debug!("skipping config file {}", name);
                        return; // file is not modified
                    }
                }
            }
        }
    }

    debug!("reading config file {}", name);

    let data = fs::read_to_string(entry.path())
        .context("failed to read file")
        .and_then(|text| de::from_str(&text));

    let res = match data {
        Ok((sections, metadata)) => Ok(File::new(name, sections, metadata)),
        Err(error) => Err(LoadFileError { name, error }),
    };

    if let Some(curr_index) = curr_index {
        vec[curr_index] = res; // replace the old file
    } else {
        vec.push(res);
    }
}
