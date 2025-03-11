use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use eyre::{bail, Context, OptionExt, Result};
use log::{info, warn};

use crate::{
    game::{ModLoader, ModLoaderKind},
    util::error::IoResultExt,
};

pub fn add_args(command: &mut Command, profile_dir: &Path, mod_loader: &ModLoader) -> Result<()> {
    match &mod_loader.kind {
        ModLoaderKind::BepInEx { .. } => add_bepinex_args(command, profile_dir),
        ModLoaderKind::MelonLoader { .. } => add_melon_loader_args(command, profile_dir),
        ModLoaderKind::Northstar {} => add_northstar_args(command, profile_dir),
        ModLoaderKind::GDWeave {} => add_gd_weave_args(command, profile_dir),
        ModLoaderKind::Shimloader {} => add_shimloader_args(command, profile_dir),
        ModLoaderKind::Lovely {} => add_lovely_args(command, profile_dir),
        ModLoaderKind::ReturnOfModding { .. } => add_return_of_modding_args(command, profile_dir),
    }
}

fn add_bepinex_args(command: &mut Command, profile_dir: &Path) -> Result<()> {
    let (enable_prefix, target_prefix) = doorstop_args(profile_dir)?;
    let preloader_path = bepinex_preloader_path(profile_dir)?;

    command
        .args([enable_prefix, "true", target_prefix])
        .arg(preloader_path);

    Ok(())
}

fn bepinex_preloader_path(profile_dir: &Path) -> Result<PathBuf> {
    let mut core_dir = profile_dir.to_path_buf();

    core_dir.push("BepInEx");
    core_dir.push("core");

    const PRELOADER_NAMES: &[&str] = &[
        "BepInEx.Unity.Mono.Preloader.dll",
        "BepInEx.Unity.IL2CPP.dll",
        "BepInEx.Preloader.dll",
        "BepInEx.IL2CPP.dll",
    ];

    let result = core_dir
        .read_dir()
        .context("failed to read BepInEx core directory. Is BepInEx installed?")?
        .filter_map(|entry| entry.ok())
        .find(|entry| {
            let file_name = entry.file_name();
            PRELOADER_NAMES.iter().any(|name| file_name == **name)
        })
        .ok_or_eyre("BepInEx preloader not found. Is BepInEx installed?")?
        .path();

    Ok(result)
}

fn doorstop_args(profile_dir: &Path) -> Result<(&'static str, &'static str)> {
    let path = profile_dir.join(".doorstop_version");

    let version = if path.exists() {
        let version = fs::read_to_string(&path)
            .fs_context("reading version file", &path)?
            .split('.') // read only the major version number
            .next()
            .and_then(|str| str.parse().ok())
            .ok_or_eyre("invalid version format")?;

        info!("doorstop version read: {}", version);
        version
    } else {
        warn!(".doorstop_version file is missing, defaulting to 3");
        3
    };

    match version {
        3 => Ok(("--doorstop-enable", "--doorstop-target")),
        4 => Ok(("--doorstop-enabled", "--doorstop-target-assembly")),
        vers => bail!("unsupported doorstop version: {}", vers),
    }
}

fn add_melon_loader_args(command: &mut Command, profile_dir: &Path) -> Result<()> {
    command.arg("--melonloader.basedir").arg(profile_dir);

    let mono_assembly_exists = profile_dir
        .join("MelonLoader/Managed/Assembly-CSharp.dll")
        .exists();
    let il2cpp_assembly_exists = profile_dir
        .join("MelonLoader/Il2CppAssemblies/Assembly-CSharp.dll")
        .exists();

    if !mono_assembly_exists && !il2cpp_assembly_exists {
        command.arg("--melonloader.agfregenerate");
    }

    Ok(())
}

fn add_northstar_args(command: &mut Command, profile_dir: &Path) -> Result<()> {
    let path = profile_dir.join("R2Northstar");
    let path = path
        .to_str()
        .ok_or_eyre("profile path is not valid UTF-8")?;

    command.arg("-northstar").arg(format!("-profile={}", path));

    Ok(())
}

fn add_gd_weave_args(command: &mut Command, profile_dir: &Path) -> Result<()> {
    let path = profile_dir.join("GDWeave");
    let path = path
        .to_str()
        .ok_or_eyre("profile path is not valid UTF-8")?;

    command.arg(format!("--gdweave-folder-override={}", path));

    Ok(())
}

fn add_shimloader_args(command: &mut Command, profile_dir: &Path) -> Result<()> {
    let path = profile_dir.join("shimloader");

    command
        .arg("--mod-dir")
        .arg(path.join("mod"))
        .arg("--pak-dir")
        .arg(path.join("pak"))
        .arg("--cfg-dir")
        .arg(path.join("cfg"));

    Ok(())
}

fn add_lovely_args(command: &mut Command, profile_dir: &Path) -> Result<()> {
    let path = profile_dir.join("mods");
    command.arg("--mod-dir").arg(path);

    Ok(())
}

fn add_return_of_modding_args(command: &mut Command, profile_dir: &Path) -> Result<()> {
    command.arg("--rom_modding_root_folder").arg(profile_dir);

    Ok(())
}
