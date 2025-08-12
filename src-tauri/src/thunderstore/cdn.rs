use chrono::Utc;
use eyre::Result;
use tauri::AppHandle;
use tracing::warn;

use crate::state::ManagerExt;

const CDNS: &[&str] = &[
    "https://gcdn.thunderstore.io",
    "https://hcdn-1.hcdn.thunderstore.io",
];

const TEST_FILE: &str = "healthz";

pub fn default() -> &'static str {
    CDNS[0]
}

pub async fn preferred(app: &AppHandle) -> Option<&'static str> {
    for cdn in CDNS {
        match check_cdn(cdn, app.http()).await {
            Ok(()) => return Some(cdn),
            Err(err) => warn!("could not reach cdn {cdn}: {err:#}"),
        }
    }

    None
}

async fn check_cdn(url: &str, http: &reqwest::Client) -> Result<()> {
    http.get(format!("{url}/{TEST_FILE}"))
        .query(&[("disableCache", Utc::now().timestamp())])
        .header("Cache-Control", "no-cache")
        .header("Pragma", "no-cache")
        .header("Expires", 0)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}
