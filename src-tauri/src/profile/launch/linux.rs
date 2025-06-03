use std::{fs, path::Path};

use eyre::Result;
use itertools::Itertools;
use tracing::{debug, info, trace};

use crate::util::error::IoResultExt;

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

pub fn ensure_wine_override(steam_id: u64, proxy_dll: &str, game_dir: &Path) -> Result<()> {
    let wine_reg_path = game_dir
        .parent() // common
        .unwrap()
        .parent() // steamapps
        .unwrap()
        .join("compatdata")
        .join(steam_id.to_string())
        .join("pfx")
        .join("user.reg");

    let text = fs::read_to_string(&wine_reg_path).fs_context("reading wine registry", &wine_reg_path)?;
    let new_text = reg_add_in_section(
        &text,
        r#"[Software\\Wine\\DllOverrides]"#,
        proxy_dll,
        "native,builtin",
    );

    if text == new_text {
        debug!("wine registry is unchanged");
    } else {
        info!("writing to wine registry");
        fs::write(&wine_reg_path, new_text).fs_context("writing wine registry", &wine_reg_path)?;
    }

    Ok(())
}

fn reg_add_in_section(reg: &str, section: &str, key: &str, value: &str) -> String {
    let mut lines = reg.lines().collect_vec();

    let mut begin = 0;
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with(section) {
            begin = i + 2; // Skip timestamp line
            break;
        }
    }

    trace!("section begins at line {}", begin);

    let mut end = begin;
    while end < lines.len() && !lines[end].is_empty() {
        end += 1;
    }

    trace!("section ends at line {}", end);

    for i in begin..end {
        if lines[i].starts_with(&format!("\"{}\"", key)) {
            debug!("found existing key in wine registry, replacing it");

            let line = format!("\"{}\"=\"{}\"", key, value);
            lines[i] = &line;

            return lines.join("\n");
        }
    }

    debug!("adding key to wine registry");

    let line = format!("\"{}\"=\"{}\"", key, value);
    lines.insert(end, &line);
    lines.join("\n")
}
