use std::{
    borrow::Cow,
    collections::HashMap,
    fs::{self},
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
    time::SystemTime,
};

use eyre::{Context, OptionExt, Result, bail, eyre};
use gale_core::game::mod_loader::{ModLoader, ModLoaderKind};
use gale_util::error::IoResultExt;
use rayon::prelude::*;
use tracing::debug;
use uuid::Uuid;
use walkdir::WalkDir;

mod bepinex;
pub mod frontend;
mod gd_weave;

#[derive(Debug, Default)]
pub struct Cache {
    files: Vec<File>,
    links: HashMap<Uuid, PathBuf>,
}

#[derive(Debug)]
pub struct File {
    display_name: String,
    relative_path: PathBuf,
    read_time: SystemTime,
    kind: FileKind,
}

#[derive(Debug)]
enum FileKind {
    BepInEx(bepinex::File),
    GDWeave(gd_weave::File),
    Err(eyre::Error),
    Unsupported,
}

impl File {
    pub fn relative_path(&self) -> &Path {
        &self.relative_path
    }

    fn file_stem(&self) -> Cow<str> {
        self.relative_path
            .file_stem()
            .expect("file should have name")
            .to_string_lossy()
    }

    fn matches(&self, package_name: &str) -> bool {
        if self.relative_path.as_os_str() == package_name {
            return true;
        }

        let Some(metadata_name) = self.kind.plugin_name() else {
            return false;
        };

        metadata_name == package_name
    }

    pub fn write(&self, root_path: &Path) -> Result<()> {
        debug!("writing config file to {}", self.relative_path.display());

        let path = root_path.join(&self.relative_path);
        let writer = fs::File::create(&path)
            .map(BufWriter::new)
            .fs_context("opening file", &path)?;

        match &self.kind {
            FileKind::BepInEx(file) => file.write(writer),
            FileKind::GDWeave(file) => file.write(writer),
            FileKind::Err(_) => Ok(()),
            FileKind::Unsupported => Ok(()),
        }
    }

    pub fn set(&mut self, section: &str, entry: &str, value: frontend::Value) -> Result<()> {
        match &mut self.kind {
            FileKind::BepInEx(file) => file.find_entry(section, entry)?.set(value.into()),
            FileKind::GDWeave(file) => file.set(entry, value.try_into()?),
            _ => Err(eyre!("unsupported for this format").into()),
        }
    }

    pub fn reset(&mut self, section: &str, entry: &str) -> Result<frontend::Value> {
        match &mut self.kind {
            FileKind::BepInEx(file) => file.find_entry(section, entry)?.reset().map(Into::into),
            _ => Err(eyre!("unsupported for this format").into()),
        }
    }
}

impl FileKind {
    fn plugin_name(&self) -> Option<&str> {
        match self {
            Self::BepInEx(file) => file.plugin_name(),
            _ => None,
        }
    }
}

impl Cache {
    pub fn refresh(&mut self, profile_root: &Path, relative_dir: &Path, mod_loader: &ModLoader) {
        let absolute_dir = profile_root.join(relative_dir);

        let files = WalkDir::new(&absolute_dir)
            .into_iter()
            .par_bridge()
            .filter_map(Result::ok)
            .filter_map(|entry| self.read_file(entry, profile_root, &absolute_dir, mod_loader))
            .collect_vec_list()
            .into_iter()
            .flatten();

        for (file, index) in files {
            match index {
                Some(index) => self.files[index] = file,
                None => self.files.push(file),
            };
        }

        self.resolve_duplicate_names();
    }

