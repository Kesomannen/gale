use serde_json::json;
use tauri::AppHandle;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::state::ManagerExt;

const PROJECT_URL: &str = "https://phpkxfkbquscgqvhtuuv.supabase.co";
const ANON_KEY: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6InBocGt4ZmticXVzY2dxdmh0dXV2Iiwicm9sZSI6ImFub24iLCJpYXQiOjE3MzcyODAzNDgsImV4cCI6MjA1Mjg1NjM0OH0._eOEhNdG5dIpLnArUcTiicwuxv-hYQlZSSqc06-Aj0k";

pub async fn send_app_start_event(app: AppHandle) {
    if !app.lock_prefs().send_telemetry {
        info!("telemetry is disabled");
        return;
    }

    let user_id = match app.db().user_id() {
        Ok(Some(user_id)) => user_id,
        Ok(None) => {
            info!("user id does not exist, creating new");

            let user_id = Uuid::new_v4();

            app.db().save_user_id(user_id).unwrap_or_else(|err| {
                warn!("failed to save user id to database: {:#}", err);
            });

            user_id
        }
        Err(err) => {
            error!("failed to read telemetry save data: {:#}", err);
            return;
        }
    };

    debug!(
        user_id = user_id.to_string(),
        "sending app_start telemetry event"
    );

    let payload = json!({
        "kind": "app_start",
        "user_id": user_id
    });

    let response = app
        .http()
        .post(format!("{PROJECT_URL}/rest/v1/rpc/send_event"))
        .header("apikey", ANON_KEY)
        .json(&payload)
        .send()
        .await
        .map(|res| res.error_for_status());

    match response {
        Ok(_) => debug!("successfully sent telemetry"),
        Err(err) => error!("failed to send telemetry: {:#}", err),
    }
}
