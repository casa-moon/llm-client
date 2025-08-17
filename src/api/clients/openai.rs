use anyhow::{anyhow, Result};

use crate::message_log::MessageLog;
use crate::api::traits::ApiClient;
use crate::api::transform::transform_messages;
use crate::api::types::{ModelResponse, TemplateKey};

pub struct OpenAIClient {
  pub api_key: String,
}

impl OpenAIClient {
  pub fn new(api_key: String) -> Self { Self { api_key } }

  // Sora video generation helper
  pub fn generate_sora_video(
    &self,
    prompt: &str,
    duration_seconds: Option<u32>,
    fps: Option<u32>,
    resolution: Option<&str>,
  ) -> Result<Vec<u8>> {
    use base64::Engine as _;
    use reqwest::header::AUTHORIZATION;
    use reqwest::blocking::multipart;
    let http = crate::http::http_client()?;
    let model_name = std::env::var("OPENAI_SORA_MODEL").unwrap_or_else(|_| "sora-1".to_string());
    let endpoint = std::env::var("OPENAI_SORA_ENDPOINT")
      .unwrap_or_else(|_| "https://api.openai.com/v1/videos".to_string());

    // Prepare fixed values used to build multipart per attempt
    let d = duration_seconds.unwrap_or(5);
    let f = fps.unwrap_or(24);
    let n_frames = d.saturating_mul(f);
    // Ensure a valid size is always sent
    let size_val: String = match resolution {
      Some(s) if !s.trim().is_empty() => s.trim().to_string(),
      _ => "1280x720".to_string(),
    };

    // Single-endpoint call: send with `prompt`, retry with `input` on specific 400s
    let f = multipart::Form::new()
      .text("model", model_name.clone())
      .text("n_frames", n_frames.to_string())
      .text("size", size_val.clone())
      .text("prompt", prompt.to_string());
    let resp = http
      .post(&endpoint)
      .header(AUTHORIZATION, format!("Bearer {}", self.api_key))
      .multipart(f)
      .send()?;
    let mut raw: serde_json::Value = if resp.status().is_success() {
      resp.json()?
    } else {
      let status = resp.status();
      let txt = resp.text().unwrap_or_default();
      if status.as_u16() == 400 {
        let lower = txt.to_lowercase();
        if lower.contains("missing required parameter: 'input'") || lower.contains("unknown parameter: 'prompt'") {
          let f2 = multipart::Form::new()
            .text("model", model_name.clone())
            .text("n_frames", n_frames.to_string())
            .text("size", size_val.clone())
            .text("input", prompt.to_string());
          let r2 = http
            .post(&endpoint)
            .header(AUTHORIZATION, format!("Bearer {}", self.api_key))
            .multipart(f2)
            .send()?;
          if r2.status().is_success() { r2.json()? } else {
            let status2 = r2.status();
            let txt2 = r2.text().unwrap_or_default();
            return Err(anyhow!("OpenAI Sora API error: {} - {}", status2, txt2));
          }
        } else {
          return Err(anyhow!("OpenAI Sora API error: {} - {}", status, txt));
        }
      } else if status.as_u16() == 404 {
        return Err(anyhow!("404:{}", endpoint));
      } else {
        return Err(anyhow!("OpenAI Sora API error: {} - {}", status, txt));
      }
    };

    // Handle immediate URL or base64 in initial response
    if let Some(url) = raw.get("video").and_then(|v| v.get("url")).and_then(|u| u.as_str()) {
      // Download the file
      let bytes = http.get(url).send()?.bytes()?;
      return Ok(bytes.to_vec());
    }
    if let Some(b64) = raw.get("video").and_then(|v| v.get("b64")).and_then(|u| u.as_str()) {
      let bytes = base64::engine::general_purpose::STANDARD.decode(b64.as_bytes())?;
      return Ok(bytes);
    }

    // If async job, poll by id until ready
    if let Some(id) = raw.get("id").and_then(|s| s.as_str()).map(|s| s.to_string()) {
      // Best-effort polling within overall request timeout
      use std::thread;
      use std::time::Duration;
      for _ in 0..30 {
        thread::sleep(Duration::from_secs(2));
        // Poll using the same endpoint base with /{id}
        let poll_url = format!("{}/{}", endpoint.trim_end_matches('/'), id);
        let r = http
          .get(&poll_url)
          .header(AUTHORIZATION, format!("Bearer {}", self.api_key))
          .send()?;
        if !r.status().is_success() { continue; }
        let st: serde_json::Value = r.json()?;
        if st.get("status").and_then(|s| s.as_str()) == Some("succeeded") {
          if let Some(url) = st.get("video").and_then(|v| v.get("url")).and_then(|u| u.as_str()) {
            let bytes = http.get(url).send()?.bytes()?;
            return Ok(bytes.to_vec());
          }
          if let Some(b64) = st.get("video").and_then(|v| v.get("b64")).and_then(|u| u.as_str()) {
            let bytes = base64::engine::general_purpose::STANDARD.decode(b64.as_bytes())?;
            return Ok(bytes);
          }
        }
      }
      return Err(anyhow!("Timed out waiting for Sora video generation"));
    }

    Err(anyhow!("Unexpected Sora response format"))
  }
}

impl ApiClient for OpenAIClient {
  fn name(&self) -> &'static str { "openai" }
  fn template(&self) -> TemplateKey { TemplateKey::OpenAI }
  fn send_message(&mut self, model: &str, log: &MessageLog) -> Result<ModelResponse> {
    use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};

    let payload = transform_messages(log.raw(), self.template())?;
    let body = serde_json::json!({
      "model": model,
      "messages": payload,
    });

    let http = crate::http::http_client()?;
    let resp = http
      .post("https://api.openai.com/v1/chat/completions")
      .header(AUTHORIZATION, format!("Bearer {}", self.api_key))
      .header(CONTENT_TYPE, "application/json")
      .json(&body)
      .send()?;

    if !resp.status().is_success() {
      let status = resp.status();
      let txt = resp.text().unwrap_or_default();
      return Err(anyhow!("OpenAI API error: {} - {}", status, txt));
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
