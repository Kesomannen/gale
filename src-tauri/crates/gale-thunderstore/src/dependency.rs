use gale_core::prelude::*;
use uuid::Uuid;

pub async fn all(version_uuid: Uuid, state: &AppState) -> Result<Vec<Uuid>> {
    Ok(vec![version_uuid])
}
