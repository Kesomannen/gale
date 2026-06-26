use std::{
    fs::{self, File},
    path::PathBuf,
};

use eyre::{Context, OptionExt, Result};
use serde::Serialize;
use tauri::{AppHandle, Emitter, command};
use tracing::{Level, level_filters::LevelFilter};
use tracing_subscriber::{Registry, filter::Targets, prelude::*};

use crate::util::{self, fs::PathExt};

pub const FILE_NAME: &str = "latest.log";

#[derive(Serialize, Clone)]
struct WebviewError<'a> {
    name: &'a str,
    message: String,
}

/// Emits an error to the webview, causing it to show an error toast and
/// log the message properly to the log file/terminal.
pub fn log_webview_err(name: impl AsRef<str>, error: eyre::Error, app: &AppHandle) {
    app.emit(
        "error",
        WebviewError {
            name: name.as_ref(),
            message: format!("{error:#}"),
        },
    )
    .unwrap_or_else(|err| {
        tracing::warn!("failed to log error to webview:");
        tracing::error!("{:#}", err)
    })
}

fn log_path() -> PathBuf {
    util::path::default_app_data_dir().join(FILE_NAME)
}

pub fn setup() -> Result<()> {
    let path = log_path();
    fs::create_dir_all(path.parent().unwrap()).context("failed to create log directory")?;
    let log_file = File::create(path).context("failed to create log file")?;

    tracing_log::LogTracer::init()?;

    let env_filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .with_env_var("GALE_LOG")
        .from_env_lossy();

    let filter = Targets::new()
        .with_target("tauri_plugin_updater", Level::INFO)
        .with_target("hyper_util::client", Level::INFO)
        .with_target("reqwest::connect", Level::INFO)
        .with_target("reqwest::retry", Level::DEBUG)
        .with_default(Level::TRACE);

    let subscriber = Registry::default()
        .with(filter)
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::fmt::layer().with_writer(log_file));

    tracing::subscriber::set_global_default(subscriber).context("failed to register subscriber")?;

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
    tracing::error!("{}", msg);
}
