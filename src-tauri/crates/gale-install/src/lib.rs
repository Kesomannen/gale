use tauri::{
    generate_handler,
    plugin::{Builder, TauriPlugin},
};

pub use thunderstore::install as from_thunderstore;

mod cache;
mod commands;
mod common;
mod local;
mod thunderstore;

pub fn init() -> TauriPlugin<tauri::Wry> {
    Builder::new("gale-install")
        .invoke_handler(generate_handler![])
        .build()
}
