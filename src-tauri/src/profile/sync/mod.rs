use std::{fmt::Display, io::Cursor};

use chrono::{DateTime, Utc};
use eyre::{bail, eyre, Context, OptionExt, Result};
use reqwest::{Method, StatusCode};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::{profile::install::InstallOptions, state::ManagerExt};

pub mod auth;
pub mod commands;
pub mod socket;

const API_URL: &str = "https://gale.kesomannen.com/api";
//const API_URL: &str = "http://127.0.0.1:8080/api"; // for local testing

async fn request(method: Method, path: impl Display, app: &AppHandle) -> reqwest::RequestBuilder {
    let mut req = app.http().request(method, format!("{API_URL}{path}"));
    if let Some(token) = auth::access_token(app).await {
        req = req.bearer_auth(token);
    }
    req
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateSyncProfileResponse {
    id: String,
    #[allow(unused)]
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SyncProfileMetadata {
    id: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    owner: auth::User,
    manifest: super::export::ProfileManifest,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SyncProfileData {
    id: String,
    owner: auth::User,
    synced_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    #[serde(default)]
    missing: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct FullUserInfo {
    #[serde(flatten)]
    user: auth::User,
    profiles: Option<Vec<ListedSyncProfile>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListedSyncProfile {
    id: String,
    name: String,
    community: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<SyncProfileMetadata> for SyncProfileData {
    fn from(value: SyncProfileMetadata) -> Self {
        SyncProfileData {
            id: value.id,
            owner: value.owner,
            synced_at: value.updated_at,
            updated_at: value.updated_at,
            missing: false,
        }
    }
}

async fn create_profile(app: &AppHandle) -> Result<String> {
    let Some(user) = auth::user_info(app) else {
        bail!("not logged in");
    };

    let bytes = {
        let manager = app.lock_manager();
        let game = manager.active_game();
        let profile = game.active_profile();

        let mut bytes = Cursor::new(Vec::new());
        super::export::export_zip(profile, &mut bytes, game.game)
            .context("failed to export profile")?;

        bytes.into_inner()
    };

    let response: CreateSyncProfileResponse = request(Method::POST, "/profile", app)
        .await
        .body(bytes)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let id = response.id.clone();

    {
        let mut manager = app.lock_manager();
        let profile = manager.active_profile_mut();

        profile.sync = Some(SyncProfileData {
            id: id.clone(),
            owner: user,
            synced_at: response.updated_at,
            updated_at: response.updated_at,
            missing: false,
        });

        profile.save(&app, true)?;
    }

    Ok(id)
}

async fn push_profile(app: &AppHandle) -> Result<()> {
    let (id, bytes) = {
        let manager = app.lock_manager();
        let game = manager.active_game();
        let profile = game.active_profile();

        let id = profile
            .sync
            .as_ref()
            .map(|data| data.id.clone())
            .ok_or_eyre("profile is not synced")?;

        let mut bytes = Cursor::new(Vec::new());
        super::export::export_zip(profile, &mut bytes, game.game)
            .context("failed to export profile")?;

        (id, bytes.into_inner())
    };

    let response: CreateSyncProfileResponse = request(Method::PUT, format!("/profile/{id}"), app)
        .await
        .body(bytes)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    {
        let mut manager = app.lock_manager();
        let profile = manager.active_profile_mut();
        let sync_data = profile.sync.as_mut().unwrap();

        sync_data.synced_at = response.updated_at;
        sync_data.updated_at = response.updated_at;

        profile.save(&app, true)?;
    };

    Ok(())
}

async fn disconnect_profile(delete: bool, app: &AppHandle) -> Result<()> {
    let (id, is_owner) = {
        let mut manager = app.lock_manager();
        let profile = manager.active_profile_mut();

        let (id, owner_discord_id) = profile
            .sync
            .as_ref()
            .map(|info| (info.id.clone(), &info.owner.discord_id))
            .ok_or_eyre("profile is not synced")?;

        let is_owner =
            auth::user_info(app).is_some_and(|user| user.discord_id == *owner_discord_id);

        (id, is_owner)
    };

    if is_owner && delete {
        delete_profile(&id, app).await?;
    }

    {
        let mut manager = app.lock_manager();
        let profile = manager.active_profile_mut();

        profile.sync = None;

        profile.save(&app, true)?;
    }

    Ok(())
}

async fn clone_profile(id: &str, override_name: Option<String>, app: &AppHandle) -> Result<()> {
    let metadata = read_profile(id, app).await?;

    download_and_import_file(override_name, metadata.into(), app).await
}

pub async fn pull_profile(dry_run: bool, app: &AppHandle) -> Result<()> {
    let (id, profile_id, name, synced_at) = {
        let mut manager = app.lock_manager();
        let profile = manager.active_profile_mut();

        match &profile.sync {
            Some(data) if data.missing => bail!("cannot pull from missing profile"),
            Some(data) => (
                data.id.clone(),
                profile.id,
                profile.name.clone(),
                data.synced_at,
            ),
            None => return Ok(()),
        }
    };

    let metadata = get_profile_meta(&id, app).await?;

    match metadata {
        Some(metadata) if !dry_run && metadata.updated_at > synced_at => {
            download_and_import_file(Some(name), metadata.into(), app).await
        }
        metadata => {
            let mut manager = app.lock_manager();
            let (_, profile) = manager.profile_by_id_mut(profile_id)?;

            let Some(sync) = profile.sync.as_mut() else {
                return Ok(());
            };

            match metadata {
                Some(metadata) => {
                    *sync = SyncProfileData {
                        synced_at: sync.synced_at,
                        ..metadata.into()
                    };
                }
                None => sync.missing = true,
            }

            profile.save(&app, true)?;

            Ok(())
        }
    }
}

async fn download_and_import_file(
    override_name: Option<String>,
    sync_profile: SyncProfileData,
    app: &AppHandle,
) -> Result<()> {
    let path = format!("/profile/{}", sync_profile.id);
    let bytes = request(Method::GET, path, app)
        .await
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    let mut data =
        super::import::read_file(Cursor::new(bytes)).context("failed to import profile")?;

    if let Some(name) = override_name {
        data.manifest.name = name;
    }

    let id = super::import::import_profile(data, InstallOptions::default(), false, app)
        .await
        .context("failed to import profile")?;

    {
        let mut manager = app.lock_manager();
        let (_, profile) = manager.profile_by_id_mut(id)?;

        profile.sync = Some(sync_profile);
        profile.save(&app, true)?;
    }

    Ok(())
}

async fn delete_profile(id: &str, app: &AppHandle) -> Result<()> {
    request(Method::DELETE, format!("/profile/{id}"), app)
        .await
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

async fn get_profile_meta(id: &str, app: &AppHandle) -> Result<Option<SyncProfileMetadata>> {
    let res = request(Method::GET, format!("/profile/{id}/meta"), app)
        .await
        .send()
        .await?
        .error_for_status();

    match res {
        Ok(res) => {
            let res = res.json().await?;
            Ok(Some(res))
        }
        Err(err) if err.status() == Some(StatusCode::NOT_FOUND) => Ok(None),
        Err(err) => Err(eyre!(err)),
    }
}

pub async fn read_profile(id: &str, app: &AppHandle) -> Result<SyncProfileMetadata> {
    get_profile_meta(id, app)
        .await?
        .ok_or_eyre("profile not found")
}

async fn get_owned_profiles(app: &AppHandle) -> Result<Vec<ListedSyncProfile>> {
    let user: FullUserInfo = request(Method::GET, "/user/me", app)
        .await
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(user.profiles.unwrap_or_default())
}
