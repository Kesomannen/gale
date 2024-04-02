use std::{
    collections::HashSet, iter, str::{self, Split}, sync::Mutex, time::{Duration, Instant}
};

use anyhow::{Context, Result};
use indexmap::IndexMap;
use itertools::Itertools;
use serde::Serialize;
use tauri::{AppHandle, Manager};
use uuid::Uuid;

use crate::NetworkClient;

use self::models::{PackageListing, PackageVersion};

pub mod commands;
pub mod models;
pub mod query;

#[derive(Serialize, Debug, Clone)]
pub struct OwnedMod {
    pub package: PackageListing,
    pub version: PackageVersion,
}

impl From<BorrowedMod<'_>> for OwnedMod {
    fn from(borrowed_mod: BorrowedMod) -> Self {
        Self {
            package: borrowed_mod.package.clone(),
            version: borrowed_mod.version.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BorrowedMod<'a> {
    pub package: &'a PackageListing,
    pub version: &'a PackageVersion,
}

impl<'a> From<BorrowedMod<'a>> for (&'a PackageListing, &'a PackageVersion) {
    fn from(borrowed_mod: BorrowedMod<'a>) -> Self {
        (borrowed_mod.package, borrowed_mod.version)
    }
}

impl<'a> From<(&'a PackageListing, &'a PackageVersion)> for BorrowedMod<'a> {
    fn from((package, version): (&'a PackageListing, &'a PackageVersion)) -> Self {
        Self { package, version }
    }
}

pub struct ThunderstoreState {
    pub finished_loading: Mutex<bool>,
    pub packages: Mutex<IndexMap<Uuid, PackageListing>>,
}

impl ThunderstoreState {
    pub fn new() -> Self {
        Self {
            finished_loading: Mutex::new(false),
            packages: Mutex::new(IndexMap::new()),
        }
    }

    pub async fn wait_for_load(&self) {
        while !*self.finished_loading.lock().unwrap() {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}

const URL: &'static str = "https://thunderstore.io/c/lethal-company/api/v1/package/";

pub async fn load_mods(app_handle: AppHandle) -> Result<()> {
    let state = app_handle.state::<ThunderstoreState>();
    let client = &app_handle.state::<NetworkClient>().0;

    let mut response = client.get(URL).send().await?;

    let mut is_first_chunk = true;
    let mut buffer = String::new();
    let mut byte_buffer = Vec::new();
    let mut i = 0;

    let start_time = Instant::now();

    while let Some(chunk) = response.chunk().await? {
        if chunk.is_empty() {
            break;
        }

        byte_buffer.extend_from_slice(&chunk);
        let chunk = str::from_utf8(&byte_buffer);

        if let Err(_) = chunk {
            continue;
        }

        let chunk = chunk.unwrap();
        match is_first_chunk {
            true => {
                is_first_chunk = false;
                buffer.extend(chunk.chars().skip(1)); // remove leading [
            }
            false => buffer.push_str(chunk),
        };

        byte_buffer.clear();

        {
            let mut packages = state.packages.lock().unwrap();
            while let Some(index) = buffer.find("}]},") {
                let (json, _) = buffer.split_at(index + 3);

                let package: PackageListing = serde_json::from_str(json)?;
                packages.insert(package.uuid4.clone(), package);
                
                buffer.replace_range(..index + 4, "");
            }

            if i % 100 == 0 {
                let _ = app_handle.emit_all("status_update", Some(format!("Fetching mods from Thunderstore... {} of unknown", packages.len())));
            }
        }

        i += 1;
    }

    println!("finished loading mods in {:?}", start_time.elapsed());
    *state.finished_loading.lock().unwrap() = true;

    let _ = app_handle.emit_all("status_update", None::<String>);
    Ok(())
}

pub fn find_package<'a>(
    full_name: &str,
    packages: &'a IndexMap<Uuid, PackageListing>,
) -> Result<&'a PackageListing> {
    packages
        .values()
        .find(|mod_listing| mod_listing.full_name == full_name)
        .with_context(|| format!("package {} not found", full_name))
}

pub fn get_package<'a>(
    uuid: &Uuid,
    packages: &'a IndexMap<Uuid, PackageListing>,
) -> Result<&'a PackageListing> {
    packages
        .get(uuid)
        .with_context(|| format!("package with id {} not found", uuid))
}

pub fn get_mod<'a>(
    package_uuid: &Uuid,
    version_uuid: &Uuid,
    packages: &'a IndexMap<Uuid, PackageListing>,
) -> Result<BorrowedMod<'a>> {
    let package = get_package(package_uuid, packages)?;
    let version = package
        .get_version(version_uuid)
        .with_context(|| format!("version with id {} not found in package {}", version_uuid, package.full_name))?;

    Ok((package, version).into())
}

pub fn latest_versions(
    packages: &IndexMap<Uuid, PackageListing>,
) -> impl Iterator<Item = BorrowedMod> {
    packages.values().map(|package| BorrowedMod {
        package,
        version: &package.versions[0],
    })
}

pub fn resolve_deps<'a>(
    dependency_strings: &'a Vec<String>,
    packages: &'a IndexMap<Uuid, PackageListing>,
) -> impl Iterator<Item = Result<BorrowedMod<'a>>> + 'a {
    let mut unique_map = HashSet::new();
    return inner(dependency_strings, packages)
        .filter_ok(move |dep| unique_map.insert(dep.package.uuid4));

    fn inner<'a>(
        dependency_strings: &'a Vec<String>,
        packages: &'a IndexMap<Uuid, PackageListing>,
    ) -> Box<dyn Iterator<Item = Result<BorrowedMod<'a>>> + 'a> {
        Box::new(
            dependency_strings
                .iter()
                .map(move |dependency| {
                    let dep = resolve_dep(dependency, packages);
    
                    dep.map(|dep| 
                        inner(&dep.version.dependencies, packages)
                            .chain(iter::once(Ok(dep)))
                    )
                })
                .flatten_ok()
                .flatten_ok()
        )
    }
}

pub fn resolve_dep<'a>(
    dependency_string: &String,
    packages: &'a IndexMap<Uuid, PackageListing>,
) -> Result<BorrowedMod<'a>> {
    let split = dependency_string.split('-');

    let (author, name, version) = parts(split)
        .with_context(|| format!("invalid dependency string format {}", dependency_string))?;

    let full_name = format!("{}-{}", author, name);
    let package = find_package(&full_name, &packages)?;
    let version = package
        .get_version_with_num(version)
        .with_context(|| format!("version {} not found in package {}", version, full_name))?;

    return Ok((package, version).into());

    fn parts<'a>(mut s: Split<'a, char>) -> Option<(&'a str, &'a str, &'a str)> {
        let author = s.next()?;
        let name = s.next()?;
        let version = s.next()?;

        Some((author, name, version))
    }
}
