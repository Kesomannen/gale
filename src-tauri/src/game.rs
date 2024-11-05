use std::{
    borrow::Cow,
    hash::{self, Hash},
    path::PathBuf,
};

use heck::{ToKebabCase, ToPascalCase};
use serde::{Deserialize, Serialize};

const JSON: &str = include_str!("../games.json");

lazy_static! {
    static ref GAMES: Vec<GameData<'static>> = serde_json::from_str(JSON).unwrap();
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct JsonGame<'a> {
    name: &'a str,
    #[serde(default)]
    slug: Option<&'a str>,
    #[serde(default)]
    popular: bool,
    #[serde(default, rename = "r2dirName")]
    r2_dir_name: Option<&'a str>,
    #[serde(borrow)]
    mod_loader: ModLoader<'a>,
    platforms: JsonPlatforms<'a>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct JsonPlatforms<'a> {
    steam: Option<Steam<'a>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase", from = "JsonGame")]
pub struct GameData<'a> {
    pub name: &'a str,
    pub slug: Cow<'a, str>,
    pub r2_dir_name: Cow<'a, str>,
    pub popular: bool,
    pub mod_loader: ModLoader<'a>,
    pub platforms: Vec<Platform<'a>>,
}

impl<'a> From<JsonGame<'a>> for GameData<'a> {
    fn from(value: JsonGame<'a>) -> Self {
        let JsonGame {
            name,
            slug,
            popular,
            r2_dir_name,
            mod_loader,
            platforms,
        } = value;

        let slug = match slug {
            Some(slug) => Cow::Borrowed(slug),
            None => Cow::Owned(name.to_kebab_case()),
        };

        let r2_dir_name = match r2_dir_name {
            Some(name) => Cow::Borrowed(name),
            None => Cow::Owned(slug.to_pascal_case()),
        };

        let platforms = {
            let mut vec = Vec::new();

            if let Some(steam) = platforms.steam {
                vec.push(Platform::Steam(steam));
            }

            vec
        };

        Self {
            name,
            slug,
            r2_dir_name,
            popular,
            mod_loader,
            platforms,
        }
    }
}

impl PartialEq for GameData<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.slug == other.slug
    }
}

impl Eq for GameData<'_> {}

