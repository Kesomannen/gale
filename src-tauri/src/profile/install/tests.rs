use std::path::PathBuf;

use super::fs::*;
use crate::game::ModLoader;

fn bepinex() -> ModLoader<'static> {
    ModLoader::BepInEx {
        extra_sub_dirs: Vec::new(),
    }
}

fn melonloader() -> ModLoader<'static> {
    ModLoader::MelonLoader
}

#[test]
fn check_map_top_level_file() {
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

    test_map_file_default(&["manifest.json"], "MyMod", melonloader(), None);
    test_map_file_default(&["a", "b.txt"], "MyMod", melonloader(), None);
}

#[test]
fn check_map_subdir() {
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

    test_map_file_default(
        &["MelonLoader", "Managed", "misc_file"],
        "Author-Package",
        melonloader(),
        Some(&["MelonLoader", "Managed", "misc_file"]),
    );
    test_map_file_default(
        &["Managed", "misc_file"],
        "Author-Package",
        melonloader(),
        Some(&["MelonLoader", "Managed", "misc_file"]),
    );
    test_map_file_default(
        &["random_folder", "UserData", "random_file"],
        "Author-Package",
        melonloader(),
        Some(&["UserData", "random_file"]),
    );
}

#[test]
fn check_map_extension() {
    test_map_file_default(
        &["hacks", "free_robux.mm.dll"],
        "FreeRobuc",
        bepinex(),
        Some(&["BepInEx", "monomod", "FreeRobuc", "free_robux.mm.dll"]),
    );

    test_map_file_default(
        &["my_map.bcm"],
        "MyMaps",
        melonloader(),
        Some(&["UserData", "CustomMaps", "my_map.bcm"]),
    );
    test_map_file_default(
        &["skins/custom_skin.png"],
        "MyMaps",
        melonloader(),
        Some(&["UserData", "CustomSkins", "custom_skin.png"]),
    );
}

#[test]
fn check_map_bepinex() {
    test_map_file_bepinex(&["icon.png"], None);
    test_map_file_bepinex(
        &["BepInExPack", ".doorstop-version"],
        Some(&[".doorstop-version"]),
    );
    test_map_file_bepinex(
        &["BepInExPack", "dotnet", "System.dll"],
        Some(&["dotnet", "System.dll"]),
    );
    test_map_file_bepinex(
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
    let expected = expected.map(|expected| expected.iter().collect::<PathBuf>());
    assert_eq!(
        map_file_default(&relative_path, full_name, &mod_loader).unwrap(),
        expected
    )
}

fn test_map_file_bepinex(relative_path: &[&str], expected: Option<&[&str]>) {
    let relative_path: PathBuf = relative_path.iter().collect();
    let expected: Option<PathBuf> = expected.map(|comps| comps.iter().collect());
    assert_eq!(map_file_bepinex(&relative_path), expected.as_deref())
}
