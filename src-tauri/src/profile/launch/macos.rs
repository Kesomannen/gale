use std::{
    borrow::Cow,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use eyre::{Context, OptionExt, Result, ensure};
use keyvalues_parser::{Obj, Value, Vdf};

const WRAPPER: &str = include_str!("steam-launch.sh");

// Only modded launches from Gale carry this marker. Launching from Steam itself
// (or Gale's vanilla button) therefore leaves the game command untouched.
pub fn write_wrapper(path: &Path) -> Result<()> {
    fs::create_dir_all(
        path.parent()
            .ok_or_eyre("wrapper has no parent directory")?,
    )?;
    atomic_write(path, WRAPPER.as_bytes())
}

pub fn bepinex_launcher(profile: &Path) -> Result<(PathBuf, &'static str)> {
    let script = ["start_game_bepinex.sh", "run_bepinex.sh"]
        .into_iter()
        .map(|name| profile.join(name))
        .find(|path| path.is_file())
        .ok_or_eyre("macOS BepInEx launch script not found in the active profile")?;
    let library = [
        "doorstop_libs/libdoorstop_x64.dylib",
        "libdoorstop.dylib",
        "doorstop_libs/libdoorstop_arm64.dylib",
    ]
    .into_iter()
    .map(|name| profile.join(name))
    .find(|path| path.is_file())
    .ok_or_eyre("macOS Doorstop library not found in the active profile")?;
    let output = Command::new("/usr/bin/lipo")
        .arg("-archs")
        .arg(library)
        .output()?;
    ensure!(
        output.status.success(),
        "failed to inspect the Doorstop library architecture"
    );
    let arches = String::from_utf8(output.stdout)?;
    // Prefer the established Intel BepInEx stack when available. ARM-only
    // loaders can run natively; the game and managed loader must support ARM too.
    let arch = if arches.split_whitespace().any(|arch| arch == "x86_64") {
        "x86_64"
    } else {
        ensure!(
            arches.split_whitespace().any(|arch| arch == "arm64"),
            "unsupported Doorstop architecture"
        );
        "arm64"
    };
    Ok((script, arch))
}

pub fn prepare_steam_launch(app_id: u32, wrapper: &Path) -> Result<()> {
    let steam = steamlocate::SteamDir::locate().context("failed to locate Steam")?;
    let logins = fs::read_to_string(steam.path().join("config/loginusers.vdf"))?;
    let account = latest_account(&logins)?;
    let config = steam
        .path()
        .join(format!("userdata/{account}/config/localconfig.vdf"));
    let original = fs::read_to_string(&config).context("failed to read Steam launch options")?;
    let Some(updated) = update_launch_options(&original, app_id, wrapper)? else {
        return Ok(());
    };

    // Steam owns an in-memory copy of this file and overwrites external edits.
    // Setup must finish before starting Steam; subsequent launches need no edits.
    let running = Command::new("/usr/bin/pgrep")
        .args(["-x", "steam_osx"])
        .status()?;
    ensure!(
        running.code() == Some(1),
        "Quit Steam, then launch the modded game from Gale again to set up macOS mod loading. Steam will reopen automatically."
    );

    let backup = config.with_extension(format!("vdf.gale-{app_id}.backup"));
    if !backup.exists() {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&backup)?;
        file.set_permissions(fs::metadata(&config)?.permissions())?;
        file.write_all(original.as_bytes())?;
        file.sync_all()?;
    }
    ensure!(
        fs::read_to_string(&config)? == original,
        "Steam configuration changed during setup; please retry with Steam closed"
    );
    atomic_write(&config, updated.as_bytes())
}

fn atomic_write(path: &Path, contents: &[u8]) -> Result<()> {
    let mut file =
        tempfile::NamedTempFile::new_in(path.parent().ok_or_eyre("file has no parent directory")?)?;
    if path.exists() {
        file.as_file()
            .set_permissions(fs::metadata(path)?.permissions())?;
    }
    file.write_all(contents)?;
    file.as_file().sync_all()?;
    file.persist(path)?;
    Ok(())
}

