use std::{borrow::Cow, collections::HashMap, fmt::Display};

use eyre::{Context, OptionExt, Result};
use reqwest::Body;
use serde::{de::DeserializeOwned, Serialize};
use tauri::{
    http::{HeaderMap, HeaderValue},
    AppHandle, Manager,
};

use crate::NetworkClient;

mod auth;
pub use auth::{login_with_oauth, user_id, OAuthProvider};

const PROJECT_URL: &str = "https://phpkxfkbquscgqvhtuuv.supabase.co";
const ANON_KEY: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6InBocGt4ZmticXVzY2dxdmh0dXV2Iiwicm9sZSI6ImFub24iLCJpYXQiOjE3MzcyODAzNDgsImV4cCI6MjA1Mjg1NjM0OH0._eOEhNdG5dIpLnArUcTiicwuxv-hYQlZSSqc06-Aj0k";

type CowStr = Cow<'static, str>;

pub fn request(method: reqwest::Method, path: impl Into<String>) -> RequestBuilder {
    RequestBuilder::new(method, path)
}

pub fn storage_request(method: reqwest::Method, path: impl Display) -> RequestBuilder {
    RequestBuilder::new(method, format!("/storage/v1{}", path))
}

pub fn db_request(method: reqwest::Method, path: impl Display) -> RequestBuilder {
    RequestBuilder::new(method, format!("/rest/v1{}", path))
}

pub fn auth_request(method: reqwest::Method, path: impl Display) -> RequestBuilder {
    RequestBuilder::new(method, format!("/auth/v1{}", path))
}

pub struct RequestBuilder {
    method: reqwest::Method,
    path: String,
    headers: HeaderMap<HeaderValue>,
    query: HashMap<CowStr, CowStr>,
    body: Option<reqwest::Body>,
}

impl RequestBuilder {
    pub fn new(method: reqwest::Method, path: impl Into<String>) -> Self {
        Self {
            method,
            path: path.into(),
            headers: HeaderMap::new(),
            query: HashMap::new(),
            body: None,
        }
    }

    pub fn json_body(mut self, payload: impl Serialize) -> Self {
        let json = serde_json::to_string(&payload).expect("failed to serialize payload");
        self.body = Some(Body::from(json));
        self.headers
            .insert("Content-Type", HeaderValue::from_static("application/json"));
        self
    }

    pub fn binary_body(mut self, payload: Vec<u8>) -> Self {
        self.body = Some(Body::from(payload));
        self.headers.insert(
            "Content-Type",
            HeaderValue::from_static("application/octet-stream"),
        );
        self
    }

    pub fn query(mut self, key: impl Into<CowStr>, value: impl Into<CowStr>) -> Self {
        self.query.insert(key.into(), value.into());
        self
    }

    pub async fn send_raw_no_auth(self, app: &AppHandle) -> Result<reqwest::Response> {
        let client = &app.state::<NetworkClient>().0;

        let url = format!("{}{}", PROJECT_URL, self.path);

        let mut request = client
            .request(self.method, url)
            .header("Prefer", "return=representation")
            .header("apikey", ANON_KEY)
            .headers(self.headers)
            .query(&self.query);

        if let Some(body) = self.body {
            request = request.body(body);
        }

        let response = request.send().await?.error_for_status()?;
        Ok(response)
    }

    pub async fn send_raw(mut self, app: &AppHandle) -> Result<reqwest::Response> {
        let token = auth::access_token(app)
            .await
            .unwrap_or_else(|| ANON_KEY.to_owned());

        let header = format!("Bearer {}", token);

        self.headers.insert(
            "Authorization",
            HeaderValue::from_maybe_shared(header).unwrap(),
        );

        self.send_raw_no_auth(app).await
    }

    pub async fn send<T>(self, app: &AppHandle) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let result = self.send_raw(app).await?.json().await?;
        Ok(result)
    }

    pub async fn send_optional<T>(self, app: &AppHandle) -> Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        Ok(self.send::<Vec<T>>(app).await?.into_iter().next())
    }

    pub async fn send_single<T>(self, app: &AppHandle) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.send_optional(app)
            .await?
            .ok_or_eyre("expected at least one result, got zero")
    }
}

pub fn setup(app: &AppHandle) -> Result<()> {
    auth::setup(app).context("failed to initialze auth")?;

    Ok(())
}
