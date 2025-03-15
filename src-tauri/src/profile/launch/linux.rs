use std::{fs, path::Path};

use eyre::{ContextCompat, Result};

use crate::{prefs::Prefs, util::error::IoResultExt};

pub fn is_proton(game_dir: &Path) -> Result<bool> {
    if game_dir.join(".forceproton").exists() {
        return Ok(true);
    }

    Ok(game_dir
        .read_dir()?
        .filter_map(Result::ok)
        .find(|entry| entry.path().extension().is_some_and(|ext| ext == "exe"))
        .is_some())
}

pub fn ensure_wine_override(steam_id: u32, proxy_dll: &str, prefs: &Prefs) -> Result<()> {
    let mut user_reg_path = prefs
        .steam_library_dir
        .clone()
        .context("steam library setting not set")?;

    if user_reg_path.ends_with("common") {
        user_reg_path.pop();
    }

    if !user_reg_path.ends_with("steamapps") {
        user_reg_path.push("steamapps");
    }

    user_reg_path.push("compatdata");
    user_reg_path.push(steam_id.to_string());
    user_reg_path.push("pfx/user.reg");

    let text = fs::read_to_string(&user_reg_path).fs_context("reading user.reg", &user_reg_path)?;
    let new_text = reg_add_in_section(
        &text,
        r#"[Software\\Wine\\DllOverrides]"#,
        proxy_dll,
        "native,builtin",
    );

    if text != new_text {
        let backup_path = user_reg_path.parent().unwrap().join("user.reg.bak");
        fs::copy(&user_reg_path, &backup_path).fs_context("backing up user.reg", &backup_path)?;

        fs::write(&user_reg_path, new_text).fs_context("writing user.reg", &user_reg_path)?;
    }

    Ok(())
}

fn reg_add_in_section(reg: &str, section: &str, key: &str, value: &str) -> String {
    let mut lines: Vec<&str> = reg.split('\n').collect();

    let mut begin = 0;
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with(section) {
            begin = i + 2; // Skip timestamp line
            break;
        }
    }

    let mut end = begin;
    while end < lines.len() && !lines[end].is_empty() {
        end += 1;
    }

    for i in begin..end {
        if lines[i].starts_with(&format!("\"{}\"", key)) {
            let line = format!("\"{}\"=\"{}\"", key, value);
            lines[i] = &line;
            return lines.join("\n");
        }
    }

    let line = format!("\"{}\"=\"{}\"", key, value);
    lines.insert(end, &line);
    lines.join("\n")
}
