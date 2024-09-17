use crate::{error::Result, state::AppState};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "platform", rename_all = "camelCase")]
pub enum GamePlatform {
    Steam { id: i64 },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ModLoader {
    BepInEx,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GameInfo {
    id: i64,
    name: String,
    slug: String,
    mod_loader: String,
    platforms: Json<Vec<GamePlatform>>,
    override_path: Option<String>,
    is_favorite: bool,
}

pub(crate) async fn get_all(state: &AppState) -> Result<Vec<GameInfo>> {
    let communities = sqlx::query_as!(
        GameInfo,
        r#"
SELECT 
    id,
    name,
    slug,
    mod_loader,
    platforms AS "platforms: Json<Vec<GamePlatform>>",
    override_path,
    is_favorite
FROM games"#
    )
    .fetch_all(&state.db)
    .await?;

    Ok(communities)
}
