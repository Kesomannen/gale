use std::{fs, io::Cursor, iter, path::Path};

use anyhow::{ensure, Context, Result};
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
    pub profile_mod: ProfileMod,
}

impl From<&BorrowedMod<'_>> for ModDownloadData {
    fn from(borrowed_mod: &BorrowedMod<'_>) -> Self {
        Self {
            name: borrowed_mod.package.full_name.clone(),
            version: borrowed_mod.version.version_number.clone(),
            url: borrowed_mod.version.download_url.clone(),
            profile_mod: ProfileMod::from(borrowed_mod),
        }
    }
}

impl Profile {
    pub fn total_download_size(
        &self,
        config: &Prefs,
        borrowed_mod: BorrowedMod<'_>,
        packages: &IndexMap<Uuid, PackageListing>,
    ) -> Result<u64> {
        Ok(
            thunderstore::resolve_deps(&borrowed_mod.version.dependencies, packages)
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

    pub fn prepare_install<'a>(
        &mut self,
        borrowed_mod: BorrowedMod<'a>,
        packages: &'a IndexMap<Uuid, PackageListing>,
    ) -> Result<Vec<BorrowedMod<'a>>> {
        ensure!(
            !self.has_mod(borrowed_mod.package.uuid4),
            "mod {} already installed",
            borrowed_mod.package.full_name
        );

        println!("preparing to install: {}", borrowed_mod.version.full_name);

        let mut to_install =
            thunderstore::resolve_deps(&borrowed_mod.version.dependencies, packages)
                .filter_ok(|dep| !self.has_mod(dep.package.uuid4))
                .collect::<Result<Vec<_>>>()
                .context("failed to resolve dependencies")?;

        to_install.push(borrowed_mod);
        Ok(to_install)
    }

    pub fn install_from_cache<'a>(
        &mut self,
        mut to_install: Vec<BorrowedMod<'a>>,
        cache_path: &Path,
    ) -> Result<Vec<BorrowedMod<'a>>> {
        let mut i = 0;
        while i < to_install.len() {
            let mod_to_install = &to_install[i];
            let name = &mod_to_install.package.full_name;

            let mut mod_cache_path = cache_path.join(name);
            mod_cache_path.push(&mod_to_install.version.version_number);

            if mod_cache_path.try_exists().unwrap_or(false) {
                println!("installing from cache: {}", name);
                install_mod_from_disk(&mod_cache_path, &self.path, &name)?;
                self.mods.push(ProfileMod::from(mod_to_install));

                to_install.remove(i);
            } else {
                i += 1;
            }
        }

        Ok(to_install)
    }
}

pub async fn install_by_download<'a>(
    to_install: Vec<ModDownloadData>,
    cache_path: &Path,
    target_path: &Path,
    client: &reqwest::Client,
    on_mod_complete: impl Fn(ProfileMod) -> Result<()>,
) -> Result<()> {
    let futures = to_install
        .into_iter()
        .map(|download| download_mod(
            download,
            cache_path,
            target_path,
            client,
            &on_mod_complete
        ));

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
    F: Fn(ProfileMod) -> Result<()>,
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
    on_complete(data.profile_mod)
}

const BEPINEX_NAME: &str = "BepInEx-BepInExPack";

fn install_mod_from_disk(src: &Path, dest: &Path, name: &str) -> Result<()> {
    fn install_other(src: &Path, dest: &Path, name: &str) -> Result<()> {
        let target_path = dest.join("BepInEx");
        let target_plugins_path = target_path.join("plugins").join(name);
        fs::create_dir_all(&target_plugins_path).context("failed to create plugins directory")?;

        for entry in fs::read_dir(src)? {
            let entry_path = entry?.path();
            let entry_name = entry_path.file_name().unwrap();

            if entry_path.is_dir() {
                if entry_name == "config" {
                    let target_path = target_path.join("config");
                    fs::create_dir_all(&target_path)?;
                    fs_util::copy_contents(&entry_path, &target_path)
                        .with_context(|| format!("error while copying config {:?}", entry_path))?;
                } else {
                    let target_path = match entry_name.to_string_lossy().as_ref() {
                        "patchers" | "core" => target_path.join(entry_name).join(name),
                        "plugins" => target_plugins_path.clone(),
                        _ => target_plugins_path.join(entry_name),
                    };

                    fs::create_dir_all(&target_path.parent().unwrap())?;
                    fs_util::copy_dir(&entry_path, &target_path).with_context(|| {
                        format!("error while copying directory {:?}", entry_path)
                    })?;
                }
            } else {
                fs::copy(&entry_path, &target_plugins_path.join(entry_name))
                    .with_context(|| format!("error while copying file {:?}", entry_path))?;
            }
        }

        Ok(())
    }

    fn install_bepinex(src: &Path, dest: &Path) -> Result<()> {
        let target_path = dest.join("BepInEx");

        for entry in fs::read_dir(src)? {
            let entry_path = entry?.path();
            let entry_name = entry_path.file_name().unwrap();

            if entry_path.is_dir() {
                let target_path = target_path.join(entry_name);
                fs::create_dir_all(&target_path)?;

                fs_util::copy_contents(&entry_path, &target_path)
                    .with_context(|| format!("error while copying directory {:?}", entry_path))?;
            } else if entry_name == "winhttp.dll" {
                fs::copy(&entry_path, dest.join(entry_name))
                    .with_context(|| format!("error while copying file {:?}", entry_path))?;
            }
        }

        Ok(())
    }

    match name {
        BEPINEX_NAME => install_bepinex(src, dest),
        _ => install_other(src, dest, name),
    }
}
