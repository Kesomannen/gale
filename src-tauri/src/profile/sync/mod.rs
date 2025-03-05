use std::{io::Cursor, sync::Mutex};

use chrono::{DateTime, Utc};
use eyre::{bail, Context, ContextCompat, OptionExt, Result};
use log::debug;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use uuid::Uuid;

use crate::{
    prefs::Prefs,
    profile::{install::InstallOptions, ModManager},
    supabase,
};

pub mod commands;

#[derive(Debug, Deserialize)]
struct SyncProfile {
    id: Uuid,
    user_id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    name: String,
}

#[derive(Debug, Serialize)]
struct NewSyncProfile {
    user_id: Uuid,
    name: String,
}

#[derive(Debug, Serialize)]
struct UpdateSyncProfile {
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProfileData {
    id: Uuid,
    #[serde(default)]
    owner_id: Uuid,
    last_synced: DateTime<Utc>,
    #[serde(default)]
    last_updated_by_owner: DateTime<Utc>,
}

impl From<SyncProfile> for ProfileData {
    fn from(value: SyncProfile) -> Self {
        let SyncProfile {
            id,
            user_id,
            updated_at,
            ..
        } = value;

        Self {
            id,
            owner_id: user_id,
            last_synced: updated_at,
            last_updated_by_owner: updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UploadResponse {
    id: Uuid,
    key: String,
}

async fn create_profile(app: &AppHandle) -> Result<Uuid> {
    let manager = app.state::<Mutex<ModManager>>();

    let (name, bytes) = {
        let manager = manager.lock().unwrap();
        let profile = manager.active_profile();

        let mut bytes = Cursor::new(Vec::new());
        super::export::export_zip(&profile, &mut bytes).context("failed to export profile")?;

        (profile.name.clone(), bytes.into_inner())
    };

    let Some(user) = supabase::user_info(app) else {
        bail!("not logged in");
    };

    let sync_profile: SyncProfile = supabase::request(Method::POST, "/rest/v1/profile")
        .json_body(NewSyncProfile {
            name,
            user_id: user.id,
        })
        .send_single(app)
        .await
        .context("failed to create profile in database")?;

    let id = sync_profile.id.clone();

    let response = upload_file(sync_profile.id, bytes, Method::POST, app)
        .await
        .context("failed to upload profile")?;

    debug!("uploaded profile: {:?}", response);

    let mut manager = manager.lock().unwrap();
    let profile = manager.active_profile_mut();

    profile.sync_data = Some(sync_profile.into());

    let prefs = app.state::<Mutex<Prefs>>();
    let prefs = prefs.lock().unwrap();

    manager.save(&prefs)?;

    Ok(id)
}

async fn push_profile(app: &AppHandle) -> Result<()> {
    let manager = app.state::<Mutex<ModManager>>();

    let (id, bytes) = {
        let manager = manager.lock().unwrap();
        let profile = manager.active_profile();

        let id = profile
            .sync_data
            .as_ref()
            .map(|data| data.id)
            .ok_or_eyre("profile is not synced")?;

        let mut bytes = Cursor::new(Vec::new());
        super::export::export_zip(&profile, &mut bytes).context("failed to export profile")?;

        (id, bytes.into_inner())
    };

    let updated_at = Utc::now();

    let _: SyncProfile = supabase::request(Method::PATCH, "/rest/v1/profile")
        .query("id", format!("eq.{id}"))
        .json_body(UpdateSyncProfile { updated_at })
        .send_single(app)
        .await
        .context("failed to update profile in database")?;

    let response = upload_file(id, bytes, Method::PUT, app)
        .await
        .context("failed to upload profile")?;

    debug!("uploaded profile: {:?}", response);

    {
        let mut manager = manager.lock().unwrap();
        let profile = manager.active_profile_mut();
        let sync_data = profile.sync_data.as_mut().unwrap();

        sync_data.last_synced = updated_at;
        sync_data.last_updated_by_owner = updated_at;

        let prefs = app.state::<Mutex<Prefs>>();
        let prefs = prefs.lock().unwrap();

        manager.save(&prefs)?;
    };

    Ok(())
}

async fn upload_file(
    id: Uuid,
    bytes: Vec<u8>,
    method: Method,
    app: &AppHandle,
) -> Result<UploadResponse> {
    supabase::request(method, format!("/storage/v1/object/profile/{}", id))
        .binary_body(bytes)
        .send(app)
        .await
        .context("failed to upload file")
}

async fn clone_profile(id: Uuid, app: &AppHandle) -> Result<()> {
    let sync_profile = get_profile(id, app).await?;

    let name = format!("{} (client)", sync_profile.name);
    download_and_import_file(name, sync_profile, app).await
}

async fn pull_profile(app: &AppHandle) -> Result<()> {
    let manager = app.state::<Mutex<ModManager>>();

    let (name, id, last_synced) = {
        let mut manager = manager.lock().unwrap();
        let profile = manager.active_profile_mut();

        match &profile.sync_data {
            Some(data) => (profile.name.clone(), data.id, data.last_synced),
            None => bail!("profile is not synced"),
        }
    };

    let sync_profile = get_profile(id, app).await?;

    if last_synced < sync_profile.updated_at {
        download_and_import_file(name, sync_profile, app).await?;
    }

    Ok(())
}

async fn fetch_profile(app: &AppHandle) -> Result<()> {
    let manager = app.state::<Mutex<ModManager>>();

    let id = {
        let manager = manager.lock().unwrap();
        let profile = manager.active_profile();

        profile
            .sync_data
            .as_ref()
            .map(|data| data.id)
            .ok_or_eyre("profile is not synced")?
    };

    let updated_at = get_profile(id, app)
        .await
        .map(|profile| profile.updated_at)?;

    let mut manager = manager.lock().unwrap();
    manager
        .active_profile_mut()
        .sync_data
        .as_mut()
        .unwrap()
        .last_updated_by_owner = updated_at;

    Ok(())
}

async fn fetch_and_pull_profile(app: &AppHandle) -> Result<()> {
    Ok(())
}

async fn download_and_import_file(
    name: String,
    sync_profile: SyncProfile,
    app: &AppHandle,
) -> Result<()> {
    let path = format!("/object/profile/{}", sync_profile.id);
    let bytes = supabase::storage_request(Method::GET, path)
        .send_raw(app)
        .await
        .context("failed to download profile")?
        .bytes()
        .await
        .context("error while downloading profile")?;

    let mut data =
        super::import::import_file(Cursor::new(bytes), app).context("failed to import profile")?;

    data.name = name.clone();

    super::import::import_data(data, InstallOptions::default(), false, app)
        .await
        .context("failed to import profile")?;

    // import_data deletes and recreates the profile, so we need to set sync_data again
    let manager = app.state::<Mutex<ModManager>>();
    let mut manager = manager.lock().unwrap();

    let game = manager.active_game_mut();
    let index = game.profile_index(&name).context("profile not found")?;

    let mut sync_data: ProfileData = sync_profile.into();
    sync_data.last_synced = Utc::now();
    game.profiles[index].sync_data = Some(sync_data);

    let prefs = app.state::<Mutex<Prefs>>();
    let prefs = prefs.lock().unwrap();

    manager.save(&prefs)
}

async fn get_profile(id: Uuid, app: &AppHandle) -> Result<SyncProfile> {
    supabase::db_request(Method::GET, "/profile")
        .query("select", "*")
        .query("id", format!("eq.{}", id))
        .send_optional(app)
        .await
        .context("failed to query database")?
        .ok_or_eyre("profile not found")
}
