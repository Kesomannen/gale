use std::{path::PathBuf, sync::Arc};

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::game::PackageNameMatcher;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JsonModLoader {
    #[serde(default)]
    pub package_name: Option<String>,
    #[serde(flatten)]
    pub kind: JsonModLoaderKind,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "name")]
pub enum JsonModLoaderKind {
    BepInEx {
        #[serde(default, rename = "subdirs")]
        extra_rules: Vec<JsonRule>,
    },
    BepisLoader {
        #[serde(default, rename = "subdirs")]
        extra_rules: Vec<JsonRule>,
    },
    MelonLoader {
        #[serde(default, rename = "subdirs")]
        extra_rules: Vec<JsonRule>,
    },
    Northstar {},
    GDWeave {},
    #[serde(rename_all = "camelCase")]
    Shimloader {
        internal_game_name: String,
    },
    Lovely {},
    ReturnOfModding {
        files: Vec<String>,
    },
}

fn make_dyn_loader<T: loadsmith::ModLoader + 'static>(loader: T) -> Arc<dyn loadsmith::ModLoader> {
    Arc::new(loader)
}

impl JsonModLoader {
    pub fn into_loadsmith(self) -> super::ModLoader {
        use loadsmith::loaders::*;

        let name_matcher = match self.package_name {
            Some(name) => PackageNameMatcher::Exact(name.into()),
            None => match self.kind {
                JsonModLoaderKind::BepInEx { .. } => {
                    PackageNameMatcher::StartsWith("BepInEx-BepInExPack".into())
                }
                JsonModLoaderKind::BepisLoader { .. } => {
                    PackageNameMatcher::StartsWith("ResoniteModding-Bep".into())
                }
                JsonModLoaderKind::MelonLoader { .. } => {
                    PackageNameMatcher::Exact("LavaGang-MelonLoader".into())
                }
                JsonModLoaderKind::Northstar {} => {
                    PackageNameMatcher::Exact("northstar-Northstar".into())
                }
                JsonModLoaderKind::GDWeave {} => PackageNameMatcher::Exact("NotNet-GDWeave".into()),
                JsonModLoaderKind::Shimloader { .. } => {
                    PackageNameMatcher::Exact("Thunderstore-unreal_shimloader".into())
                }
                JsonModLoaderKind::Lovely {} => {
                    PackageNameMatcher::Exact("Thunderstore-lovely".into())
                }
                JsonModLoaderKind::ReturnOfModding { .. } => {
                    PackageNameMatcher::Exact("ReturnOfModding-ReturnOfModding".into())
                }
            },
        };

        let inner = match self.kind {
            JsonModLoaderKind::BepInEx { extra_rules } => make_dyn_loader(
                BepInExBuilder::new()
                    .with_extra_rules(extra_rules.into_iter().map_into().collect())
                    .build(),
            ),
            JsonModLoaderKind::BepisLoader { extra_rules } => make_dyn_loader(
                BepisLoaderBuilder::new()
                    .with_extra_rules(extra_rules.into_iter().map_into().collect())
                    .build(),
            ),
            JsonModLoaderKind::MelonLoader { extra_rules } => make_dyn_loader(
                MelonLoaderBuilder::new()
                    .with_extra_rules(extra_rules.into_iter().map_into().collect())
                    .build(),
            ),
            JsonModLoaderKind::Northstar {} => make_dyn_loader(Northstar::new()),
            JsonModLoaderKind::GDWeave {} => make_dyn_loader(GDWeave::new()),
            JsonModLoaderKind::Shimloader { internal_game_name } => {
                make_dyn_loader(Shimloader::new(internal_game_name))
            }
            JsonModLoaderKind::Lovely {} => make_dyn_loader(Lovely::new()),
            JsonModLoaderKind::ReturnOfModding { .. } => make_dyn_loader(ReturnOfModding::new()),
        };

        super::ModLoader {
            inner,
            name_matcher,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JsonRule {
    pub name: String,
    pub target: String,
    pub mode: JsonRuleMode,
}

impl From<JsonRule> for loadsmith::rule::Rule {
    fn from(value: JsonRule) -> Self {
        loadsmith::rule::Rule::new(value.name, PathBuf::from(value.target), value.mode.into())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum JsonRuleMode {
    Separate,
    SeparateFlatten,
    Track,
    None,
}

impl From<JsonRuleMode> for loadsmith::rule::RuleMode {
    fn from(value: JsonRuleMode) -> Self {
        match value {
            JsonRuleMode::Separate => Self::Separate,
            JsonRuleMode::SeparateFlatten => Self::SeparateFlatten,
            JsonRuleMode::Track => Self::Track,
            JsonRuleMode::None => Self::None,
        }
    }
}

// fn is_loader_package(&self, full_name: &str) -> bool {
//     if let Some(package_name) = self.package_name {
//         full_name == package_name
//     } else {
//         match &self.kind {
//             ModLoaderKind::BepInEx { .. } => full_name.starts_with("BepInEx-BepInExPack"),
//             ModLoaderKind::BepisLoader { .. } => {
//                 full_name == "ResoniteModding-BepisLoader"
//                     || full_name == "ResoniteModding-BepInExRenderer"
//             }
//             ModLoaderKind::MelonLoader { .. } => full_name == "LavaGang-MelonLoader",
//             ModLoaderKind::GDWeave {} => full_name == "NotNet-GDWeave",
//             ModLoaderKind::Northstar {} => full_name == "northstar-Northstar",
//             ModLoaderKind::Shimloader {} => full_name == "Thunderstore-unreal_shimloader",
//             ModLoaderKind::Lovely {} => full_name == "Thunderstore-lovely",
//             ModLoaderKind::ReturnOfModding { .. } => {
//                 full_name == "ReturnOfModding-ReturnOfModding"
//             }
//         }
//     }
// }

//     pub fn proxy_dll(&'static self) -> Option<&'static str> {
//         match &self.kind {
//             ModLoaderKind::BepInEx { .. } => Some("winhttp"),
//             ModLoaderKind::GDWeave {} => Some("winmm"),
//             ModLoaderKind::ReturnOfModding { files } => Some(files[0]),
//             _ => None,
//         }
//     }
// }
