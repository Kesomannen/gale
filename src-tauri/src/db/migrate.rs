use std::fs;

use eyre::{Context, Result};
use itertools::Itertools;
use log::info;

use crate::{
    game::{self, Platform},
    prefs::{GamePrefs, Prefs},
    profile::{
        export::modpack::ModpackArgs, launch::LaunchMode, LocalMod, ProfileMod, ProfileModKind,
        ThunderstoreMod,
    },
    thunderstore::ModId,
    util,
};

use super::{ManagerData, SaveData};

pub fn should_migrate() -> bool {
    util::path::default_app_config_dir()
        .join("prefs.json")
        .exists()
}

pub fn migrate() -> Result<(SaveData, Prefs)> {
    info!("migrating legacy save data");

    let prefs_path = util::path::default_app_config_dir().join("prefs.json");
    let prefs: legacy::Prefs = util::fs::read_json(&prefs_path).context("failed to read prefs")?;
    let prefs = Prefs::from(prefs);

    let manifest_path = prefs.data_dir.join("manager.json");
    let manager_data: legacy::ManagerSaveData = util::fs::read_json(&manifest_path)?;

    let manager = ManagerData {
        id: 1,
        active_game_slug: Some(manager_data.active_game),
    };

    let mut games = Vec::new();
    let mut profiles = Vec::new();

    let game_dirs = prefs
        .data_dir
        .read_dir()?
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|ty| ty.is_dir()))
        .filter_map(|entry| {
            game::from_slug(&entry.file_name().to_string_lossy()).map(|game| (game, entry.path()))
        });

    for (game, path) in game_dirs {
        let data: legacy::ManagedGameSaveData = util::fs::read_json(path.join("game.json"))?;

        let mut active_profile_id: i64 = 1;

        let profile_dirs = path
            .join("profiles")
            .read_dir()?
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_ok_and(|ty| ty.is_dir()))
            .map(|entry| entry.path());

        for (index, path) in profile_dirs.enumerate() {
            let name = util::fs::file_name_owned(&path);
            let profile_data: legacy::ProfileSaveData =
                util::fs::read_json(path.join("profile.json"))?;

            let id = (profiles.len() + 1) as i64;

            profiles.push(super::ProfileData {
                id,
                name,
                path: path.to_string_lossy().into_owned(),
                game_slug: game.slug.to_string(),
                mods: profile_data.mods.into_iter().map_into().collect(),
                modpack: profile_data.modpack.map(Into::into),
                ignored_updates: Some(profile_data.ignored_updates),
            });

            if data.active_profile_index == index {
                active_profile_id = id;
            }
        }

        games.push(super::ManagedGameData {
            id: (games.len() + 1) as i64,
            slug: game.slug.to_string(),
            favorite: data.favorite,
            active_profile_id,
        });
    }

    fs::rename(&prefs_path, prefs_path.with_extension("old"))
        .context("failed to rename prefs file")?;

    Ok((
        SaveData {
            manager,
            games,
            profiles,
        },
        prefs,
    ))
}

impl From<legacy::Prefs> for Prefs {
    fn from(legacy: legacy::Prefs) -> Self {
        Prefs {
            steam_exe_path: legacy.steam_exe_path,
            data_dir: legacy.data_dir.into(),
            send_telemetry: legacy.send_telemetry,
            fetch_mods_automatically: legacy.fetch_mods_automatically,
            zoom_factor: legacy.zoom_factor,
            game_prefs: legacy
                .game_prefs
                .into_iter()
                .map(|(key, value)| (key, value.into()))
                .collect(),
        }
    }
}

impl From<legacy::GamePrefs> for GamePrefs {
    fn from(legacy: legacy::GamePrefs) -> Self {
        GamePrefs {
            dir_override: legacy.dir_override,
            custom_args: legacy.custom_args,
            launch_mode: legacy.launch_mode.into(),
            platform: legacy.platform.map(Into::into),
        }
    }
}

impl From<legacy::LaunchMode> for LaunchMode {
    fn from(legacy: legacy::LaunchMode) -> Self {
        match legacy {
            legacy::LaunchMode::Launcher => LaunchMode::Launcher,
            legacy::LaunchMode::Direct {
                instances,
                interval_secs,
            } => LaunchMode::Direct {
                instances,
                interval_secs,
            },
        }
    }
}

impl From<legacy::Platform> for Platform {
    fn from(legacy: legacy::Platform) -> Self {
        match legacy {
            legacy::Platform::Steam => Platform::Steam,
            legacy::Platform::EpicGames => Platform::EpicGames,
            legacy::Platform::Oculus => Platform::Oculus,
            legacy::Platform::Origin => Platform::Origin,
            legacy::Platform::XboxStore => Platform::XboxStore,
        }
    }
}

impl From<legacy::ProfileMod> for ProfileMod {
    fn from(legacy: legacy::ProfileMod) -> Self {
        ProfileMod {
            enabled: legacy.enabled,
            install_time: legacy.install_time,
            kind: legacy.kind.into(),
        }
    }
}

