use std::{collections::HashMap, time::Duration};

use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, Utc};
use eyre::{eyre, Context, ContextCompat, OptionExt, Result};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Url};
use tauri_plugin_oauth::OauthConfig;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::{profile::sync::API_URL, state::ManagerExt};

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
    pub discord_id: i64,
    pub name: String,
    pub display_name: String,
    pub avatar: String,
}

impl AuthState {
    fn from_tokens(access_token: String, refresh_token: String) -> Result<Self> {
        let JwtPayload { exp, user } = decode_jwt(&access_token).context("failed to decode jwt")?;

        Ok(Self {
            access_token,
            refresh_token,
            token_expiry: exp,
            user,
        })
    }
}

const OAUTH_TIMEOUT: Duration = Duration::from_secs(60);

pub async fn login_with_oauth(app: &AppHandle) -> Result<User> {
    let url = format!("{}/auth/login", API_URL);
    open::that(url).context("failed to open url in browser")?;

    let (access_token, refresh_token) = run_oauth_server()
        .await
        .context("failed to run OAuth callback server")?;

    app.get_window("main").unwrap().set_focus().ok();

    let state = AuthState::from_tokens(access_token, refresh_token)?;
    let user = state.user.clone();

    info!("logged in as {}", user.name);

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
            let url = Url::parse(&url).expect("invalid url");
            let query: HashMap<_, _> = url.query_pairs().collect();

            let access_token = query.get("access_token").context("access_token parameter missing")?.clone();
            let refresh_token = query.get("refresh_token").context("refresh_token parameter missing")?.clone();

            Ok((access_token.into_owned(), refresh_token.into_owned()))
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

    #[serde(flatten)]
    user: User,
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GrantTokenRequest {
    refresh_token: String,
}

async fn request_token(refresh_token: String, app: &AppHandle) -> Result<String> {
    debug!("refreshing access token");

    let response: TokenResponse = app
        .http()
        .get(format!("{}/auth/token", API_URL))
        .json(&GrantTokenRequest { refresh_token })
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let state = AuthState::from_tokens(response.access_token.clone(), response.refresh_token)?;

    let mut auth = app.lock_auth();
    *auth = Some(state);
    app.db().save_auth(auth.as_ref())?;

    Ok(response.access_token)
}