fn parse_vdf(text: &str) -> Result<Vdf<'_>> {
    let parsed = keyvalues_parser::parse(text)?;
    ensure!(
        parsed.bases.is_empty(),
        "Steam configuration contains unsupported #base directives"
    );
    Ok(Vdf::from(parsed))
}

fn latest_account(text: &str) -> Result<u32> {
    let vdf = parse_vdf(text)?;
    let users = vdf
        .value
        .get_obj()
        .ok_or_eyre("invalid Steam login users")?;
    let account = users
        .iter()
        .filter_map(|(id, values)| {
            let user = values.first()?.get_obj()?;
            let timestamp = user
                .iter()
                .find(|(key, _)| key.eq_ignore_ascii_case("Timestamp"))?
                .1
                .first()?
                .get_str()?
                .parse::<u64>()
                .ok()?;
            Some((timestamp, id.parse::<u64>().ok()?))
        })
        .max_by_key(|(timestamp, _)| *timestamp)
        .ok_or_eyre("no Steam account found; sign in to Steam first")?;
    // Steam stores userdata by the low 32 bits of the SteamID64.
    Ok((account.1 & u64::from(u32::MAX)) as u32)
}

fn object_at_mut<'a, 'b>(mut value: &'b mut Value<'a>, path: &[&str]) -> Result<&'b mut Obj<'a>> {
    for key in path {
        let object = value
            .get_mut_obj()
            .ok_or_eyre("invalid Steam configuration object")?;
        let actual_key = object
            .keys()
            .find(|candidate| candidate.eq_ignore_ascii_case(key))
            .cloned()
            .unwrap_or_else(|| Cow::Owned((*key).to_string()));
        let values = object
            .entry(actual_key)
            .or_insert_with(|| vec![Value::Obj(Obj::new())]);
        ensure!(
            values.len() == 1,
            "ambiguous Steam configuration key: {key}"
        );
        value = &mut values[0];
    }
    value
        .get_mut_obj()
        .ok_or_eyre("invalid Steam app configuration")
}

fn update_launch_options(text: &str, app_id: u32, wrapper: &Path) -> Result<Option<String>> {
    let mut vdf = parse_vdf(text).context("failed to parse Steam configuration")?;
    ensure!(
        vdf.key.eq_ignore_ascii_case("UserLocalConfigStore"),
        "unexpected Steam configuration root"
    );
    let id = app_id.to_string();
    let app = object_at_mut(&mut vdf.value, &["Software", "Valve", "Steam", "apps", &id])?;
    let key = app
        .keys()
        .find(|key| key.eq_ignore_ascii_case("LaunchOptions"))
        .cloned()
        .unwrap_or(Cow::Borrowed("LaunchOptions"));
    let current = match app.get(&key) {
        Some(values) => {
            ensure!(values.len() == 1, "ambiguous Steam launch options");
            values[0]
                .get_str()
                .ok_or_eyre("invalid Steam launch options")?
        }
        None => "",
    };
    let prefix = shell_words::join([
        "/bin/sh",
        wrapper.to_str().ok_or_eyre("wrapper path is not UTF-8")?,
    ]);
    let updated = wrap_options(current, &prefix)?;
    if updated == current {
        return Ok(None);
    }
    app.insert(key, vec![Value::Str(Cow::Owned(updated))]);
    // Validate the serialization before replacing Steam's file.
    let rendered = vdf.to_string();
    ensure!(
        parse_vdf(&rendered)? == vdf,
        "Steam configuration failed round-trip validation"
    );
    Ok(Some(rendered))
}

