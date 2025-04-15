use std::{collections::HashMap, fmt::Display, time::Duration};

use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, Utc};
use eyre::{eyre, Context, OptionExt, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::{AppHandle, Manager};
use tauri_plugin_oauth::OauthConfig;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::state::ManagerExt;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthState {
    user: User,
    access_token: String,
    token_expiry: i64,
    refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub display_name: Option<String>,
    pub avatar_url: String,
}

impl AuthState {
    fn from_jwt(access_token: String, refresh_token: String) -> Result<Self> {
        let JwtPayload {
            exp,
            sub,
            user_metadata,
        } = decode_jwt(&access_token).context("failed to decode jwt")?;

        Ok(Self {
            access_token,
            refresh_token,
            token_expiry: exp,
            user: User {
                id: sub,
                name: user_metadata.full_name,
                display_name: user_metadata.custom_claims.global_name,
                avatar_url: user_metadata.avatar_url,
            },
        })
    }
}

const OAUTH_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OAuthProvider {
    Discord,
    Github,
}

impl Display for OAuthProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OAuthProvider::Discord => write!(f, "discord"),
            OAuthProvider::Github => write!(f, "github"),
        }
    }
}

pub async fn login_with_oauth(provider: OAuthProvider, app: &AppHandle) -> Result<User> {
    let url = format!(
        "{}/auth/v1/authorize?provider={}&scopes=identify",
        super::PROJECT_URL,
        provider
    );
    open::that(url).context("failed to open url in browser")?;

    let (access_token, refresh_token) = run_oauth_server()
        .await
        .context("failed to run OAuth callback server")?;

    app.get_window("main").unwrap().set_focus().ok();

    let state = AuthState::from_jwt(access_token, refresh_token).context("failed to save state")?;
    let user = state.user.clone();

    info!("logged in as {} with {}", user.name, provider);

    *app.lock_auth() = Some(state);

    Ok(user)
}

async fn run_oauth_server() -> Result<(String, String)> {
    let (tx, mut rx) = mpsc::channel(1);
    let port = tauri_plugin_oauth::start_with_config(
        OauthConfig {
            ports: Some(vec![22942]),
            response: None,
        },
        move |url| {
            if let Err(url) = tx.blocking_send(url) {
                warn!(
                    "got OAuth callback but channel was already closed (url: {})",
                    url.0
                );
            }
        },
    )?;

    tokio::select! {
        url = rx.recv() => {
            tauri_plugin_oauth::cancel(port).ok();

            let url = url.expect("url sender was dropped too early!");

            let tokens = url.split_once("#").and_then(|(_, fragment)| {
                let params = fragment
                    .split("&")
                    .filter_map(|param| param.split_once("="))
                    .collect::<HashMap<&str, &str>>();

                let access_token = params.get("access_token")?.to_string();
                let refresh_token = params.get("refresh_token")?.to_string();

                Some((access_token, refresh_token))
            })
            .ok_or_eyre("invalid callback url format")?;

            Ok(tokens)
        }
        _ = tokio::time::sleep(OAUTH_TIMEOUT) => {
            tauri_plugin_oauth::cancel(port).ok();

            Err(eyre!("timed out"))
        }
    }
}

#[derive(Debug, Deserialize)]
struct JwtPayload {
    exp: i64,
    sub: Uuid,
    user_metadata: UserMetadata,
}

#[derive(Debug, Deserialize)]
struct UserMetadata {
    full_name: String,
    avatar_url: String,
    custom_claims: CustomClaims,
}

#[derive(Debug, Deserialize)]
struct CustomClaims {
    #[serde(default)]
    global_name: Option<String>,
}

fn decode_jwt(token: &str) -> Result<JwtPayload> {
    let payload = token.split(".").nth(1).ok_or_eyre("token is malformed")?;

    let bytes = BASE64_URL_SAFE_NO_PAD
        .decode(payload)
        .context("failed to decode base64")?;

    serde_json::from_slice(&bytes).context("failed to deserialize json")
}

pub fn user_info(app: &AppHandle) -> Option<User> {
    app.lock_auth().as_ref().map(|state| state.user.clone())
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: String,
}

pub async fn access_token(app: &AppHandle) -> Option<String> {
    let refresh_token = {
        let auth = app.lock_auth();
        let state = auth.as_ref()?;

        let Some(expiry) = DateTime::from_timestamp(state.token_expiry, 0) else {
            warn!("token expiry date is invalid");
            return None;
        };

        if Utc::now() < expiry {
            return Some(state.access_token.clone());
        }

        state.refresh_token.clone()
    };

    match request_token(refresh_token, app).await {
        Ok(token) => Some(token),
        Err(err) => {
            error!("failed to refresh access token: {:#}", err);
            None
        }
    }
}

async fn request_token(refresh_token: String, app: &AppHandle) -> Result<String> {
    debug!("refreshing access token");

    let response: TokenResponse = super::auth_request(reqwest::Method::POST, "/token")
        .query("grant_type", "refresh_token")
        .json_body(json!({
            "refresh_token": refresh_token
        }))
        .send_raw_no_auth(app)
        .await?
        .json()
        .await?;

    let state = AuthState::from_jwt(response.access_token.clone(), response.refresh_token)?;

    *app.lock_auth() = Some(state);

    Ok(response.access_token)
}
