use gale_core::prelude::*;
use tauri::{
    generate_handler,
    plugin::{Builder, TauriPlugin},
};

mod commands;
mod export;
mod import;
mod modpack;
mod r2modman;

pub fn init() -> TauriPlugin<tauri::Wry> {
    Builder::new("gale-io")
        .setup(|app, _| Ok(()))
        .invoke_handler(generate_handler![])
        .build()
}
