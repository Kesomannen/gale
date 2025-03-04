use std::sync::Mutex;

use eyre::{Context, Result};
use log::{debug, error, info};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::{AppHandle, Manager};
use uuid::Uuid;

use crate::{
    prefs::Prefs,
    supabase,
    util::{self, fs::JsonStyle},
};

pub async fn send_app_start_event(app: AppHandle) {
    let prefs = app.state::<Mutex<Prefs>>();

    if !prefs.lock().unwrap().send_telemetry() {
        info!("telemetry is disabled");
        return;
    }

    debug!("sending app_start telemetry event");

    let data = match read_save_data() {
        Ok(data) => data,
        Err(err) => {
            error!("failed to read telemetry save data: {:#}", err);
            return;
        }
    };

    let payload = json!({
        "kind": "app_start",
        "user_id": data.user_id
    });

    let response = supabase::request(Method::POST, "/rest/v1/rpc/send_event")
        .json_body(payload)
        .send_raw(&app)
        .await;

    match response {
        Ok(_) => debug!("successfully sent telemetry"),
        Err(err) => error!("failed to send telemetry: {:#}", err),
    }
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
