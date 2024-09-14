use crate::{error::Result, state::AppState};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CommunityInfo {
    id: i64,
    name: String,
    slug: String,
}

pub(crate) async fn get_all(state: &AppState) -> Result<Vec<CommunityInfo>> {
    let communities = sqlx::query_as!(
        CommunityInfo,
        "SELECT id, name, slug
        FROM communities"
    )
    .fetch_all(&state.db)
    .await?;

    Ok(communities)
}
