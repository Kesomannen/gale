use std::{
    fmt::Display,
    fs,
    io::{self, BufReader, BufWriter},
    ops::Range,
    path::{Path, PathBuf},
    str::FromStr,
    time::{Instant, SystemTime},
};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use walkdir::WalkDir;

use crate::{manager::Profile, util::fs::PathExt};
use log::trace;

pub mod commands;
pub mod de;
pub mod ser;

#[cfg(test)]
mod tests;

#[derive(Error, Debug)]
#[error("failed to load config file: {}", error)]
pub struct LoadFileError {
    relative_path: PathBuf,
    error: anyhow::Error,
}

pub type LoadFileResult = std::result::Result<File, LoadFileError>;

pub trait LoadFileResultExt {
    fn relative_path(&self) -> &Path;

    fn path(&self, profile_dir: &Path) -> PathBuf {
        profile_dir.join(file_path(self.relative_path()))
    }
}

impl LoadFileResultExt for LoadFileResult {
    fn relative_path(&self) -> &Path {
        match self {
            Ok(file) => &file.relative_path,
            Err(err) => &err.relative_path,
        }
    }
}

#[derive(Serialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct File {
    // relative to the BepInEx/config directory
    relative_path: PathBuf,
    #[serde(skip)]
    read_time: SystemTime,
    metadata: Option<FileMetadata>,
    sections: Vec<Section>,
}

impl File {
    pub fn new(
        relative_path: PathBuf,
        sections: Vec<Section>,
        metadata: Option<FileMetadata>,
    ) -> Self {
        Self {
            relative_path,
            read_time: SystemTime::now(),
            metadata,
            sections,
        }
    }

    pub fn path(&self, profile_dir: &Path) -> PathBuf {
        profile_dir.join(file_path(&self.relative_path))
    }

    pub fn save(&self, root: &Path) -> io::Result<()> {
        let file = fs::File::create(self.path(root))?;
        let writer = BufWriter::new(file);
        ser::to_writer(self, writer)
    }
}

#[derive(Serialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FileMetadata {
    plugin_name: String,
    plugin_version: String,
    plugin_guid: String,
}

pub fn file_path(relative: &Path) -> PathBuf {
    let mut path = ["BepInEx", "config"].into_iter().collect::<PathBuf>();
    path.push(relative);
    path
}

#[derive(Serialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Section {
    name: String,
    entries: Vec<EntryKind>,
}

#[derive(Serialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum EntryKind {
    Normal(Entry),
    Orphaned { name: String, value: String },
}

impl EntryKind {
    pub fn name(&self) -> &str {
        match self {
            Self::Normal(e) => &e.name,
            Self::Orphaned { name, .. } => name,
        }
    }

    fn as_normal_mut(&mut self) -> Result<&mut Entry> {
        match self {
            Self::Normal(e) => Ok(e),
            Self::Orphaned { .. } => Err(anyhow!("entry is not tagged")),
        }
    }
}

impl From<Entry> for EntryKind {
    fn from(e: Entry) -> Self {
        Self::Normal(e)
    }
}

#[derive(Serialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    name: String,
    description: String,
    type_name: String,
    default_value: Option<Value>,
    value: Value,
}

impl Entry {
    fn reset(&mut self) -> Result<()> {
        self.value = self
            .default_value
            .clone()
            .ok_or(anyhow!("no default value"))?;
        Ok(())
    }
}

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
    fn options(&self) -> Option<&[String]> {
        match self {
            Self::Enum { options, .. } => Some(options),
            Self::Flags { options, .. } => Some(options),
            _ => None,
        }
    }
}

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
        load_config(self.path.clone(), &mut self.config);
        self.link_config();
    }

    fn link_config(&mut self) {
        for profile_mod in &self.mods {
            let name = profile_mod.kind.name();
            let file = self.config.iter().find(|file| matches(file, name));

            if let Some(file) = file {
                self.linked_config
                    .insert(*profile_mod.uuid(), file.relative_path().to_path_buf());
            }
        }

        fn matches(file: &LoadFileResult, mod_name: &str) -> bool {
            if file.relative_path().as_os_str() == mod_name {
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

    fn find_config_file<'a>(&'a self, relative_path: &Path) -> Result<&'a LoadFileResult> {
        self.config
            .iter()
            .find(|f| f.relative_path() == relative_path)
            .ok_or_else(|| {
                anyhow!(
                    "config file at {} not found in profile {}",
                    relative_path.display(),
                    self.name
                )
            })
    }

    fn modify_config<F, R>(
        &mut self,
        relative_path: &Path,
        section: &str,
        entry: &str,
        f: F,
    ) -> Result<R>
    where
        F: FnOnce(&mut EntryKind) -> Result<R>,
    {
        let file = self
            .config
            .iter_mut()
            .filter_map(|f| f.as_mut().ok())
            .find(|f| f.relative_path == relative_path)
            .ok_or_else(|| {
                anyhow!(
                    "config file {} not found in profile {}",
                    relative_path.display(),
                    self.name
                )
            })?;

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

    trace!("loading config files from {}", root.display());

    let start = Instant::now();

    for entry in files {
        load_config_file(entry, &root, vec);
    }

    trace!("loaded config files in {:?}", start.elapsed());
}

fn load_config_file(entry: walkdir::DirEntry, root: &Path, vec: &mut Vec<LoadFileResult>) {
    let relative_path = entry
        .path()
        .strip_prefix(root)
        .expect("file path should be a child of root")
        .to_path_buf();

    trace!("loading config file {:?}", relative_path.display());

    let curr_index = vec
        .iter()
        .position(|file| file.relative_path() == relative_path);

    if let Some(curr_index) = curr_index {
        if let Ok(curr_file) = &vec[curr_index] {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    if modified <= curr_file.read_time {
                        trace!(
                            "file {} is not modified since last read",
                            relative_path.display()
                        );
                        return; // file is not modified since we last read it
                    }
                }
            }
        }
    }

    let start = Instant::now();

    let data = fs::File::open(entry.path())
        .map(BufReader::new)
        .context("failed to open file")
        .and_then(|reader| {
            trace!("opening file took {:?}", start.elapsed());

            let start = Instant::now();

            let data = de::from_reader(reader);

            trace!("reading file took {:?}", start.elapsed());

            data
        });

    let res = match data {
        Ok((sections, metadata)) => Ok(File::new(relative_path, sections, metadata)),
        Err(error) => Err(LoadFileError {
            relative_path,
            error,
        }),
    };

    if let Some(curr_index) = curr_index {
        vec[curr_index] = res; // replace the old file
    } else {
        vec.push(res);
    }
}
