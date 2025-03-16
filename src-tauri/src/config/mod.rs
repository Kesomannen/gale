use std::{
    borrow::Cow,
    collections::HashMap,
    fs::{self},
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
    time::SystemTime,
};

use eyre::{Context, OptionExt, Result};
use log::debug;
use rayon::prelude::*;
use walkdir::WalkDir;

use crate::{
    game::{ModLoader, ModLoaderKind},
    profile::Profile,
    util::error::IoResultExt,
};

mod bepinex;
pub mod commands;
mod frontend;
mod gd_weave;
pub mod steam_vdf;

#[derive(Debug, Default)]
pub struct ConfigCache(Vec<AnyFile>);

#[derive(Debug)]
struct AnyFile {
    display_name: String,
    relative_path: PathBuf,
    read_time: SystemTime,
    kind: AnyFileKind,
}

#[derive(Debug)]
enum AnyFileKind {
    BepInEx(bepinex::File),
    GDWeave(gd_weave::File),
    Err(eyre::Error),
    Unsupported,
}

impl AnyFile {
    fn file_stem(&self) -> Cow<str> {
        self.relative_path
            .file_stem()
            .expect("file should have name")
            .to_string_lossy()
    }

    fn write(&self, profile_dir: &Path) -> Result<()> {
        debug!("writing config file to {}", self.relative_path.display());

        let path = profile_dir.join(&self.relative_path);
        let writer = fs::File::create(&path)
            .map(BufWriter::new)
            .fs_context("opening file", &path)?;

        match &self.kind {
            AnyFileKind::BepInEx(file) => file.write(writer),
            AnyFileKind::GDWeave(file) => file.write(writer),
            AnyFileKind::Err(_) => Ok(()),
            AnyFileKind::Unsupported => Ok(()),
        }
    }
}

impl AnyFileKind {
    fn mod_name(&self) -> Option<&str> {
        match self {
            Self::BepInEx(file) => file.mod_name(),
            _ => None,
        }
    }
}

impl Profile {
    pub fn refresh_config(&mut self) {
        self.config_cache.refresh(&self.path, &self.game.mod_loader);
        self.link_config();
    }

    fn link_config(&mut self) {
        for profile_mod in &self.mods {
            let ident = profile_mod.ident();
            let file = self
                .config_cache
                .0
                .iter()
                .find(|file| matches(file, ident.name()));

            if let Some(file) = file {
                self.linked_config
                    .insert(profile_mod.uuid(), file.relative_path.clone());
            }
        }

        fn matches(file: &AnyFile, mod_name: &str) -> bool {
            if file.relative_path.as_os_str() == mod_name {
                return true;
            }

            let Some(metadata_name) = file.kind.mod_name() else {
                return false;
            };

            metadata_name == mod_name
        }
    }
}

impl ConfigCache {
    pub fn refresh(&mut self, root: &Path, mod_loader: &ModLoader) {
        let config_dir = root.join(mod_loader.config_path());

        let files = WalkDir::new(&config_dir)
            .into_iter()
            .par_bridge()
            .filter_map(Result::ok)
            .filter_map(|entry| self.read_file(entry, root, &config_dir, mod_loader))
            .collect_vec_list()
            .into_iter()
            .flatten();

        for (file, index) in files {
            match index {
                Some(index) => self.0[index] = file,
                None => self.0.push(file),
            };
        }

        self.resolve_duplicate_names();
    }

    fn read_file(
        &self,
        entry: walkdir::DirEntry,
        root: &Path,
        config_dir: &Path,
        mod_loader: &ModLoader,
    ) -> Option<(AnyFile, Option<usize>)> {
        const EXTENSIONS: &[&str] = &["cfg", "txt", "json", "yml", "yaml", "ini", "xml"];

        let extension = entry.path().extension().and_then(|ext| ext.to_str())?;

        let relative_path = entry
            .path()
            .strip_prefix(root)
            .expect("file path should be a child of root")
            .to_path_buf();

        let curr_index = self
            .0
            .iter()
            .position(|file| file.relative_path == relative_path);

        if !self.needs_refresh(curr_index, &entry) {
            return None;
        }

        let kind = match (&mod_loader.kind, extension) {
            (ModLoaderKind::BepInEx { .. }, "cfg") => {
                read_file(&entry, bepinex::File::read, AnyFileKind::BepInEx)
            }
            (ModLoaderKind::GDWeave {}, "json") => {
                read_file(&entry, gd_weave::File::read, AnyFileKind::GDWeave)
            }
            (_, ext) if EXTENSIONS.contains(&ext) => AnyFileKind::Unsupported,
            _ => return None,
        };

        let display_name = match kind.mod_name() {
            Some(name) => Cow::Borrowed(name),
            None => match &kind {
                AnyFileKind::BepInEx(_) | AnyFileKind::GDWeave(_) => {
                    relative_path.file_stem().unwrap().to_string_lossy()
                }
                AnyFileKind::Unsupported | AnyFileKind::Err(_) => entry
                    .path()
                    .strip_prefix(config_dir)
                    .unwrap()
                    .to_string_lossy(),
            },
        }
        .replace(['_', '-', ' '], "");

        let file = AnyFile {
            display_name,
            relative_path,
            read_time: SystemTime::now(),
            kind,
        };

        return Some((file, curr_index));

        fn read_file<T, F, G>(entry: &walkdir::DirEntry, f: F, g: G) -> AnyFileKind
        where
            F: FnOnce(BufReader<fs::File>) -> Result<T>,
            G: FnOnce(T) -> AnyFileKind,
        {
            let file = fs::File::open(entry.path())
                .map(BufReader::new)
                .context("failed to open file")
                .and_then(f);

            match file {
                Ok(file) => g(file),
                Err(err) => AnyFileKind::Err(err),
            }
        }
    }

    fn needs_refresh(&self, curr_index: Option<usize>, entry: &walkdir::DirEntry) -> bool {
        let Some(curr_index) = curr_index else {
            return true;
        };
        let Some(curr_file) = self.0.get(curr_index) else {
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

        for (i, file_a) in self.0.iter().enumerate() {
            for (j, file_b) in self.0[i + 1..].iter().enumerate() {
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
            self.0[index].display_name = new_name;
        }
    }

    fn to_frontend(&self) -> Vec<frontend::File> {
        use frontend::FileKind;

        self.0
            .iter()
            .map(|file| {
                let kind = match &file.kind {
                    AnyFileKind::BepInEx(file) => FileKind::Ok(file.to_frontend()),
                    AnyFileKind::GDWeave(file) => match file.to_frontend() {
                        Ok(file) => FileKind::Ok(file),
                        Err(err) => FileKind::err(err),
                    },
                    AnyFileKind::Err(err) => FileKind::err(err),
                    AnyFileKind::Unsupported => FileKind::Unsupported,
                };

                frontend::File {
                    display_name: file.display_name.clone(),
                    relative_path: file.relative_path.clone(),
                    kind,
                }
            })
            .collect()
    }

    fn find_file(&mut self, file: &Path) -> Result<&mut AnyFile> {
        self.0
            .iter_mut()
            .find(|f| f.relative_path == file)
            .ok_or_eyre("file not found")
    }
}