fn wrap_options(current: &str, prefix: &str) -> Result<String> {
    let managed = format!("{prefix} %command%");
    if current.contains(&managed) {
        return Ok(current.to_string());
    }
    ensure!(
        current.matches("%command%").count() <= 1,
        "Steam launch options contain multiple %command% placeholders"
    );
    if current.contains("%command%") {
        Ok(current.replacen("%command%", &managed, 1))
    } else if current.trim().is_empty() {
        Ok(managed)
    } else {
        Ok(format!("{managed} {current}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_user_options_and_is_idempotent() {
        for options in [
            "",
            "-console",
            "env FOO=bar %command% -console",
            "custom-wrapper %command%",
        ] {
            let wrapped =
                wrap_options(options, "/bin/sh '/path with spaces/steam-launch.sh'").unwrap();
            assert_eq!(
                wrap_options(&wrapped, "/bin/sh '/path with spaces/steam-launch.sh'").unwrap(),
                wrapped
            );
            if !options.is_empty() {
                assert_eq!(
                    wrapped.replace(
                        "/bin/sh '/path with spaces/steam-launch.sh' %command%",
                        "%command%"
                    ),
                    if options.contains("%command%") {
                        options.to_string()
                    } else {
                        format!("%command% {options}")
                    }
                );
            }
        }
        assert!(wrap_options("%command% %command%", "/bin/sh wrapper").is_err());
    }

    #[test]
    fn config_preserves_unrelated_values_and_handles_key_case() {
        let text = r#""UserLocalConfigStore" { "unrelated" "quotes \" and \\ backslash" "Software" { "valve" { "Steam" { "apps" { "892970" { "LaunchOptions" "-console" "other" "unchanged" } "42" { "LaunchOptions" "keep" } } } } } }"#;
        let updated = update_launch_options(text, 892970, Path::new("/tmp/Gale's wrapper.sh"))
            .unwrap()
            .unwrap();
        let mut original = parse_vdf(text).unwrap();
        let mut parsed = parse_vdf(&updated).unwrap();
        object_at_mut(
            &mut original.value,
            &["Software", "Valve", "Steam", "apps", "892970"],
        )
        .unwrap()
        .remove("LaunchOptions");
        object_at_mut(
            &mut parsed.value,
            &["Software", "Valve", "Steam", "apps", "892970"],
        )
        .unwrap()
        .remove("LaunchOptions");
        assert_eq!(parsed, original);
        assert!(
            update_launch_options(&updated, 892970, Path::new("/tmp/Gale's wrapper.sh"))
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn vanilla_wrapper_preserves_arguments() {
        let dir = tempfile::tempdir().unwrap();
        let wrapper = dir.path().join("wrapper.sh");
        write_wrapper(&wrapper).unwrap();
        let output = Command::new("/bin/sh")
            .arg(wrapper)
            .args([
                "/usr/bin/printf",
                "%s\\n",
                "spaces and 'quotes'",
                "-console",
            ])
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            "spaces and 'quotes'\n-console\n"
        );
    }

    #[test]
    fn modded_wrapper_selects_profile_and_removes_marker() {
        let dir = tempfile::tempdir().unwrap();
        let wrapper = dir.path().join("wrapper.sh");
        let loader = dir.path().join("Profile's loader.sh");
        write_wrapper(&wrapper).unwrap();
        fs::write(&loader, "printf 'loader\\n'\nexec \"$@\"\n").unwrap();
        let arch = if cfg!(target_arch = "aarch64") {
            "arm64"
        } else {
            "x86_64"
        };
        let output = Command::new("/bin/sh")
            .arg(&wrapper)
            .args([
                "/usr/bin/printf",
                "%s\\n",
                "before marker",
                "--gale-bepinex",
            ])
            .arg(&loader)
            .arg(arch)
            .arg("after marker")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            "loader\nbefore marker\nafter marker\n"
        );
        assert!(
            !Command::new("/bin/sh")
                .arg(wrapper)
                .args(["/usr/bin/true", "--gale-bepinex"])
                .status()
                .unwrap()
                .success()
        );
    }

    #[test]
    fn selects_most_recent_account() {
        let text = r#""users" { "76561198030445521" { "Timestamp" "200" } "76561199660605927" { "Timestamp" "100" } }"#;
        assert_eq!(latest_account(text).unwrap(), 70179793);
        assert!(latest_account(r#""users" {}"#).is_err());
    }

    #[test]
    fn selects_most_recent_account_regardless_of_timestamp_case() {
        for key in ["timestamp", "Timestamp", "TIMESTAMP"] {
            let text = format!(
                r#""users" {{ "76561198030445521" {{ "{key}" "200" }} "76561199660605927" {{ "Timestamp" "100" }} }}"#
            );
            assert_eq!(latest_account(&text).unwrap(), 70179793);
        }
    }
}
