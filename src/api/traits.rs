use anyhow::Result;

use crate::message_log::MessageLog;
use crate::api::types::{ModelResponse, TemplateKey};

pub trait ApiClient {
  fn template(&self) -> TemplateKey;
  fn send_message(&mut self, model: &str, log: &MessageLog) -> Result<ModelResponse>;

  // Default extractor for OpenAI-style responses: choices[0].message.content
  fn extract_text(&self, raw: &serde_json::Value) -> String {
    raw
      .get("choices").and_then(|c| c.get(0))
      .and_then(|c| c.get("message"))
      .and_then(|m| m.get("content"))
      .and_then(|s| s.as_str())
      .unwrap_or_default()
      .to_string()
  }
}
