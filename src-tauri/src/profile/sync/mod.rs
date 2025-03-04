use std::{io::Cursor, sync::Mutex};

use chrono::{DateTime, Utc};
use eyre::{bail, ensure, Context, OptionExt, Result};
use log::{debug, info};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use uuid::{uuid, Uuid};

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
    last_synced: DateTime<Utc>,
    is_owner: bool,
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

    let Some(user_id) = supabase::user_id(app) else {
        bail!("not logged in");
    };

    let sync_profile: SyncProfile = supabase::request(Method::POST, "/rest/v1/profile")
        .json_body(NewSyncProfile { name, user_id })
        .send_single(app)
        .await
        .context("failed to create profile in database")?;

    let response = upload_file(sync_profile.id, bytes, Method::POST, app)
        .await
        .context("failed to upload profile")?;

    debug!("uploaded profile: {:?}", response);

    {
        let mut manager = manager.lock().unwrap();
        let profile = manager.active_profile_mut();

        profile.sync_data = Some(ProfileData {
            id: sync_profile.id,
            last_synced: sync_profile.created_at,
            is_owner: true,
        });

        let prefs = app.state::<Mutex<Prefs>>();
        let prefs = prefs.lock().unwrap();

        manager.save(&prefs)?;
    };

    Ok(sync_profile.id)
}

async fn push_profile(app: &AppHandle) -> Result<()> {
    let manager = app.state::<Mutex<ModManager>>();

    let (id, bytes) = {
        let manager = manager.lock().unwrap();
        let profile = manager.active_profile();

        let id = match &profile.sync_data {
            Some(data) if data.is_owner => data.id,
            Some(_) => bail!("not the owner of the profile"),
            None => bail!("profile is not synced"),
        };

        let mut bytes = Cursor::new(Vec::new());
        super::export::export_zip(&profile, &mut bytes).context("failed to export profile")?;

        (id, bytes.into_inner())
    };

    let time = Utc::now();

    let _: SyncProfile = supabase::request(Method::PATCH, "/rest/v1/profile")
        .query("id", format!("eq.{id}"))
        .json_body(UpdateSyncProfile { updated_at: time })
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

        profile.sync_data.as_mut().unwrap().last_synced = time;

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
    let sync_profile: SyncProfile = supabase::db_request(Method::GET, "/profile")
        .query("select", "*")
        .query("id", format!("eq.{id}"))
        .send_optional(app)
        .await
        .context("failed to query database")?
        .ok_or_eyre("profile not found")?;

    let name = format!("{} (client)", sync_profile.name);
    download_and_import_file(id, name, app).await?;

    Ok(())
}

async fn pull_profile(app: &AppHandle) -> Result<()> {
    let manager = app.state::<Mutex<ModManager>>();

    let (id, name) = {
        let mut manager = manager.lock().unwrap();
        let profile = manager.active_profile_mut();

        let info = match &mut profile.sync_data {
            Some(data) if !data.is_owner => {
                data.last_synced = Utc::now();

                data.id
            }
            Some(_) => bail!("owner of the profile"),
            None => bail!("profile is not synced"),
        };

        (info, profile.name.clone())
    };

    download_and_import_file(id, name, app).await?;

    Ok(())
}

async fn download_and_import_file(
    id: Uuid,
    name: String,
    app: &AppHandle,
) -> Result<(), eyre::Error> {
    let bytes = supabase::storage_request(Method::GET, format!("/object/profile/{}", id))
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

    {
        let manager = app.state::<Mutex<ModManager>>();
        let mut manager = manager.lock().unwrap();

        let game = manager.active_game_mut();
        let index = game.profile_index(&name).expect("profile not found");

        game.profiles[index].sync_data = Some(ProfileData {
            id,
            last_synced: Utc::now(),
            is_owner: false,
        });

        let prefs = app.state::<Mutex<Prefs>>();
        let prefs = prefs.lock().unwrap();

        manager.save(&prefs)?;
    }

    Ok(())
}
