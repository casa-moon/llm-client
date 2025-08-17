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
  fn template(&self) -> TemplateKey { TemplateKey::Google }
  fn send_message(&mut self, model: &str, log: &MessageLog) -> Result<ModelResponse> {
    let contents = transform_messages(log.raw(), self.template())?;
    let body = serde_json::json!({
      "contents": contents,
      "generationConfig": { "maxOutputTokens": 2048 },
    });

    // Use v1beta endpoint and x-goog-api-key header per latest format
    let url = format!(
      "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
      model
    );
    let http = crate::http::http_client()?;
    let resp = http
      .post(&url)
      .header("x-goog-api-key", &self.api_key)
      .json(&body)
      .send()?;
    if !resp.status().is_success() {
      let status = resp.status();
      let txt = resp.text().unwrap_or_default();
      return Err(anyhow!("Google API error: {} - {}", status, txt));
    }
    let raw: serde_json::Value = resp.json()?;
    // Robustly collect all text parts from the first candidate (or all, if desired)
    let mut texts: Vec<String> = Vec::new();
    if let Some(cands) = raw.get("candidates").and_then(|c| c.as_array()) {
      for cand in cands.iter().take(1) {
        // Typical shape: candidates[0].content.parts[*].text
        if let Some(parts) = cand
          .get("content")
          .and_then(|content| content.get("parts"))
          .and_then(|parts| parts.as_array())
        {
          for p in parts {
            if let Some(s) = p.get("text").and_then(|t| t.as_str()) {
              texts.push(s.to_string());
            }
          }
        }
      }
    }
    // Fallback: sometimes providers may inline a top-level text field (defensive)
    if texts.is_empty() {
      if let Some(s) = raw.get("text").and_then(|t| t.as_str()) {
        texts.push(s.to_string());
      }
    }
    let text = texts.join("\n");
    Ok(ModelResponse { raw, text })
  }
}
