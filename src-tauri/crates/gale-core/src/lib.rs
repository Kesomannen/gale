use state::AppState;
use tauri::{
    generate_handler,
    plugin::{Builder, TauriPlugin},
    Manager,
};

pub mod prelude;
pub mod state;

pub type Error = anyhow::Error;
pub type Result<T> = std::result::Result<T, Error>;

pub fn init() -> TauriPlugin<tauri::Wry> {
    Builder::new("gale-core")
        .setup(|app, _| {
            let state = tauri::async_runtime::block_on(AppState::setup(app))?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(generate_handler![])
        .build()
}
