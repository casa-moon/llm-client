use anyhow::{anyhow, Result};

use crate::message_log::MessageLog;

use crate::api::traits::ApiClient;
use crate::api::transform::transform_messages;
use crate::api::types::{ModelResponse, TemplateKey};

pub struct GoogleClient {
  pub api_key: String,
}

impl GoogleClient { pub fn new(api_key: String) -> Self { Self { api_key } } }

impl ApiClient for GoogleClient {
  fn name(&self) -> &'static str { "google" }
  fn template(&self) -> TemplateKey { TemplateKey::Google }
  fn send_message(&mut self, model: &str, log: &MessageLog) -> Result<ModelResponse> {
    let contents = transform_messages(log.raw(), self.template())?;
    let body = serde_json::json!({
      "contents": contents,
      "generationConfig": { "maxOutputTokens": 2048 },
    });

    let url = format!(
      "https://generativelanguage.googleapis.com/v1/models/{}:generateContent?key={}",
      model, self.api_key
    );
    let http = crate::http::http_client()?;
    let resp = http.post(&url).json(&body).send()?;
    if !resp.status().is_success() {
      let status = resp.status();
      let txt = resp.text().unwrap_or_default();
      return Err(anyhow!("Google API error: {} - {}", status, txt));
    }
    let raw: serde_json::Value = resp.json()?;
    let text = raw
      .get("candidates")
      .and_then(|c| c.get(0))
      .and_then(|cand| cand.get("content"))
      .and_then(|content| content.get("parts"))
      .and_then(|parts| parts.as_array().cloned())
      .map(|arr| arr
        .into_iter()
        .filter_map(|p| p.get("text").and_then(|t| t.as_str()).map(|s| s.to_string()))
        .collect::<Vec<_>>()
        .join("\n")
      )
      .unwrap_or_default();
    Ok(ModelResponse { raw, text })
  }
}
