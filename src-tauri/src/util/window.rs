pub trait WindowExt {
    fn zoom(&self, factor: f64) -> tauri::Result<()>;
}

impl WindowExt for tauri::WebviewWindow {
    fn zoom(&self, scale_factor: f64) -> tauri::Result<()> {
        #[cfg(target_os = "macos")]
        return self.set_zoom(scale_factor);

        #[cfg(not(target_os = "macos"))]
        self.with_webview(move |webview| {
            #[cfg(target_os = "linux")]
            {
                use webkit2gtk::WebViewExt;
                webview.inner().set_zoom_level(scale_factor);
            }

            #[cfg(windows)]
            unsafe {
                webview.controller().SetZoomFactor(scale_factor).unwrap();
            }
        })
    }
}
