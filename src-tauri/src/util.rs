use std::path::Path;

use serde::Serialize;
use tauri::{AppHandle, Manager};
use anyhow::Context;

#[derive(Serialize, Clone)]
struct JsError<'a> {
    name: &'a str,
    message: String,
}

pub fn print_err(context: &str, error: &anyhow::Error, handle: &AppHandle) {
    eprintln!("{}: {:#}", context, error);
    let _ = handle.emit_all("error", JsError {
        name: context,
        message: format!("{:#}", error),
    });
}

pub trait IoResultExt<T> {
    fn fs_context(self, op: &str, path: &Path) -> anyhow::Result<T>;
}

impl<T, E> IoResultExt<T> for std::result::Result<T, E>
where E: std::error::Error + Send + Sync + 'static {
    fn fs_context(self, op: &str, path: &Path) -> anyhow::Result<T> {
        self.with_context(|| format!("error while {} (at {})", op, path.display()))
    }
}
