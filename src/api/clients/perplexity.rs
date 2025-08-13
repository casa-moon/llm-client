use anyhow::{anyhow, Result};

use crate::message_log::MessageLog;

use super::super::traits::ApiClient;
use super::super::transform::transform_messages;
use super::super::types::{ModelResponse, TemplateKey};

pub struct PerplexityClient {
  pub api_key: String,
}

impl PerplexityClient { pub fn new(api_key: String) -> Self { Self { api_key } } }

impl ApiClient for PerplexityClient {
  fn name(&self) -> &'static str { "perplexity" }
  fn template(&self) -> TemplateKey { TemplateKey::Perplexity }
  fn send_message(&mut self, model: &str, log: &MessageLog) -> Result<ModelResponse> {
    use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
    let payload = transform_messages(log.raw(), self.template())?;
    let body = serde_json::json!({
      "model": model,
      "max_tokens": 4096,
      "temperature": 0,
      "messages": payload,
    });
    let http = crate::http::http_client()?;
    let resp = http
      .post("https://api.perplexity.ai/chat/completions")
      .header(AUTHORIZATION, format!("Bearer {}", self.api_key))
      .header(CONTENT_TYPE, "application/json")
      .json(&body)
      .send()?;
    if !resp.status().is_success() {
      let status = resp.status();
      let txt = resp.text().unwrap_or_default();
      return Err(anyhow!("Perplexity API error: {} - {}", status, txt));
    }
    let raw: serde_json::Value = resp.json()?;
    let text = raw
      .get("choices").and_then(|c| c.get(0))
      .and_then(|c| c.get("message"))
      .and_then(|m| m.get("content"))
      .and_then(|s| s.as_str())
      .unwrap_or_default()
      .to_string();
    Ok(ModelResponse { raw, text })
  }
}

