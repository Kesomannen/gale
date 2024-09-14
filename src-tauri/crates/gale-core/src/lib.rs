use state::AppState;
use tauri::{
    generate_handler,
    plugin::{Builder, TauriPlugin},
    Manager,
};

mod commands;
pub mod community;
pub mod error;
pub mod event;
pub mod state;
pub mod util;

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
        .invoke_handler(generate_handler![commands::get_communities])
        .build()
}

pub mod prelude {
    pub use crate::{
        error::{CmdError, CmdResult, Error, Result},
        event::LoadingBar,
        state::{AppState, ManagerExt},
        util::PathExt,
        ResultExt,
    };
}
