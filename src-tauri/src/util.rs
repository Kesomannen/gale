use std::path::Path;

use anyhow::Context;
use chrono::format::Item;
use log::error;
use serde::Serialize;
use tauri::{AppHandle, Manager};

#[derive(Serialize, Clone)]
struct JsError<'a> {
    name: &'a str,
    message: String,
}

pub fn print_err(name: &str, error: &anyhow::Error, handle: &AppHandle) {
    error!("{}: {:#}", name, error);
    let _ = handle.emit_all(
        "error",
        JsError {
            name,
            message: format!("{:#}", error),
        },
    );
}

pub trait IoResultExt<T> {
    fn fs_context(self, op: &str, path: &Path) -> anyhow::Result<T>;
}

impl<T, E> IoResultExt<T> for std::result::Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn fs_context(self, op: &str, path: &Path) -> anyhow::Result<T> {
        self.with_context(|| format!("error while {} (at {})", op, path.display()))
    }
}