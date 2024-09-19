use crate::error::Result;
use serde_json::json;
use std::borrow::Cow;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

pub struct LoadingBar<'a> {
    id: Uuid,
    app: &'a AppHandle,
}

impl<'a> LoadingBar<'a> {
    pub fn new(title: &str, app: &'a AppHandle) -> Self {
        let id = Uuid::new_v4();
        app.emit("loading-bar-create", &json!({ "id": id, "title": title }))
            .ok();
        Self { id, app }
    }

    pub fn update(&self) -> LoadingBarUpdate {
        LoadingBarUpdate {
            bar: self,
            text: None,
            progress: None,
        }
    }

    pub fn close(mut self) -> Result<()> {
        self._close()
    }

    fn _close(&mut self) -> Result<()> {
        self.app
            .emit("loading-bar-close", &json!({ "id": self.id }))?;

        Ok(())
    }
}

impl Drop for LoadingBar<'_> {
    fn drop(&mut self) {
        if let Err(err) = self._close() {
            log::warn!("failed to close loading bar: {:#}", err);
        }
    }
}

pub struct LoadingBarUpdate<'a, 'b, 'c> {
    bar: &'a LoadingBar<'b>,
    text: Option<Cow<'c, str>>,
    progress: Option<f32>,
}

impl<'a, 'b, 'c> LoadingBarUpdate<'a, 'b, 'c> {
    pub fn set_text(mut self, text: impl Into<Cow<'c, str>>) -> Self {
        self.text = Some(text.into());
        self
    }

    pub fn set_progress(mut self, progress: f32) -> Self {
        self.progress = Some(progress);
        self
    }

    pub fn send(mut self) -> Result<()> {
        self._send()
    }

    fn _send(&mut self) -> Result<()> {
        self.bar.app.emit(
            "loading-bar-update",
            &json!({
                "id": self.bar.id,
                "text": self.text,
                "progress": self.progress,
            }),
        )?;

        Ok(())
    }
}

impl Drop for LoadingBarUpdate<'_, '_, '_> {
    fn drop(&mut self) {
        if let Err(err) = self._send() {
            log::warn!("failed to send loading bar update: {:#}", err);
        }
    }
}
