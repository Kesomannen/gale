use anyhow::Context;
use log::error;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use std::path::Path;

#[derive(Serialize, Clone)]
struct JsError<'a> {
    name: &'a str,
    message: String,
}

pub fn log(name: &str, error: &anyhow::Error, handle: &AppHandle) {
    error!("{}: {:#}", name, error);
    let _ = handle.emit(
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
