use std::{str::Split, sync::Mutex};

use itertools::Itertools;
use ordered_hash_map::OrderedHashMap;
use serde::Serialize;
use tauri::{AppHandle, Manager};
use uuid::Uuid;
use anyhow::{Context, Result};

use crate::NetworkClient;

use self::models::{PackageListing, PackageVersion};

pub mod commands;
pub mod query;
pub mod models;

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
    finished_loading: Mutex<bool>,
    pub all_mods: Mutex<OrderedHashMap<Uuid, PackageListing>>,
}

impl ThunderstoreState {
    pub fn new() -> Self {
        Self {
            finished_loading: Mutex::new(false),
            all_mods: Mutex::new(OrderedHashMap::new()),
        }
    }
}

const URL: &'static str = "https://thunderstore.io/c/lethal-company/api/v1/package/";

pub async fn load_mods(app_handle: AppHandle) -> Result<()> {
    let state = app_handle.state::<ThunderstoreState>();
    let client = &app_handle.state::<NetworkClient>().client;

    let response: Vec<PackageListing> = client.get(URL)
        .send().await?
        .json().await?;

    let mut all_mods = state.all_mods.lock().unwrap();
    for mod_listing in response.into_iter() {
        all_mods.insert(mod_listing.uuid4, mod_listing);
    }

    println!("loaded {} mods", all_mods.len());

    *state.finished_loading.lock().unwrap() = true;

    Ok(())
}

pub fn find_package<'a>(
    full_name: &str,
    mod_list: &'a OrderedHashMap<Uuid, PackageListing>,
) -> Option<&'a PackageListing> {
    mod_list.values()
        .find(|mod_listing| mod_listing.full_name == full_name)
}

pub fn resolve_deps_all<'a>(
    dependency_strings: &'a Vec<String>,
    mod_map: &'a OrderedHashMap<Uuid, PackageListing>,
) -> impl Iterator<Item = BorrowedMod<'a>> + 'a {
    return inner(dependency_strings, mod_map).unique_by(|dep| dep.package.uuid4);

    fn inner<'a>(
        dependency_strings: &'a Vec<String>,
        mod_map: &'a OrderedHashMap<Uuid, PackageListing>,
    ) -> Box<dyn Iterator<Item = BorrowedMod<'a>> + 'a> {
        Box::new(
            dependency_strings
                .iter()
                .filter_map(move |dependency| {
                    let dep = resolve_dep(dependency, mod_map).ok()?;

                    Some(inner(&dep.version.dependencies, mod_map).chain(std::iter::once(dep)))
                })
                .flatten(),
        )
    }
}

pub fn resolve_deps<'a>(
    dependency_strings: &'a Vec<String>,
    mod_map: &'a OrderedHashMap<Uuid, PackageListing>,
) -> impl Iterator<Item = Result<BorrowedMod<'a>>> + 'a {
    dependency_strings
        .iter()
        .map(move |dependency| resolve_dep(dependency, mod_map))
}

pub fn resolve_dep<'a>(
    dependency_string: &'a String,
    mod_map: &'a OrderedHashMap<Uuid, PackageListing>,
) -> Result<BorrowedMod<'a>> {
    let mut split = dependency_string.split('-');

    let (author, name, version) = parts(split).context("invalid dependency string format")?;

    let full_name = format!("{}-{}", author, name);
    let package = find_package(&full_name, &mod_map).context("package not found")?;
    let version = package.get_version_with_num(version).context("version not found")?;

    return Ok((package, version).into());

    fn parts<'a>(mut s: Split<'a, char>) -> Option<(&'a str, &'a str, &'a str)> {
        let author = s.next()?;
        let name = s.next()?;
        let version = s.next()?;

        Some((author, name, version))
    }
}
