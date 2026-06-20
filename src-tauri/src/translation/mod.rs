use std::{collections::HashMap, time::Duration};

use eyre::{bail, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, warn};

pub mod commands;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct TranslationPrefs {
    pub enabled: bool,
    pub api_url: String,
    pub api_key: String,
    pub model: String,
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
}

fn default_batch_size() -> usize {
    20
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TranslateRequest {
    pub uuid: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TranslateResponse {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessageResponse,
}

#[derive(Deserialize)]
struct ChatMessageResponse {
    content: String,
}

pub struct TranslationService {
    client: Client,
}

impl TranslationService {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .expect("failed to build HTTP client"),
        }
    }

    pub async fn translate_mods(
        &self,
        mods: &[TranslateRequest],
        target_language: &str,
        prefs: &TranslationPrefs,
    ) -> Result<Vec<TranslateResponse>> {
        if prefs.api_url.is_empty() || prefs.api_key.is_empty() {
            bail!("Translation API URL or API key is not configured");
        }

        let batch_size = if prefs.batch_size == 0 { 20 } else { prefs.batch_size };
        let mut all_results = Vec::with_capacity(mods.len());

        for chunk in mods.chunks(batch_size) {
            let batch_results = self.translate_batch(chunk, target_language, prefs).await?;
            all_results.extend(batch_results);
        }

        Ok(all_results)
    }

    async fn translate_batch(
        &self,
        mods: &[TranslateRequest],
        target_language: &str,
        prefs: &TranslationPrefs,
    ) -> Result<Vec<TranslateResponse>> {
        let prompt = self.build_batch_prompt(mods, target_language);

        let request = ChatRequest {
            model: prefs.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: format!(
                        "You are a professional translator for game mods. Translate the mod names and descriptions to {}. Keep translations natural and concise. Return ONLY a JSON array, no other text. Each element: {{\"uuid\": \"...\", \"name\": \"...\", \"description\": \"...\"}}",
                        target_language
                    ),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: prompt,
                },
            ],
            temperature: 0.3,
        };

        let base = prefs.api_url.trim_end_matches('/');
        let url = if base.ends_with("/chat/completions") {
            base.to_string()
        } else if base.ends_with("/v1") {
            format!("{}/chat/completions", base)
        } else {
            format!("{}/v1/chat/completions", base)
        };

        debug!("Translating batch of {} mods -> {}", mods.len(), url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", prefs.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Translation API error: {} - {}", status, body);
            bail!("Translation API returned error: {}", status);
        }

        let chat_response: ChatResponse = response.json().await?;

        let content = chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        self.parse_batch_response(&content, mods)
    }

    fn build_batch_prompt(&self, mods: &[TranslateRequest], target_language: &str) -> String {
        let mut lines = vec![format!("Translate these mods to {}:\n", target_language)];

        for m in mods {
            let desc = m.description.as_deref().unwrap_or("");
            lines.push(format!("[uuid:{}]\nname:{}\ndescription:{}\n", m.uuid, m.name, desc));
        }

        lines.join("\n")
    }

    fn parse_batch_response(
        &self,
        content: &str,
        originals: &[TranslateRequest],
    ) -> Result<Vec<TranslateResponse>> {
        let content = content.trim();

        // Try to find JSON array in the response
        if let Some(json_start) = content.find('[') {
            if let Some(json_end) = content.rfind(']') {
                let json_str = &content[json_start..=json_end];
                if let Ok(parsed) = serde_json::from_str::<Vec<HashMap<String, String>>>(json_str) {
                    let mut results = Vec::with_capacity(originals.len());
                    for original in originals {
                        let translated = parsed.iter().find(|m| m.get("uuid").map(|u| u.as_str()) == Some(&original.uuid));
                        results.push(TranslateResponse {
                            name: translated
                                .and_then(|t| t.get("name").cloned())
                                .unwrap_or_else(|| original.name.clone()),
                            description: translated
                                .and_then(|t| t.get("description").cloned())
                                .or_else(|| original.description.clone()),
                        });
                    }
                    return Ok(results);
                }
            }
        }

        warn!("Failed to parse translation response, falling back to originals. Response: {}", &content[..content.len().min(200)]);
        Ok(originals
            .iter()
            .map(|m| TranslateResponse {
                name: m.name.clone(),
                description: m.description.clone(),
            })
            .collect())
    }
}
