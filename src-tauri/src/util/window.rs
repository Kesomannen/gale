pub fn zoom(window: &tauri::Window, scale_factor: f64) -> tauri::Result<()> {
    window.with_webview(move |webview| {
        #[cfg(target_os = "linux")]
        {
            use webkit2gtk::traits::WebViewExt;
            webview.inner().set_zoom_level(scale_factor);
        }

        #[cfg(windows)]
        unsafe {
            webview.controller().SetZoomFactor(scale_factor).unwrap();
        }

        #[cfg(target_os = "macos")]
        unsafe {
            let () = msg_send![webview.inner(), setPageZoom: scale_factor];
        }
    })
}
