use eyre::Result;
use tauri::window::Color;

#[cfg(target_os = "linux")]
pub async fn system_accent() -> Result<Option<Color>> {
    use ashpd::desktop::settings::Settings;

    let settings = Settings::new().await?;
    match settings.accent_color().await {
        Ok(color) => {
            let color = Color(
                (color.red() * 255.0) as u8,
                (color.green() * 255.0) as u8,
                (color.blue() * 255.0) as u8,
                255,
            );
            Ok(Some(color))
        }
        Err(ashpd::Error::Portal(ashpd::PortalError::NotFound(_))) => Ok(None),
        Err(err) => Err(err.into()),
    }
}

#[cfg(target_os = "windows")]
pub async fn system_accent() -> Result<Option<Color>> {
    use windows::UI::ViewManagement::{UIColorType, UISettings};

    let settings = UISettings::new()?;
    let color: windows::UI::Color = settings.GetColorValue(UIColorType::Accent)?;

    Ok(Some(Color(color.R, color.G, color.B, 255)))
}
