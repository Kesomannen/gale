use std::{borrow::Cow, collections::HashMap, fmt::Display};

use eyre::{OptionExt, Result};
use reqwest::Body;
use serde::{de::DeserializeOwned, Serialize};
use tauri::http::{HeaderMap, HeaderValue};

pub const PROJECT_URL: &str = "https://phpkxfkbquscgqvhtuuv.supabase.co";
pub const ANON_KEY: &str = "...";

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

    pub async fn send_raw(self, client: &reqwest::Client) -> Result<reqwest::Response> {
        let url = format!("{}{}", PROJECT_URL, self.path);

        let mut request = client
            .request(self.method, url)
            .bearer_auth(ANON_KEY)
            .header("apikey", ANON_KEY)
            .header("Prefer", "return=representation")
            .headers(self.headers)
            .query(&self.query);

        if let Some(body) = self.body {
            request = request.body(body);
        }

        let response = request.send().await?.error_for_status()?;
        Ok(response)
    }

    pub async fn send<T>(self, client: &reqwest::Client) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let result = self.send_raw(client).await?.json().await?;
        Ok(result)
    }

    pub async fn send_optional<T>(self, client: &reqwest::Client) -> Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        Ok(self.send::<Vec<T>>(client).await?.into_iter().next())
    }

    pub async fn send_single<T>(self, client: &reqwest::Client) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.send_optional(client)
            .await?
            .ok_or_eyre("expected at least one result, got zero")
    }
}
