use anyhow::{anyhow, Result};

use crate::message_log::MessageLog;

use super::super::traits::ApiClient;
use super::super::transform::transform_messages;
use super::super::types::{ModelResponse, TemplateKey};

pub struct AnthropicClient {
  pub api_key: String,
}

impl AnthropicClient { pub fn new(api_key: String) -> Self { Self { api_key } } }

impl ApiClient for AnthropicClient {
  fn name(&self) -> &'static str { "anthropic" }
  fn template(&self) -> TemplateKey { TemplateKey::Anthropic }
  fn send_message(&mut self, model: &str, log: &MessageLog) -> Result<ModelResponse> {
    let payload = transform_messages(log.raw(), self.template())?;
    let body = serde_json::json!({
      "model": model,
      "max_tokens": 4096,
      "temperature": 0,
      "messages": payload,
    });
    let http = crate::http::http_client()?;
    let resp = http
      .post("https://api.anthropic.com/v1/messages")
      .header("x-api-key", &self.api_key)
      .header("anthropic-version", "2023-06-01")
      .json(&body)
      .send()?;
    if !resp.status().is_success() {
      let status = resp.status();
      let txt = resp.text().unwrap_or_default();
      return Err(anyhow!("Anthropic API error: {} - {}", status, txt));
    }
    let raw: serde_json::Value = resp.json()?;
    let text = raw
      .get("content").and_then(|c| c.get(0))
      .and_then(|p| p.get("text"))
      .and_then(|s| s.as_str())
      .unwrap_or_default()
      .to_string();
    Ok(ModelResponse { raw, text })
  }
}

