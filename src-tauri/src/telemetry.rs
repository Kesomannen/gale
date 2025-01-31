use std::sync::Mutex;

use eyre::{Context, Result};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::{AppHandle, Manager};
use uuid::Uuid;

use crate::{
    prefs::Prefs,
    util::{self, fs::JsonStyle},
    NetworkClient,
};

const PROJECT_URL: &str = "https://phpkxfkbquscgqvhtuuv.supabase.co";
const ANON_KEY: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6InBocGt4ZmticXVzY2dxdmh0dXV2Iiwicm9sZSI6ImFub24iLCJpYXQiOjE3MzcyODAzNDgsImV4cCI6MjA1Mjg1NjM0OH0._eOEhNdG5dIpLnArUcTiicwuxv-hYQlZSSqc06-Aj0k";

pub async fn send_app_start_event(app: AppHandle) {
    let prefs = app.state::<Mutex<Prefs>>();

    if !prefs.lock().unwrap().send_telemetry() {
        info!("telemetry is disabled");
        return;
    }

    debug!("sending app_start telemetry event");

    let client = &app.state::<NetworkClient>().inner().0;
    let data = match read_save_data() {
        Ok(data) => data,
        Err(err) => {
            error!("failed to read telemetry save data: {:#}", err);
            return;
        }
    };

    let url = format!("{}/rest/v1/rpc/send_event", PROJECT_URL);

    let payload = json!({
        "kind": "app_start",
        "user_id": data.user_id
    });

    match send_request(url, payload, client).await {
        Ok(_) => debug!("successfully sent telemetry"),
        Err(err) => error!("failed to send telemetry: {:#}", err),
    }
}

async fn send_request(
    url: String,
    payload: serde_json::Value,
    client: &reqwest::Client,
) -> Result<()> {
    client
        .post(url)
        .bearer_auth(ANON_KEY)
        .header("apikey", ANON_KEY)
        .json(&payload)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SaveData {
    user_id: Uuid,
}

fn read_save_data() -> Result<SaveData> {
    let path = util::path::default_app_data_dir().join("telementary.json"); // old typo -- oops

    if path.exists() {
        util::fs::read_json(path).context("failed to read save file")
    } else {
        let data = SaveData {
            user_id: Uuid::new_v4(),
        };

        info!(
            "telemetry save data does not exist, creating new user with id {}",
            data.user_id
        );

        util::fs::write_json(path, &data, JsonStyle::Pretty)
            .context("failed to write save file")?;

        Ok(data)
    }
}
