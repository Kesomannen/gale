use crate::{error::Result, state::AppState};
use serde::{Deserialize, Serialize};
use strum_macros::EnumString;

#[derive(Serialize, Deserialize, Debug, EnumString)]
pub enum ModLoader {
    BepInEx,
}

impl From<String> for ModLoader {
    fn from(s: String) -> Self {
        s.parse().expect("invalid mod loader")
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GameInfo {
    id: i64,
    name: String,
    slug: String,
    mod_loader: ModLoader,
    steam_id: i64,
    override_path: Option<String>,
    is_favorite: bool,
}

pub(crate) async fn get_all(state: &AppState) -> Result<Vec<GameInfo>> {
    let communities = sqlx::query_as!(
        GameInfo,
        "SELECT 
            id,
            name,
            slug,
            mod_loader,
            steam_id,
            override_path,
            is_favorite
        FROM games"
    )
    .fetch_all(&state.db)
    .await?;

    Ok(communities)
}
