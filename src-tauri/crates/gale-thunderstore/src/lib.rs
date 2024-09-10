use gale_core::prelude::*;
use tauri::{
    generate_handler,
    plugin::{Builder, TauriPlugin},
};
use uuid::uuid;

pub mod api;
mod commands;
mod dependency;
mod fetch;
mod query;

pub fn init() -> TauriPlugin<tauri::Wry> {
    Builder::new("gale-thunderstore")
        .setup(|app, _| {
            let handle = app.to_owned();
            tauri::async_runtime::spawn(async move {
                let state = handle.app_state();
                dependency::all(uuid!("96e1becb-a672-4bea-860e-ddd52f2625b8"), &state).await.unwrap();
                /*
                fetch::fetch_packages(&state, 2)
                    .await
                    .unwrap_or_else(|err| {
                        log::error!("failed to fetch packages: {err:#}");
                    })
                */
            });

            Ok(())
        })
        .invoke_handler(generate_handler![
            commands::query_packages,
            commands::query_package
        ])
        .build()
}
