use std::{fs, io::Cursor, iter, path::Path};

use anyhow::{bail, Context, Result};
use indexmap::IndexMap;
use itertools::Itertools;
use uuid::Uuid;

use crate::{
    fs_util,
    prefs::Prefs,
    thunderstore::{self, models::PackageListing, BorrowedMod},
};

use super::{Profile, ProfileMod};

pub mod commands;

pub struct ModDownloadData {
    pub name: String,
    pub version: String,
    pub url: String,
}

impl Profile {
    pub fn total_download_size(
        &self,
        config: &Prefs,
        borrowed_mod: BorrowedMod<'_>,
        packages: &IndexMap<Uuid, PackageListing>,
    ) -> Result<u64> {
        Ok(
            thunderstore::resolve_deps_all(&borrowed_mod.version.dependencies, packages)
                .filter_map(|dep| dep.ok())
                .filter(move |dep| !self.has_mod(dep.package.uuid4))
                .chain(iter::once(borrowed_mod))
                .filter(|mod_to_install| {
                    let name = &mod_to_install.package.full_name;

                    let mod_cache_path = config
                        .cache_path
                        .join(name)
                        .join(&mod_to_install.version.version_number);

                    !mod_cache_path.try_exists().unwrap_or(false)
                })
                .map(|mod_to_install| mod_to_install.version.file_size as u64)
                .sum(),
        )
    }

    pub fn install<'a>(
        &mut self,
        borrowed_mod: BorrowedMod<'a>,
        cache_path: &Path,
        packages: &'a IndexMap<Uuid, PackageListing>,
    ) -> Result<(Vec<ModDownloadData>, usize)> {
        if self.has_mod(borrowed_mod.package.uuid4) {
            bail!("mod {} already installed", borrowed_mod.package.full_name);
        }

        println!("preparing to install: {}", borrowed_mod.version.full_name);

        let mut to_install =
            thunderstore::resolve_deps_all(&borrowed_mod.version.dependencies, packages)
                .filter_ok(|dep| !self.has_mod(dep.package.uuid4))
                .collect::<Result<Vec<_>>>()
                .context("failed to resolve dependencies")?;

        to_install.push(borrowed_mod);

        self.mods.extend(to_install.iter().map(|m| ProfileMod {
            package_uuid: m.package.uuid4,
            version_uuid: m.version.uuid4,
        }));

        let total = to_install.len();

        self.install_from_cache(&mut to_install, cache_path)
            .context("failed to install from cache")?;

        Ok((
            to_install
                .iter()
                .map(|m| ModDownloadData {
                    name: m.package.full_name.clone(),
                    version: m.version.version_number.clone(),
                    url: m.version.download_url.clone(),
                })
                .collect::<Vec<_>>(),
            total,
        ))
    }

    fn install_from_cache<'a>(
        &self,
        to_install: &mut Vec<BorrowedMod<'a>>,
        cache_path: &Path,
    ) -> Result<()> {
        let mut i = 0;
        while i < to_install.len() {
            let mod_to_install = &to_install[i];
            let name = &mod_to_install.package.full_name;

            let mut mod_cache_path = cache_path.join(name);
            mod_cache_path.push(&mod_to_install.version.version_number);

            if mod_cache_path.try_exists().unwrap_or(false) {
                println!("installing from cache: {}", name);
                install_mod_from_disk(&mod_cache_path, &self.path, &name)?;

                to_install.remove(i);
            } else {
                i += 1;
            }
        }

        Ok(())
    }
}

pub async fn install_by_download<'a>(
    to_install: Vec<ModDownloadData>,
    cache_path: &Path,
    target_path: &Path,
    client: &reqwest::Client,
    on_mod_complete: impl Fn() -> (),
) -> Result<()> {
    let futures = to_install
        .into_iter()
        .map(|download| download_mod(download, cache_path, target_path, client, &on_mod_complete));

    futures_util::future::join_all(futures)
        .await
        .into_iter()
        .collect::<Result<Vec<_>>>()?;

    return Ok(());
}

async fn download_mod<F>(
    data: ModDownloadData,
    cache_path: &Path,
    target_path: &Path,
    client: &reqwest::Client,
    on_complete: &F,
) -> Result<()>
where
    F: Fn() -> (),
{
    let mod_cache_path = cache_path.join(&data.name).join(&data.version);

    fs::create_dir_all(&mod_cache_path)?;

    println!("downloading: {}", data.url);

    let response = client
        .get(&data.url)
        .send()
        .await
        .context("failed to download mod")?
        .bytes()
        .await
        .context("failed to download mod")?;

    println!("extracting: {}", data.name);

    zip_extract::extract(Cursor::new(response), &mod_cache_path, true)?;

    fs_util::flatten_if_exists(&mod_cache_path.join("BepInExPack"))?;
    fs_util::flatten_if_exists(&mod_cache_path.join("BepInEx"))?;
    fs_util::flatten_if_exists(&mod_cache_path.join("plugins"))?;

    install_mod_from_disk(&mod_cache_path, target_path, &data.name)?;

    println!("done: {}", data.name);
    on_complete();

    Ok(())
}

const BEPINEX_NAME: &str = "BepInEx-BepInExPack";

fn install_mod_from_disk(src: &Path, dest: &Path, name: &str) -> Result<()> {
    let is_bepinex = name == BEPINEX_NAME;

    let target_path = dest.join("BepInEx");
    let target_plugins_path = target_path.join("plugins").join(name);
    if !is_bepinex {
        fs::create_dir_all(&target_plugins_path).context("failed to create plugins directory")?;
    }

    for entry in fs::read_dir(src)? {
        let entry_path = entry?.path();
        let entry_name = entry_path.file_name().unwrap();

        if entry_path.is_dir() {
            if entry_name == "config" {
                let target_path = target_path.join("config");
                fs::create_dir_all(&target_path)?;
                fs_util::copy_contents(&entry_path, &target_path)?;
                continue;
            }

            let target_path = match entry_name.to_str().unwrap() {
                "patchers" | "core" => match is_bepinex {
                    true => target_path.join(entry_name),
                    false => target_path.join(entry_name).join(name),
                },
                _ => target_plugins_path.join(entry_name),
            };

            fs::create_dir_all(&target_path.parent().unwrap())?;
            fs_util::copy_dir(&entry_path, &target_path)
                .with_context(|| format!("error while copying directory {:?}", entry_path))?;
        } else {
            let parent = match is_bepinex {
                true => dest,
                false => &target_plugins_path,
            };

            fs::copy(&entry_path, parent.join(entry_name))
                .with_context(|| format!("error while copying file {:?}", entry_path))?;
        }
    }

    Ok(())
}
