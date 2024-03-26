use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zip::{write::FileOptions, ZipWriter};

use crate::thunderstore::models::{PackageListing, PackageManifest};

use super::Profile;

pub mod commands;

type Result<T> = anyhow::Result<T>;

#[derive(Serialize, Deserialize, Debug)]
pub struct ModpackArgs {
    pub name: String,
    pub description: String,
    pub version_number: String,
    pub icon: PathBuf,
    pub website_url: Option<String>,
}

impl Profile {
    fn export_pack(
        &self,
        path: &Path,
        args: ModpackArgs,
        packages: &IndexMap<Uuid, PackageListing>,
    ) -> Result<()> {
        let dependencies = self
            .mods
            .iter()
            .map(|p| Ok(p.get(packages)?.version.full_name.clone()))
            .collect::<Result<Vec<_>>>()?;

        let manifest = PackageManifest {
            name: args.name,
            description: args.description,
            version_number: args.version_number,
            website_url: args.website_url.unwrap_or(String::new()),
            dependencies,
            installers: None,
        };

        let options = FileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        let file = File::create(path)?;
        let mut zip = ZipWriter::new(file);

        zip.start_file("manifest.json", options)?;
        let manifest_str = serde_json::to_string_pretty(&manifest)?;
        zip.write_all(manifest_str.as_bytes())?;

        zip.start_file("README.md", options)?;
        let readme = format!("# {}\n\n{}", manifest.name, manifest.description);
        zip.write_all(readme.as_bytes())?;

        zip.start_file("icon.png", options)?;  
        zip.write_all(&fs::read(&args.icon)?)?;

        zip.finish()?;

        Ok(())
    }
}
