use std::{
    fs::{self, File},
    path::PathBuf,
};

use eyre::{Context, OptionExt, Result};
use log::LevelFilter;
use serde::Serialize;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};
use tauri::{command, AppHandle, Emitter};

use crate::util::{self, fs::PathExt};

pub const FILE_NAME: &str = "latest.log";

#[derive(Serialize, Clone)]
struct WebviewError<'a> {
    name: &'a str,
    message: String,
}

/// Emits an error to the webview, causing it to show an error toast and
/// log the message properly to the log file/terminal.
pub fn log_webview_err(name: &str, error: eyre::Error, handle: &AppHandle) {
    handle
        .emit(
            "error",
            WebviewError {
                name,
                message: format!("{:#}", error),
            },
        )
        .unwrap_or_else(|err| {
            log::warn!("failed to log error to webview:");
            log::error!("{:#}", err)
        })
}

fn log_path() -> PathBuf {
    util::path::default_app_data_dir().join(FILE_NAME)
}

pub fn setup() -> Result<()> {
    let path = log_path();
    fs::create_dir_all(path.parent().unwrap()).context("failed to create log directory")?;
    let log_file = File::create(path).context("failed to create log file")?;

    let filter = match cfg!(debug_assertions) {
        true => LevelFilter::Trace,
        false => LevelFilter::Info,
    };

    CombinedLogger::init(vec![
        TermLogger::new(
            filter,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(filter, Config::default(), log_file),
    ])?;

    Ok(())
}

#[command]
pub fn open_gale_log() -> util::cmd::Result<()> {
    let path = log_path()
        .exists_or_none()
        .ok_or_eyre("no log file found")?;

    open::that_detached(&path).context("failed to open log file")?;

    Ok(())
}

#[command]
pub fn log_err(msg: String) {
    log::error!("{}", msg);
}
