use eyre::Result;
use tauri::window::Color;

#[cfg(target_os = "linux")]
pub async fn system_accent() -> Result<Option<Color>> {
    use ashpd::desktop::settings::Settings;

    let settings = Settings::new().await?;
    let color: ashpd::desktop::Color = settings.accent_color().await?;
    let color = Color(
        (color.red() * 255.0) as u8,
        (color.green() * 255.0) as u8,
        (color.blue() * 255.0) as u8,
        0,
    );
    Ok(Some(color))
}

#[cfg(target_os = "windows")]
pub async fn system_accent() -> Result<Option<Color>> {
    Ok(None)
}
