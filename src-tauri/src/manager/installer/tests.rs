use super::*;

#[test]
fn check_map_top_level_file() {
    test_map_file_default(
        &["manifest.json"],
        "MyMod",
        &["BepInEx", "plugins", "MyMod", "manifest.json"],
    );
    test_map_file_default(
        &["ignored_folder_1", "ignored_folder_2", "hi.txt"],
        "MyMod",
        &["BepInEx", "plugins", "MyMod", "hi.txt"],
    );
}

#[test]
fn check_map_subdir() {
    test_map_file_default(
        &["plugins", "MyMod.dll"],
        "MyMod",
        &["BepInEx", "plugins", "MyMod", "MyMod.dll"],
    );
    test_map_file_default(
        &["I_Drive/patchers/CustomSongs/nightcall.mp3"],
        "I_Drive",
        &[
            "BepInEx",
            "patchers",
            "I_Drive",
            "CustomSongs",
            "nightcall.mp3",
        ],
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

fn test_map_file_default(relative_path: &[&str], full_name: &str, expected: &[&str]) {
    let relative_path: PathBuf = relative_path.into_iter().collect();
    let expected: PathBuf = expected.into_iter().collect();
    assert_eq!(
        map_file_default(&relative_path, full_name).unwrap(),
        expected
    )
}

fn test_map_file_bepinex(relative_path: &[&str], expected: Option<&[&str]>) {
    let relative_path: PathBuf = relative_path.into_iter().collect();
    let expected: Option<PathBuf> = expected.map(|comps| comps.into_iter().collect());
    assert_eq!(map_file_bepinex(&relative_path), expected.as_deref())
}
