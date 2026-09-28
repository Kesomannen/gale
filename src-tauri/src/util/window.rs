pub trait WindowExt {
    fn zoom(&self, factor: f64) -> tauri::Result<()>;
}

impl WindowExt for tauri::WebviewWindow {
    fn zoom(&self, scale_factor: f64) -> tauri::Result<()> {
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

            #[cfg(target_os = "macos")]
            unsafe {
                use objc2_web_kit::WKWebView;
                use tracing::warn;

                // SAFETY: on macOS `inner()` is the WKWebView pointer owned by wry, and
                // `with_webview` runs this closure on the main thread, which is the only
                // thread WKWebView may be used from. The reference does not outlive the
                // closure.
                let webview = &*(webview.inner() as *const WKWebView);

                // `setPageZoom:` is macOS 11.0+ and objc2 does no availability checking,
                // so calling it unguarded on 10.13-10.15 throws an uncatchable
                // unrecognized-selector exception and aborts the process. Probe the
                // method lists up the class chain (wry hands us a WryWebView subclass,
                // and `instance_methods` only lists a class's own methods) rather than
                // `respondsToSelector:`, which would need the `objc2` crate as a direct
                // dependency for `sel!`.
                let supported =
                    std::iter::successors(Some(webview.class()), |class| class.superclass()).any(
                        |class| {
                            class
                                .instance_methods()
                                .iter()
                                .any(|method| method.name().name() == c"setPageZoom:")
                        },
                    );

                if supported {
                    webview.setPageZoom(scale_factor);
                } else {
                    warn!("page zoom requires macOS 11 or newer, ignoring");
                }
            }
        })
    }
}
