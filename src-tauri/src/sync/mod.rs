use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod commands;

#[derive(Debug, Deserialize)]
struct SyncProfile {
    id: Uuid,
    user_id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    name: String,
}

#[derive(Debug, Deserialize)]
struct NewSyncProfile {
    user_id: Uuid,
    name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProfileData {
    id: Uuid,
    last_synced: DateTime<Utc>,
}