    pub fn refresh_links<'a>(&mut self, mod_names: impl Iterator<Item = (Uuid, &'a str)>) {
        for (uuid, name) in mod_names {
            let file = self.files.iter().find(|file| file.matches(name));

            if let Some(file) = file {
                self.links.insert(uuid, file.relative_path.clone());
            }
        }
    }

    pub fn link(&self, uuid: Uuid) -> Option<&Path> {
        self.links.get(&uuid).map(|path| &**path)
    }

    fn read_file(
        &self,
        entry: walkdir::DirEntry,
        profile_root: &Path,
        absolute_dir: &Path,
        mod_loader: &ModLoader,
    ) -> Option<(File, Option<usize>)> {
        const KNOWN_EXTENSIONS: &[&str] = &["cfg", "txt", "json", "yml", "yaml", "ini", "xml"];

        let extension = entry.path().extension().and_then(|ext| ext.to_str())?;

        let relative_path = entry
            .path()
            .strip_prefix(profile_root)
            .expect("file path should be a child of root")
            .to_path_buf();

        let curr_index = self
            .files
            .iter()
            .position(|file| file.relative_path == relative_path);

        if !self.needs_refresh(curr_index, &entry) {
            return None;
        }

        let kind = match (&mod_loader.kind, extension) {
            (ModLoaderKind::BepInEx { .. }, "cfg") => {
                read_file(&entry, bepinex::File::read, FileKind::BepInEx)
            }
            (ModLoaderKind::GDWeave {}, "json") => {
                read_file(&entry, gd_weave::File::read, FileKind::GDWeave)
            }
            (_, ext) if KNOWN_EXTENSIONS.contains(&ext) => FileKind::Unsupported,
            _ => return None,
        };

        let display_name = match kind.plugin_name() {
            Some(name) => Cow::Borrowed(name),
            None => match &kind {
                FileKind::BepInEx(_) | FileKind::GDWeave(_) => {
                    relative_path.file_stem().unwrap().to_string_lossy()
                }
                FileKind::Unsupported | FileKind::Err(_) => entry
                    .path()
                    .strip_prefix(absolute_dir)
                    .unwrap()
                    .to_string_lossy(),
            },
        }
        .replace('-', "")
        .replace('_', " ");

        let file = File {
            display_name,
            relative_path,
            read_time: SystemTime::now(),
            kind,
        };

        return Some((file, curr_index));

        fn read_file<T, F, G>(entry: &walkdir::DirEntry, f: F, g: G) -> FileKind
        where
            F: FnOnce(BufReader<fs::File>) -> Result<T>,
            G: FnOnce(T) -> FileKind,
        {
            let file = fs::File::open(entry.path())
                .map(BufReader::new)
                .context("failed to open file")
                .and_then(f);

            match file {
                Ok(file) => g(file),
                Err(err) => FileKind::Err(err),
            }
        }
    }

    fn needs_refresh(&self, curr_index: Option<usize>, entry: &walkdir::DirEntry) -> bool {
        let Some(curr_index) = curr_index else {
            return true;
        };
        let Some(curr_file) = self.files.get(curr_index) else {
            return true;
        };
        let Ok(metadata) = entry.metadata() else {
            return true;
        };
        let Ok(modified) = metadata.modified() else {
            return true;
        };
        if modified > curr_file.read_time {
            return true;
        };

        false
    }

    fn resolve_duplicate_names(&mut self) {
        let mut name_changes = HashMap::new();

        for (i, file_a) in self.files.iter().enumerate() {
            for (j, file_b) in self.files[i + 1..].iter().enumerate() {
                let name_a = &file_a.display_name;
                let name_b = &file_b.display_name;

                if name_a != name_b {
                    continue;
                }

                // find the difference in the file names and append it to the display name
                // to differentiate between the two files

                let path_a = file_a.file_stem();
                let path_b = file_b.file_stem();
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
            self.files[index].display_name = new_name;
        }
    }

    pub fn to_frontend(&self) -> Vec<frontend::File> {
        self.files
            .iter()
            .map(|file| {
                let kind = match &file.kind {
                    FileKind::BepInEx(file) => frontend::FileKind::Ok(file.to_frontend()),
                    FileKind::GDWeave(file) => match file.to_frontend() {
                        Ok(file) => frontend::FileKind::Ok(file),
                        Err(err) => frontend::FileKind::err(err),
                    },
                    FileKind::Err(err) => frontend::FileKind::err(err),
                    FileKind::Unsupported => frontend::FileKind::Unsupported,
                };

                frontend::File {
                    display_name: file.display_name.clone(),
                    relative_path: file.relative_path.clone(),
                    kind,
                }
            })
            .collect()
    }

    pub fn find_file(&mut self, path: &Path) -> Result<&mut File> {
        self.files
            .iter_mut()
            .find(|f| f.relative_path == path)
            .ok_or_eyre("file not found")
    }

    pub fn remove_file(&mut self, path: &Path) -> Result<()> {
        let Some(index) = self.files.iter().position(|f| f.relative_path == path) else {
            bail!("config file not found"); // ignore if the file is not in the list
        };

        self.files.remove(index);
        Ok(())
    }
}
