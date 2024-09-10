use state::AppState;
use tauri::{
    generate_handler,
    plugin::{Builder, TauriPlugin},
    Manager,
};

pub mod error;
pub mod prelude;
pub mod state;

pub trait ResultExt<T, E> {
    fn map_into<U, V>(self) -> Result<U, V>
    where
        U: From<T>,
        V: From<E>;
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    fn map_into<U, V>(self) -> Result<U, V>
    where
        U: From<T>,
        V: From<E>,
    {
        match self {
            Ok(value) => Ok(U::from(value)),
            Err(error) => Err(V::from(error)),
        }
    }
}

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
