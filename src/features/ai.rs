use crate::core::{Config, Item, ItemType};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct AiManager {
    api_key: Option<String>,
}

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
}

#[derive(Serialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<GeminiCandidate>>,
    error: Option<GeminiError>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiResponseContent,
}

#[derive(Deserialize)]
struct GeminiResponseContent {
    parts: Vec<GeminiResponsePart>,
}

#[derive(Deserialize)]
struct GeminiResponsePart {
    text: String,
}

#[derive(Deserialize)]
struct GeminiError {
    message: String,
}

impl AiManager {
    pub fn new(config: &Config) -> Self {
        Self {
            api_key: config.gemini_api_key.clone(),
        }
    }

    pub fn get_items(&self, query: &str) -> Vec<Item> {
        let mut items = Vec::new();

        if self.api_key.is_none() {
            items.push(
                Item::new("ai:no_key", "API key not configured", ItemType::AiQuery)
                    .with_description("Add gemini_api_key to ~/.config/wlaunch/config.json")
                    .with_icon("dialog-warning"),
            );
            return items;
        }

        if query.is_empty() {
            items.push(
                Item::new("ai:hint", "Ask a question...", ItemType::AiQuery)
                    .with_description("Type your question to query Gemini AI")
                    .with_icon("dialog-question"),
            );
        } else {
            items.push(
                Item::new(format!("ai:query:{}", query), format!("Ask: {}", query), ItemType::AiQuery)
                    .with_description("Press Enter to query Gemini AI")
                    .with_icon("dialog-question"),
            );
        }

        items
    }

    pub async fn query(&self, prompt: &str) -> Result<String> {
        let api_key = self.api_key.as_ref().ok_or_else(|| {
            anyhow::anyhow!("API key not configured")
        })?;

        let client = reqwest::Client::new();
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-pro:generateContent?key={}",
            api_key
        );

        let request = GeminiRequest {
            contents: vec![GeminiContent {
                parts: vec![GeminiPart {
                    text: prompt.to_string(),
                }],
            }],
        };

        let response = client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .json::<GeminiResponse>()
            .await?;

        if let Some(error) = response.error {
            return Err(anyhow::anyhow!("API error: {}", error.message));
        }

        let text = response
            .candidates
            .and_then(|c| c.into_iter().next())
            .map(|c| c.content.parts.into_iter().map(|p| p.text).collect::<Vec<_>>().join(""))
            .unwrap_or_else(|| "No response".to_string());

        Ok(text)
    }
}

impl Default for AiManager {
    fn default() -> Self {
        Self::new(&Config::default())
    }
}
