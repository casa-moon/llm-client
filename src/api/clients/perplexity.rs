use anyhow::{anyhow, Result};

use crate::api::traits::ApiClient;
use crate::api::transform::transform_messages;
use crate::api::types::{ModelResponse, TemplateKey};
use crate::message_log::MessageLog;

pub struct PerplexityClient {
  pub api_key: String,
}

impl PerplexityClient {
  pub fn new(api_key: String) -> Self {
    Self { api_key }
  }
}

impl ApiClient for PerplexityClient {
  fn template(&self) -> TemplateKey {
    TemplateKey::Perplexity
  }
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
    let text = self.extract_text(&raw);
    Ok(ModelResponse { raw, text })
  }
}
