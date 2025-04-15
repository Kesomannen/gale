use reqwest::Method;
use serde_json::json;
use tauri::AppHandle;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{state::ManagerExt, supabase};

pub async fn send_app_start_event(app: AppHandle) {
    if !app.lock_prefs().send_telemetry() {
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

    let response = supabase::request(Method::POST, "/rest/v1/rpc/send_event")
        .json_body(payload)
        .send_raw(&app)
        .await;

    match response {
        Ok(_) => debug!("successfully sent telemetry"),
        Err(err) => error!("failed to send telemetry: {:#}", err),
    }
}
