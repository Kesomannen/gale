use std::{env, path::Path};

use tracing::{debug, info};

const DISABLE_DMABUF_RENDERER: &str = "WEBKIT_DISABLE_DMABUF_RENDERER";

/// Works around WebKitGTK's DMA-BUF renderer failing on the NVIDIA driver,
/// which makes Gale crash on launch (`Error 71 (Protocol error) dispatching to Wayland display`)
/// or show a blank window (`Failed to create GBM buffer`),
/// see https://github.com/Kesomannen/gale/issues/467.
///
/// A value set by the user is left untouched, so `WEBKIT_DISABLE_DMABUF_RENDERER=0`
/// keeps the renderer enabled.
///
/// # Safety
///
/// Modifies the environment, so it must be called before any other threads are spawned.
pub unsafe fn apply_workarounds() {
    if let Some(value) = env::var_os(DISABLE_DMABUF_RENDERER) {
        info!("{DISABLE_DMABUF_RENDERER} is already set to {value:?}, leaving it as is");
        return;
    }

    // unlike /sys/module/nvidia, this is also visible inside the flatpak sandbox
    if !Path::new("/proc/driver/nvidia/version").exists() {
        debug!("NVIDIA driver not detected, keeping WebKit DMA-BUF renderer enabled");
        return;
    }

    info!(
        "NVIDIA driver detected, disabling WebKit DMA-BUF renderer to prevent crashes on launch. \
         This turns off GPU acceleration of the UI, which may degrade performance. \
         To enable it again, set {DISABLE_DMABUF_RENDERER}=0"
    );

    // SAFETY: guaranteed by the caller
    unsafe { env::set_var(DISABLE_DMABUF_RENDERER, "1") };
}
