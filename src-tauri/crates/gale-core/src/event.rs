use crate::error::Result;
use serde::Serialize;
use serde_json::json;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct LoadingBar<'t, 'a> {
    id: Uuid,
    title: &'t str,
    #[serde(skip)]
    app: &'a AppHandle,
}

impl<'t, 'a> LoadingBar<'t, 'a> {
    pub fn create(title: &'t str, app: &'a AppHandle) -> Result<Self> {
        let this = Self {
            id: Uuid::new_v4(),
            title,
            app,
        };

        app.emit("loading-bar-create", &this)?;

        Ok(this)
    }

    fn emit(&self, message: Option<&str>, progress: Option<f32>) -> Result<()> {
        self.app.emit(
            "loading-bar-update",
            json!({
                "id": self.id,
                "message": message,
                "progress": progress,
            }),
        )?;

        Ok(())
    }

    pub fn close(mut self) -> Result<()> {
        self._close()
    }

    fn _close(&mut self) -> Result<()> {
        self.app.emit("loading-bar-close", self.id)?;

        Ok(())
    }
}

impl<'t, 'a> Drop for LoadingBar<'t, 'a> {
    fn drop(&mut self) {
        if let Err(err) = self._close() {
            log::warn!("failed to close loading bar: {:#}", err);
        }
    }
}