impl From<legacy::ProfileModKind> for ProfileModKind {
    fn from(legacy: legacy::ProfileModKind) -> Self {
        match legacy {
            legacy::ProfileModKind::Thunderstore(ts_mod) => {
                ProfileModKind::Thunderstore(ts_mod.into())
            }
            legacy::ProfileModKind::Local(local_mod) => {
                ProfileModKind::Local(Box::new(local_mod.into()))
            }
        }
    }
}

impl From<legacy::ThunderstoreMod> for ThunderstoreMod {
    fn from(legacy: legacy::ThunderstoreMod) -> Self {
        ThunderstoreMod {
            ident: legacy.ident,
            id: legacy.id.into(),
        }
    }
}

impl From<legacy::ModId> for ModId {
    fn from(legacy: legacy::ModId) -> Self {
        ModId {
            package_uuid: legacy.package_uuid,
            version_uuid: legacy.version_uuid,
        }
    }
}

impl From<legacy::LocalMod> for LocalMod {
    fn from(legacy: legacy::LocalMod) -> Self {
        LocalMod {
            name: legacy.name,
            icon: legacy.icon,
            author: legacy.author,
            description: legacy.description,
            version: legacy.version,
            dependencies: legacy.dependencies,
            uuid: legacy.uuid,
            file_size: legacy.file_size,
        }
    }
}

impl From<legacy::ModpackArgs> for ModpackArgs {
    fn from(legacy: legacy::ModpackArgs) -> Self {
        ModpackArgs {
            name: legacy.name,
            description: legacy.description,
            author: legacy.author,
            categories: legacy.categories,
            nsfw: legacy.nsfw,
            readme: legacy.readme,
            changelog: legacy.changelog,
            version_number: legacy.version_number,
            icon_path: legacy.icon_path,
            website_url: legacy.website_url,
            include_disabled: legacy.include_disabled,
            include_files: legacy.include_files,
        }
    }
}

mod legacy {
    use std::{
        collections::{HashMap, HashSet},
        path::PathBuf,
    };

    use chrono::{DateTime, Utc};
    use serde::Deserialize;
    use uuid::Uuid;

    use crate::thunderstore::VersionIdent;

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ManagerSaveData {
        pub active_game: String,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ManagedGameSaveData {
        pub favorite: bool,
        pub active_profile_index: usize,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ProfileSaveData {
        pub mods: Vec<ProfileMod>,

        #[serde(default)]
        pub modpack: Option<ModpackArgs>,

        #[serde(default)]
        pub ignored_updates: HashSet<Uuid>,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ProfileMod {
        pub enabled: bool,

        #[serde(default = "Utc::now")]
        pub install_time: DateTime<Utc>,

        #[serde(flatten)]
        pub kind: ProfileModKind,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", untagged)]
    pub enum ProfileModKind {
        Thunderstore(ThunderstoreMod),
        Local(LocalMod),
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ThunderstoreMod {
        #[serde(rename = "fullName")]
        pub ident: VersionIdent,

        #[serde(flatten)]
        pub id: ModId,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ModId {
        pub package_uuid: Uuid,
        pub version_uuid: Uuid,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct LocalMod {
        pub name: String,
        pub icon: Option<PathBuf>,
        pub author: Option<String>,
        pub description: Option<String>,
        pub version: Option<semver::Version>,
        pub dependencies: Option<Vec<VersionIdent>>,
        pub uuid: Uuid,
        #[serde(default)]
        pub file_size: u64,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ModpackArgs {
        pub name: String,
        pub description: String,
        pub author: String,
        pub categories: Vec<String>,
        pub nsfw: bool,
        pub readme: String,
        #[serde(default)]
        pub changelog: String,
        pub version_number: String,
        pub icon_path: PathBuf,
        pub website_url: String,
        pub include_disabled: bool,
        #[serde(default, rename = "includeFileMap")]
        pub include_files: HashMap<PathBuf, bool>,
    }

    #[derive(Deserialize, Default)]
    #[serde(default, rename_all = "camelCase")]
    pub struct Prefs {
        pub steam_exe_path: Option<PathBuf>,
        pub steam_library_dir: Option<PathBuf>,
        pub data_dir: PathBuf,

        #[serde(alias = "sendTelementary")] // old typo (oops)
        pub send_telemetry: bool,
        pub fetch_mods_automatically: bool,
        pub zoom_factor: f32,

        pub game_prefs: HashMap<String, GamePrefs>,
    }

    #[derive(Deserialize, Default)]
    #[serde(default, rename_all = "camelCase")]
    pub struct GamePrefs {
        pub dir_override: Option<PathBuf>,
        pub custom_args: Option<Vec<String>>,
        pub launch_mode: LaunchMode,
        pub platform: Option<Platform>,
    }

    #[derive(Deserialize, Default)]
    #[serde(rename_all = "camelCase", tag = "type", content = "content")]
    pub enum LaunchMode {
        #[default]
        #[serde(alias = "steam")]
        Launcher,
        #[serde(rename_all = "camelCase")]
        Direct { instances: u32, interval_secs: f32 },
    }

    #[derive(Deserialize, Default)]
    #[serde(rename_all = "camelCase")]
    pub enum Platform {
        #[default]
        Steam,
        EpicGames,
        Oculus,
        Origin,
        XboxStore,
    }
}
