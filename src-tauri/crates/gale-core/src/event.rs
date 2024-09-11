use crate::error::Result;
use serde::Serialize;
use serde_json::json;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct LoadingBar<'a, T: Serialize> {
    id: Uuid,
    title: &'a str,
    #[serde(skip)]
    app: &'a AppHandle,
    kind: T,
}

#[derive(Debug, Serialize)]
pub struct Indeterminate;

#[derive(Debug, Serialize)]
pub struct Determinate {
    progress: f32,
}

#[derive(Debug, Serialize)]
pub struct Bounded {
    progress: f32,
    total: usize,
}

impl<'a, T: Serialize> LoadingBar<'a, T> {
    fn create(app: &'a AppHandle, title: &'a str, kind: T) -> Result<Self> {
        let this = Self {
            id: Uuid::new_v4(),
            title,
            app,
            kind,
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

impl<'a> LoadingBar<'a, Indeterminate> {
    pub fn indeterminate(title: &'a str, app: &'a AppHandle) -> Result<Self> {
        Self::create(app, title, Indeterminate)
    }

    pub fn update(&self, message: Option<&str>) -> Result<()> {
        self.emit(message, None)
    }
}

impl<'a> LoadingBar<'a, Determinate> {
    pub fn determinate(title: &'a str, app: &'a AppHandle) -> Result<Self> {
        Self::create(app, title, Determinate { progress: 0.0 })
    }

    pub fn update(&mut self, message: Option<&str>, progress: f32) -> Result<()> {
        self.kind.progress = progress;
        self.emit(message, Some(progress))
    }
}

impl<T: Serialize> Drop for LoadingBar<'_, T> {
    fn drop(&mut self) {
        if let Err(err) = self._close() {
            log::warn!("failed to close loading bar: {:#}", err);
        }
    }
}
