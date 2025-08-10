use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModLoader<'a> {
    #[serde(default)]
    pub package_name: Option<&'a str>,
    #[serde(flatten)]
    pub kind: ModLoaderKind<'a>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "name")]
pub enum ModLoaderKind<'a> {
    BepInEx {
        #[serde(default, borrow, rename = "subdirs")]
        extra_subdirs: Vec<Subdir<'a>>,
    },
    MelonLoader {
        #[serde(default, borrow, rename = "subdirs")]
        extra_subdirs: Vec<Subdir<'a>>,
    },
    Northstar {},
    GDWeave {},
    Shimloader {},
    Lovely {},
    ReturnOfModding {
        files: Vec<&'a str>,
    },
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
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

    pub const fn separated(name: &'a str, target: &'a str) -> Self {
        Self::new(name, target, SubdirMode::Separate)
    }

    pub const fn flat_separated(name: &'a str, target: &'a str) -> Self {
        Self::new(name, target, SubdirMode::SeparateFlatten)
    }

    pub const fn tracked(name: &'a str, target: &'a str) -> Self {
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

impl ModLoader<'_> {
    pub fn as_str(&self) -> &'static str {
        match &self.kind {
            ModLoaderKind::BepInEx { .. } => "BepInEx",
            ModLoaderKind::MelonLoader { .. } => "MelonLoader",
            ModLoaderKind::Northstar {} => "Northstar",
            ModLoaderKind::GDWeave {} => "GDWeave",
            ModLoaderKind::Shimloader {} => "Shimloader",
            ModLoaderKind::Lovely {} => "Lovely",
            ModLoaderKind::ReturnOfModding { .. } => "ReturnOfModding",
        }
    }

    pub fn log_path(&self) -> Option<&str> {
        match &self.kind {
            ModLoaderKind::BepInEx { .. } => Some("BepInEx/LogOutput.log"),
            ModLoaderKind::MelonLoader { .. } => Some("MelonLoader/Latest.log"),
            ModLoaderKind::GDWeave {} => Some("GDWeave/GDWeave.log"),
            ModLoaderKind::Northstar {} => None,
            ModLoaderKind::Shimloader {} => None,
            ModLoaderKind::Lovely {} => Some("mods/lovely/log"),
            ModLoaderKind::ReturnOfModding { .. } => None,
        }
    }

    pub fn mod_config_dir(&self) -> PathBuf {
        match &self.kind {
            ModLoaderKind::BepInEx { .. } => "BepInEx/config",
            ModLoaderKind::MelonLoader { .. } => ".",
            ModLoaderKind::GDWeave {} => "GDWeave/configs",
            ModLoaderKind::Northstar {} => ".",
            ModLoaderKind::Shimloader {} => ".",
            ModLoaderKind::Lovely {} => ".",
            ModLoaderKind::ReturnOfModding { .. } => "ReturnOfModding/config",
        }
        .into()
    }
}

impl ModLoader<'static> {
    pub fn proxy_dll(&'static self) -> Option<&'static str> {
        match &self.kind {
            ModLoaderKind::BepInEx { .. } => Some("winhttp"),
            ModLoaderKind::GDWeave {} => Some("winmm"),
            ModLoaderKind::ReturnOfModding { files } => Some(files[0]),
            _ => None,
        }
    }
}
