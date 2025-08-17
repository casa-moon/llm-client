use anyhow::{anyhow, Result};

use crate::message_log::{MessageLog, MsgType, Role};
use crate::api::traits::ApiClient;
use crate::api::transform::strip_doc_tags_if_only_one_set;
use crate::api::types::{ModelResponse, TemplateKey};

pub struct OllamaClient {}
impl OllamaClient { pub fn new() -> Self { Self {} } }

impl ApiClient for OllamaClient {
  fn template(&self) -> TemplateKey { TemplateKey::Ollama }
  fn send_message(&mut self, model: &str, log: &MessageLog) -> Result<ModelResponse> {
    let msgs = build_ollama_messages(log)?;
    let body = serde_json::json!({
      "model": model,
      "messages": msgs,
      "stream": false,
    });
    let http = crate::http::http_client()?;
    let base = std::env::var("OLLAMA_API_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());
    let ollama_api_url = format!("{}/v1/chat/completions", base.trim_end_matches('/'));
    let resp = http
      .post(&ollama_api_url)
      .json(&body)
      .send()?;
    if !resp.status().is_success() {
      let status = resp.status();
      let txt = resp.text().unwrap_or_default();
      return Err(anyhow!("Ollama API error: {} - {}", status, txt));
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

fn build_ollama_messages(log: &MessageLog) -> Result<Vec<serde_json::Value>> {
  let raw = log.raw();
  let mut out: Vec<serde_json::Value> = Vec::new();
  let mut user_buf = String::new();
  for m in raw.iter() {
    match m.role {
      Role::User => {
        use std::fmt::Write as _;
        match m.kind {
          MsgType::Text => { let _ = write!(user_buf, "<doc>{}</doc>", m.content); }
          MsgType::Image => { /* ignore images in simple fallback */ }
        }
      }
      Role::Model => {
        if !user_buf.is_empty() {
          out.push(serde_json::json!({"role":"user","content": strip_doc_tags_if_only_one_set(&user_buf)}));
          user_buf.clear();
        }
        out.push(serde_json::json!({"role":"assistant","content": m.content}));
      }
    }
  }
  if !user_buf.is_empty() {
    out.push(serde_json::json!({"role":"user","content": strip_doc_tags_if_only_one_set(&user_buf)}));
  }
  Ok(out)
}

pub fn list_ollama_models() -> Result<Vec<String>> {
  #[derive(serde::Deserialize)]
  struct OAId { id: String }
  #[derive(serde::Deserialize)]
  struct OAData { data: Vec<OAId> }

  let http = crate::http::http_client()?;
  let base = std::env::var("OLLAMA_API_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());
  let url_v1 = format!("{}/v1/models", base.trim_end_matches('/'));

  let resp = http.get(&url_v1).send()?;
  if !resp.status().is_success() {
    let status = resp.status();
    let txt = resp.text().unwrap_or_default();
    return Err(anyhow!("Failed to fetch Ollama models: {} - {}", status, txt));
  }
  let parsed: OAData = resp.json()?;
  let models: Vec<String> = parsed.data.into_iter().map(|m| m.id).collect();
  Ok(models)
}
