use std::{
    collections::HashMap,
    path::PathBuf,
};

use serde::Deserialize;

// taken from https://github.com/CosmicHorrorDev/vdf-rs/blob/main/keyvalues-serde/examples/libraryfolders.rs

#[derive(Deserialize, Debug)]
pub struct LibraryFolders {
    pub libraries: Vec<Library>,
}

#[derive(Deserialize, Debug)]
pub struct Library {
    pub path: PathBuf,
    // we don't use any of these, plus it seems from the link above these, these names have
    // changed, we only use the ones we actually need
    // label: String,
    // #[serde(rename = "contentid")]
    // content_id: i128,
    // #[serde(rename = "totalsize")]
    // total_size: u64,
    // update_clean_bytes_tally: u64,
    // time_last_update_verified: u64,
    pub apps: HashMap<u64, u64>,
}
