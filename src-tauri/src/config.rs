use std::{
    borrow::Cow,
    collections::HashMap,
    fmt::Display,
    fs,
    io::{self, BufReader, BufWriter},
    ops::Range,
    path::{Path, PathBuf},
    str::FromStr,
    time::SystemTime,
};

use anyhow::{anyhow, Context, Result};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use walkdir::WalkDir;

use crate::manager::Profile;

pub mod commands;
pub mod de;
pub mod ser;

#[cfg(test)]
mod tests;

#[derive(Error, Debug)]
#[error("failed to load config file: {}", error)]
pub struct LoadFileError {
    display_name: String,
    relative_path: PathBuf,
    error: anyhow::Error,
}

pub type LoadFileResult = std::result::Result<File, LoadFileError>;

pub trait LoadFileResultExt {
    fn relative_path(&self) -> &Path;
    fn display_name(&self) -> &str;
}

impl LoadFileResultExt for LoadFileResult {
    fn relative_path(&self) -> &Path {
        match self {
            Ok(file) => &file.relative_path,
            Err(err) => &err.relative_path,
        }
    }

    fn display_name(&self) -> &str {
        match self {
            Ok(file) => &file.display_name,
            Err(err) => &err.display_name,
        }
    }
}

#[derive(Serialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct File {
    display_name: String,
    // relative to the BepInEx/config directory
    relative_path: PathBuf,
    #[serde(skip)]
    read_time: SystemTime,
    metadata: Option<FileMetadata>,
    sections: Vec<Section>,
}

impl File {
    pub fn new(
        display_name: String,
        relative_path: PathBuf,
        sections: Vec<Section>,
        metadata: Option<FileMetadata>,
    ) -> Self {
        Self {
            display_name,
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
    pub fn refresh_config(&mut self) -> Vec<PathBuf> {
        let other_files = load_config(self.path.clone(), &mut self.config);
        self.link_config();
        other_files
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

// returns a list of non-cfg files that were found
pub fn load_config(mut profile_dir: PathBuf, vec: &mut Vec<LoadFileResult>) -> Vec<PathBuf> {
    enum File {
        Cfg {
            res: LoadFileResult,
            index: Option<usize>,
        },
        Other(PathBuf),
    }

    const OTHER_EXTENSIONS: &[&str] = &["json", "toml", "yaml", "yml", "xml", "ini"];

    profile_dir.push("BepInEx");
    profile_dir.push("config");

    let files = WalkDir::new(&profile_dir)
        .into_iter()
        .par_bridge()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let extension = entry.path().extension()?;

            if extension == "cfg" {
                load_config_file(entry, &profile_dir, vec)
                    .map(|(res, index)| File::Cfg { res, index })
            } else if OTHER_EXTENSIONS.iter().any(|ext| extension == *ext) {
                let relative_path = entry
                    .path()
                    .strip_prefix(&profile_dir)
                    .expect("file path should be a child of root")
                    .to_path_buf();

                Some(File::Other(relative_path))
            } else {
                None
            }
        })
        .collect_vec_list()
        .into_iter()
        .flatten();

    let mut other_files = Vec::new();

    for file in files {
        match file {
            File::Cfg { res, index } => match index {
                Some(index) => vec[index] = res,
                None => vec.push(res),
            },
            File::Other(path) => other_files.push(path),
        }
    }

    resolve_duplicate_names(vec);

    other_files
}

fn resolve_duplicate_names(vec: &mut [LoadFileResult]) {
    let mut name_changes = HashMap::new();

    for (i, file_a) in vec.iter().enumerate() {
        for (j, file_b) in vec[i + 1..].iter().enumerate() {
            let name_a = file_a.display_name();
            let name_b = file_b.display_name();

            if name_a != name_b {
                continue;
            }

            // find the difference in the file names and append it to the display name
            // to differentiate between the two files

            let path_a = file_stem(file_a);
            let path_b = file_stem(file_b);
            let max_len = path_a.len().min(path_b.len());

            let mut common = 0;
            while common < max_len {
                if path_a.chars().nth(common) != path_b.chars().nth(common) {
                    break;
                }

                common += 1;
            }

            let mut new_name_a = name_a.to_owned();
            new_name_a.push_str(&path_a[common..]);
            name_changes.insert(i, new_name_a);

            let mut new_name_b = name_b.to_owned();
            new_name_b.push_str(&path_b[common..]);
            name_changes.insert(j + i + 1, new_name_b);
        }
    }

    for (index, new_name) in name_changes {
        match &mut vec[index] {
            Ok(file) => file.display_name = new_name,
            Err(err) => err.display_name = new_name,
        }
    }

    fn file_stem(file: &LoadFileResult) -> Cow<str> {
        file.relative_path()
            .file_stem()
            .expect("file should have name")
            .to_string_lossy()
    }
}

fn load_config_file(
    entry: walkdir::DirEntry,
    root: &Path,
    existing: &[LoadFileResult],
) -> Option<(LoadFileResult, Option<usize>)> {
    let relative_path = entry
        .path()
        .strip_prefix(root)
        .expect("file path should be a child of root")
        .to_path_buf();

    let curr_index = existing
        .iter()
        .position(|file| file.relative_path() == relative_path);

    if let Some(curr_index) = curr_index {
        if let Ok(curr_file) = &existing[curr_index] {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    if modified <= curr_file.read_time {
                        return None; // file is not modified since we last read it
                    }
                }
            }
        }
    }

    let data = fs::File::open(entry.path())
        .map(BufReader::new)
        .context("failed to open file")
        .and_then(de::from_reader);

    let display_name = match &data {
        Ok((_, Some(metadata))) => format_name(&metadata.plugin_name),
        _ => {
            let name = relative_path
                .file_stem()
                .expect("file should have name")
                .to_string_lossy();

            format_name(&name)
        }
    };

    let res = match data {
        Ok((sections, metadata)) => Ok(File::new(display_name, relative_path, sections, metadata)),
        Err(error) => Err(LoadFileError {
            display_name,
            relative_path,
            error,
        }),
    };

    return Some((res, curr_index));

    fn format_name(name: &str) -> String {
        name.replace(['_', '-', ' '], "")
    }
}
