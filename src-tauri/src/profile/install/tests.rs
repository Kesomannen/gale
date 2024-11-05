use std::path::PathBuf;

use super::fs::*;
use crate::game::{ModLoader, ModLoaderKind};

fn bepinex() -> ModLoader<'static> {
    ModLoader {
        package_name: None,
        extra_sub_dirs: Vec::new(),
        kind: ModLoaderKind::BepInEx,
    }
}

fn melonloader() -> ModLoader<'static> {
    ModLoader {
        package_name: None,
        extra_sub_dirs: Vec::new(),
        kind: ModLoaderKind::MelonLoader,
    }
}

#[test]
fn check_map_top_level_file_bepinex() {
    test_map_file_default(
        &["manifest.json"],
        "MyMod",
        bepinex(),
        Some(&["BepInEx", "plugins", "MyMod", "manifest.json"]),
    );

    test_map_file_default(
        &["ignored_folder_1", "ignored_folder_2", "hi.txt"],
        "MyMod",
        bepinex(),
        Some(&["BepInEx", "plugins", "MyMod", "hi.txt"]),
    );
}

#[test]
fn check_map_top_level_file_melonloader() {
    test_map_file_default(&["manifest.json"], "MyMod", melonloader(), None);
    test_map_file_default(
        &["a", "b.txt"],
        "MyMod",
        melonloader(),
        Some(&["Mods", "b.txt"]),
    );
}

#[test]
fn check_map_subdir_bepinex() {
    test_map_file_default(
        &["plugins", "MyMod.dll"],
        "Author-Package",
        bepinex(),
        Some(&["BepInEx", "plugins", "Author-Package", "MyMod.dll"]),
    );
    test_map_file_default(
        &["I_Drive/patchers/CustomSongs/nightcall.mp3"],
        "I_Drive",
        bepinex(),
        Some(&[
            "BepInEx",
            "patchers",
            "I_Drive",
            "CustomSongs",
            "nightcall.mp3",
        ]),
    );
}

#[test]
fn check_map_subdir_melonloader() {
    test_map_file_default(
        &["ModManager", "misc_file"],
        "Author-Package",
        melonloader(),
        Some(&["UserData", "ModManager", "Author-Package", "misc_file"]),
    );
}

#[test]
fn check_map_extension_bepinex() {
    test_map_file_default(
        &["hacks", "free_robux.mm.dll"],
        "FreeRobuc",
        bepinex(),
        Some(&["BepInEx", "monomod", "FreeRobuc", "free_robux.mm.dll"]),
    );
}

#[test]
fn check_map_extension_melonloader() {
    test_map_file_default(
        &["hacks", "free_robux.lib.dll"],
        "FreeRobuc",
        melonloader(),
        Some(&["UserLibs", "free_robux.lib.dll"]),
    );
}

#[test]
fn check_map_bepinex() {
    test_map_file_loader(&["icon.png"], None);
    test_map_file_loader(
        &["BepInExPack", ".doorstop-version"],
        Some(&[".doorstop-version"]),
    );
    test_map_file_loader(
        &["BepInExPack", "dotnet", "System.dll"],
        Some(&["dotnet", "System.dll"]),
    );
    test_map_file_loader(
        &["BepInExPack", "BepInEx", "core", "BepInEx.Core.dll"],
        Some(&["BepInEx", "core", "BepInEx.Core.dll"]),
    );
}

fn test_map_file_default(
    relative_path: &[&str],
    full_name: &str,
    mod_loader: ModLoader,
    expected: Option<&[&str]>,
) {
    let relative_path: PathBuf = relative_path.iter().collect();
    let expected = expected.map(|comps| comps.iter().collect::<PathBuf>());
    assert_eq!(
        map_file_default(&relative_path, full_name, &mod_loader).unwrap(),
        expected
    )
}

fn test_map_file_loader(relative_path: &[&str], expected: Option<&[&str]>) {
    let relative_path: PathBuf = relative_path.iter().collect();
    let expected = expected.map(|comps| comps.iter().collect::<PathBuf>());
    assert_eq!(map_file_loader(&relative_path), expected.as_deref())
}
