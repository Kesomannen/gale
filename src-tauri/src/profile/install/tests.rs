use crate::game::{ModLoader, ModLoaderKind};
use std::path::PathBuf;

use super::PackageInstaller;

#[test]
fn check_map_top_level_file_bepinex() {
    test_map_mod_loader(
        &["manifest.json"],
        "MyMod",
        bepinex(),
        Some(&["BepInEx", "plugins", "MyMod", "manifest.json"]),
    );

    test_map_mod_loader(
        &["ignored_folder_1", "ignored_folder_2", "hi.txt"],
        "MyMod",
        bepinex(),
        Some(&["BepInEx", "plugins", "MyMod", "hi.txt"]),
    );
}

#[test]
fn check_map_top_level_file_melon_loader() {
    test_map_mod_loader(&["manifest.json"], "MyMod", melon_loader(), None);
    test_map_mod_loader(
        &["a", "b.txt"],
        "MyMod",
        melon_loader(),
        Some(&["Mods", "b.txt"]),
    );
}

#[test]
fn check_map_subdir_bepinex() {
    test_map_mod_loader(
        &["plugins", "MyMod.dll"],
        "Author-Package",
        bepinex(),
        Some(&["BepInEx", "plugins", "Author-Package", "MyMod.dll"]),
    );
    test_map_mod_loader(
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
fn check_map_subdir_melon_loader() {
    test_map_mod_loader(
        &["ModManager", "misc_file"],
        "Author-Package",
        melon_loader(),
        Some(&["UserData", "ModManager", "Author-Package", "misc_file"]),
    );
}

#[test]
fn check_map_extension_bepinex() {
    test_map_mod_loader(
        &["hacks", "free_robux.mm.dll"],
        "FreeRobuc",
        bepinex(),
        Some(&["BepInEx", "monomod", "FreeRobuc", "free_robux.mm.dll"]),
    );
}

#[test]
fn check_map_extension_melon_loader() {
    test_map_mod_loader(
        &["hacks", "free_robux.lib.dll"],
        "FreeRobuc",
        melon_loader(),
        Some(&["UserLibs", "free_robux.lib.dll"]),
    );
}

fn bepinex() -> ModLoader<'static> {
    ModLoader {
        package_name: None,
        kind: ModLoaderKind::BepInEx {
            extra_sub_dirs: Vec::new(),
        },
    }
}

fn melon_loader() -> ModLoader<'static> {
    ModLoader {
        package_name: None,
        kind: ModLoaderKind::MelonLoader {
            extra_sub_dirs: Vec::new(),
        },
    }
}

fn test_map_mod_loader(
    relative_path: &[&str],
    full_name: &str,
    mod_loader: ModLoader,
    expected: Option<&[&str]>,
) {
    test_map_file(
        relative_path,
        full_name,
        mod_loader.installer(full_name),
        expected,
    );
}

fn test_map_file(
    relative_path: &[&str],
    full_name: &str,
    installer: PackageInstaller,
    expected: Option<&[&str]>,
) {
    let relative_path: PathBuf = relative_path.iter().collect();
    let expected = expected.map(|comps| comps.iter().collect::<PathBuf>());
    assert_eq!(
        installer
            .map_file(&relative_path, full_name)
            .unwrap()
            .map(|cow| cow.into_owned()),
        expected
    )
}
