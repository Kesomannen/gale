use std::{borrow::Cow, collections::HashMap};

use eyre::Result;
use serde::{de::DeserializeOwned, Serialize};

pub const PROJECT_URL: &str = "https://phpkxfkbquscgqvhtuuv.supabase.co";
pub const ANON_KEY: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6InBocGt4ZmticXVzY2dxdmh0dXV2Iiwicm9sZSI6ImFub24iLCJpYXQiOjE3MzcyODAzNDgsImV4cCI6MjA1Mjg1NjM0OH0._eOEhNdG5dIpLnArUcTiicwuxv-hYQlZSSqc06-Aj0k";

type CowStr = Cow<'static, str>;

pub fn request(method: reqwest::Method, path: impl Into<String>) -> RequestBuilder {
    RequestBuilder::new(method, path)
}

pub struct RequestBuilder {
    method: reqwest::Method,
    path: String,
    query: HashMap<CowStr, CowStr>,
    payload: Option<String>,
}

impl RequestBuilder {
    pub fn new(method: reqwest::Method, path: impl Into<String>) -> Self {
        Self {
            method,
            path: path.into(),
            query: HashMap::new(),
            payload: None,
        }
    }

    pub fn payload(mut self, payload: impl Serialize) -> Self {
        let payload = serde_json::to_string(&payload).expect("failed to serialize payload");
        self.payload = Some(payload);
        self
    }

    pub fn query(mut self, key: impl Into<CowStr>, value: impl Into<CowStr>) -> Self {
        self.query.insert(key.into(), value.into());
        self
    }

    pub async fn send<T>(self, client: &reqwest::Client) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let url = format!("{}/rest/v1{}", PROJECT_URL, self.path);

        let mut request = client
            .request(self.method, url)
            .bearer_auth(ANON_KEY)
            .header("apikey", ANON_KEY)
            .query(&self.query);

        if let Some(payload) = self.payload {
            request = request.json(&payload);
        }

        let response = request.send().await?.error_for_status()?.json().await?;
        Ok(response)
    }
}
