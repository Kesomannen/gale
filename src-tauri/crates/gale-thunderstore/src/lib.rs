use gale_core::state::AppState;
use tauri::{
    generate_handler,
    plugin::{Builder, TauriPlugin},
    Manager,
};

mod commands;
mod fetch;
mod query;

pub fn init() -> TauriPlugin<tauri::Wry> {
    Builder::new("gale-thunderstore")
        .setup(|app, _| {
            let handle = app.to_owned();
            tauri::async_runtime::spawn(async move {
                let state = handle.state::<AppState>();
                fetch::fetch_packages(&state, 1)
                    .await
                    .unwrap_or_else(|err| {
                        log::error!("failed to fetch packages: {err:#}");
                    })
            });

            Ok(())
        })
        .invoke_handler(generate_handler![commands::query_packages])
        .build()
}
