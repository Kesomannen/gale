pub trait WindowExt {
    fn zoom(&self, factor: f64) -> tauri::Result<()>;
}

impl WindowExt for tauri::WebviewWindow {
    fn zoom(&self, scale_factor: f64) -> tauri::Result<()> {
        self.set_zoom(scale_factor)
    }
}
