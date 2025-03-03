use reqwest::Method;
use tauri::State;

use super::SyncProfile;
use crate::{
    profile::ModManager,
    supabase,
    util::cmd::{Result, StateMutex},
    NetworkClient,
};

#[tauri::command]
pub async fn connect(id: String, client: State<'_, NetworkClient>) -> Result<()> {
    let response: Vec<SyncProfile> = supabase::request(Method::GET, "/profile")
        .query("select", "*")
        .query("id", format!("eq.{id}"))
        .send(&client.0)
        .await?;

    dbg!(response);

    Ok(())
}

#[tauri::command]
pub async fn sync_profile(
    profile_index: usize,
    client: State<'_, NetworkClient>,
    manager: StateMutex<'_, ModManager>,
) -> Result<()> {
    let name = {
        let mut manager = manager.lock().unwrap();

        let mut profile = manager.active_profile_mut();
        profile.name
    };

    let response = supabase::request(Method::POST, path)
}
