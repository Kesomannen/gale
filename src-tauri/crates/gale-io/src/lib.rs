use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::{
    generate_handler,
    plugin::{Builder, TauriPlugin},
};
use walkdir::WalkDir;

mod commands;
mod export;
mod import;
mod modpack;
mod r2modman;

pub fn init() -> TauriPlugin<tauri::Wry> {
    Builder::new("gale-io")
        .invoke_handler(generate_handler![
            commands::read_code,
            commands::read_file,
            commands::import,
            commands::export_file,
            commands::export_code,
            commands::export_modpack,
            commands::publish_modpack,
        ])
        .build()
}

#[derive(Serialize, Deserialize, Debug, Default)]
enum ModManager {
    #[default]
    R2ModMan,
    Gale,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct LegacyProfileManifest {
    profile_name: String,
    mods: Vec<LegacyProfileMod>,
    #[serde(default)]
    source: ModManager,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct LegacyProfileMod {
    name: String,
    enabled: bool,
    #[serde(flatten)]
    kind: LegacyProfileModKind,
}

// this is done in this convoluted way to maintain compatibility with r2modman
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
enum LegacyProfileModKind {
    /// Either from thunderstore or locally
    Default {
        #[serde(rename = "versionNumber")]
        version: LegacyVersion,
    },
    Github {
        tag: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct LegacyVersion {
    major: u32,
    minor: u32,
    patch: u32,
}

fn find_config_files(profile_path: &Path) -> impl Iterator<Item = PathBuf> + '_ {
    // Include any files in the BepInEx/config directory,
    // and any other files with the following extensions:
    const INCLUDE_EXTENSIONS: [&str; 6] = ["cfg", "txt", "json", "yml", "yaml", "ini"];
    const EXCLUDE_FILES: [&str; 5] = [
        "profile.json",
        "manifest.json",
        "mods.yml",
        "doorstop_config.ini",
        "snapshots",
    ];

    let config_dir = ["BepInEx", "config"].into_iter().collect::<PathBuf>();

    WalkDir::new(profile_path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .map(move |entry| {
            entry
                .into_path()
                .strip_prefix(profile_path)
                .expect("WalkDir should only return files within the profile directory")
                .to_path_buf()
        })
        .filter(move |path| {
            !EXCLUDE_FILES
                .iter()
                .any(|exc| path.starts_with(exc) || path.ends_with(exc))
        })
        .filter(move |path| {
            path.starts_with(&config_dir)
                || path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| INCLUDE_EXTENSIONS.contains(&ext))
        })
}