impl Hash for GameData<'_> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.slug.hash(state);
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase", tag = "name")]
pub enum Platform<'a> {
    Steam(Steam<'a>),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Steam<'a> {
    pub id: u32,
    #[serde(default)]
    pub dir_name: Cow<'a, str>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SubdirMode {
    /// Separate mods into `author-name` dirs.
    Separate,
    /// Same as [`SubdirMode::Separate`], but also flatten any dirs that
    /// come before the subdir.
    #[default]
    SeparateFlatten,
    /// Track which files are installed by which mod.
    Track,
    /// Don't track or separate mods. This prevents disabling
    /// or uninstallation of files in the subdir.
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subdir<'a> {
    /// The name which "triggers" the subdir. Must be a single path component.
    pub name: &'a str,
    /// The target path of the subdir, relative to the profile dir.
    ///
    /// Use forward slashes to separate path components.
    pub target: &'a str,
    #[serde(default)]
    pub mode: SubdirMode,
    /// Whether files in this subdir can be/are expected to be mutated.
    ///
    /// When this is `false` (as default), files are installed using hard links
    /// instead of copying, which saves disk space and copy time.
    #[serde(default)]
    pub mutable: bool,
    /// File extension(s) that automatically route to this subdir.
    /// Multiple extensions are separated by a comma.
    #[serde(default)]
    pub extension: Option<&'a str>,
}

impl<'a> Subdir<'a> {
    pub const fn new(name: &'a str, target: &'a str, mode: SubdirMode) -> Self {
        Self {
            name,
            target,
            mode,
            mutable: false,
            extension: None,
        }
    }

    pub const fn separate(name: &'a str, target: &'a str) -> Self {
        Self::new(name, target, SubdirMode::Separate)
    }

    pub const fn separate_flat(name: &'a str, target: &'a str) -> Self {
        Self::new(name, target, SubdirMode::SeparateFlatten)
    }

    pub const fn track(name: &'a str, target: &'a str) -> Self {
        Self::new(name, target, SubdirMode::Track)
    }

    pub const fn untracked(name: &'a str, target: &'a str) -> Self {
        Self::new(name, target, SubdirMode::None)
    }

    pub const fn mutable(mut self) -> Self {
        self.mutable = true;
        self
    }

    pub const fn extension(mut self, ext: &'a str) -> Self {
        self.extension = Some(ext);
        self
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModLoader<'a> {
    #[serde(default)]
    pub package_name: Option<&'a str>,
    #[serde(default, borrow, rename = "subdirs")]
    pub extra_sub_dirs: Vec<Subdir<'a>>,
    #[serde(flatten)]
    pub kind: ModLoaderKind,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "name")]
pub enum ModLoaderKind {
    BepInEx,
    MelonLoader,
}

impl<'a> ModLoader<'a> {
    /// Checks for the mod loader's own package on Thunderstore.
    ///
    /// We need the `package_name` field because this name sometimes varies from game to game.
    pub fn is_package(&self, full_name: &str) -> bool {
        if let Some(package_name) = self.package_name {
            full_name == package_name
        } else {
            match &self.kind {
                ModLoaderKind::BepInEx { .. } => full_name.starts_with("BepInEx-BepInExPack"),
                ModLoaderKind::MelonLoader => full_name == "LavaGang-MelonLoader",
            }
        }
    }

    pub fn log_path(&self) -> PathBuf {
        match &self.kind {
            ModLoaderKind::BepInEx { .. } => &["BepInEx", "LogOutput.log"],
            ModLoaderKind::MelonLoader => &["MelonLoader", "Latest.log"],
        }
        .iter()
        .collect()
    }

    pub fn default_subdir(&self) -> Option<&Subdir> {
        match &self.kind {
            ModLoaderKind::BepInEx { .. } => {
                const SUBDIR: &Subdir = &Subdir::separate_flat("plugins", "BepInEx/plugins");
                Some(SUBDIR)
            }
            ModLoaderKind::MelonLoader => {
                const SUBDIR: &Subdir = &Subdir::track("Mods", "Mods");
                Some(SUBDIR)
            }
        }
    }

    pub fn subdirs(&self) -> impl Iterator<Item = &Subdir> {
        let default = match &self.kind {
            ModLoaderKind::BepInEx => {
                const SUBDIRS: &[Subdir] = &[
                    Subdir::separate_flat("plugins", "BepInEx/plugins"),
                    Subdir::separate_flat("patchers", "BepInEx/patchers"),
                    Subdir::separate_flat("monomod", "BepInEx/monomod").extension(".mm.dll"),
                    Subdir::separate_flat("core", "BepInEx/core"),
                    Subdir::untracked("config", "BepInEx/config").mutable(),
                ];
                SUBDIRS
            }
            ModLoaderKind::MelonLoader => {
                const SUBDIRS: &[Subdir] = &[
                    Subdir::track("UserLibs", "UserLibs").extension(".lib.dll"),
                    Subdir::track("Managed", "MelonLoader/Managed").extension(".managed.dll"),
                    Subdir::track("Mods", "Mods").extension(".dll"),
                    Subdir::separate("ModManager", "UserData/ModManager"),
                    Subdir::track("MelonLoader", "MelonLoader"),
                    Subdir::track("Libs", "MelonLoader/Libs"),
                ];
                SUBDIRS
            }
        };

        self.extra_sub_dirs.iter().chain(default.iter())
    }

    pub fn match_subdir(&self, name: &str) -> Option<&Subdir> {
        self.subdirs().find(|subdir| {
            subdir.name == name
                || subdir
                    .extension
                    .is_some_and(|ext| ext.split(',').any(|ext| name.ends_with(ext)))
        })
    }
}

pub type Game = &'static GameData<'static>;

pub fn all() -> impl Iterator<Item = Game> {
    GAMES.iter()
}

pub fn from_slug(slug: &str) -> Option<Game> {
    GAMES.iter().find(|game| game.slug == slug)
}
