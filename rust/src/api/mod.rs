use crate::message_log::{Message, MessageLog, MsgType, Role};
use anyhow::{anyhow, Result};

#[derive(Debug, Clone, Copy)]
pub enum TemplateKey {
  OpenAI,
  Anthropic,
  Google,
  Mistral,
  Perplexity,
  Ollama,
}

pub trait ApiClient {
  fn name(&self) -> &'static str;
  fn template(&self) -> TemplateKey;
  fn send_message(&mut self, model: &str, log: &MessageLog) -> Result<String>;
}

pub struct OpenAIClient {
  pub api_key: String,
}

impl OpenAIClient {
  pub fn new(api_key: String) -> Self { Self { api_key } }
}

impl ApiClient for OpenAIClient {
  fn name(&self) -> &'static str { "openai" }
  fn template(&self) -> TemplateKey { TemplateKey::OpenAI }
  fn send_message(&mut self, model: &str, log: &MessageLog) -> Result<String> {
    use reqwest::blocking::Client as HttpClient;
    use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};

    let payload = transform_messages(log.raw(), self.template())?;
    let body = serde_json::json!({
            "model": model,
            "messages": payload
        });

    let http = HttpClient::new();
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

    #[derive(serde::Deserialize)]
    struct ChatChoiceMsg {
      content: String,
    }
    #[derive(serde::Deserialize)]
    struct ChatChoice {
      message: ChatChoiceMsg,
    }
    #[derive(serde::Deserialize)]
    struct ChatResp {
      choices: Vec<ChatChoice>,
    }

    let parsed: ChatResp = resp.json()?;
    let text = parsed
      .choices
      .get(0)
      .map(|c| c.message.content.clone())
      .unwrap_or_default();
    Ok(text)
  }
}

pub struct AnthropicClient {
  pub api_key: String,
}
impl AnthropicClient { pub fn new(api_key: String) -> Self { Self { api_key } } }
impl ApiClient for AnthropicClient {
  fn name(&self) -> &'static str { "anthropic" }
  fn template(&self) -> TemplateKey { TemplateKey::Anthropic }
  fn send_message(&mut self, model: &str, log: &MessageLog) -> Result<String> {
    use reqwest::blocking::Client as HttpClient;
    let payload = transform_messages(log.raw(), self.template())?;
    // For Anthropics Messages API, messages is a list with role and content string
    let body = serde_json::json!({
            "model": model,
            "max_tokens": 4096,
            "temperature": 0,
            "messages": payload
        });
    let http = HttpClient::new();
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
    #[derive(serde::Deserialize)]
    struct ContentItem {
      text: Option<String>,
    }
    #[derive(serde::Deserialize)]
    struct AnthResp {
      content: Vec<ContentItem>,
    }
    let parsed: AnthResp = resp.json()?;
    let text = parsed.content.get(0).and_then(|c| c.text.clone()).unwrap_or_default();
    Ok(text)
  }
}

pub struct GoogleClient {
  pub api_key: String,
}
impl GoogleClient { pub fn new(api_key: String) -> Self { Self { api_key } } }
impl ApiClient for GoogleClient {
  fn name(&self) -> &'static str { "google" }
  fn template(&self) -> TemplateKey { TemplateKey::Google }
  fn send_message(&mut self, model: &str, log: &MessageLog) -> Result<String> {
    use reqwest::blocking::Client as HttpClient;
    let contents = transform_messages(log.raw(), self.template())?;
    let body = serde_json::json!({
            "contents": contents,
            "generationConfig": { "maxOutputTokens": 2048 }
        });

    let url = format!(
      "https://generativelanguage.googleapis.com/v1/models/{}:generateContent?key={}",
      model, self.api_key
    );
    let http = HttpClient::new();
    let resp = http.post(&url).json(&body).send()?;
    if !resp.status().is_success() {
      let status = resp.status();
      let txt = resp.text().unwrap_or_default();
      return Err(anyhow!("Google API error: {} - {}", status, txt));
    }
    #[derive(serde::Deserialize)]
    struct Part {
      text: Option<String>,
    }
    #[derive(serde::Deserialize)]
    struct Content {
      parts: Vec<Part>,
    }
    #[derive(serde::Deserialize)]
    struct Candidate {
      content: Content,
    }
    #[derive(serde::Deserialize)]
    struct Resp {
      candidates: Vec<Candidate>,
    }

    let parsed: Resp = resp.json()?;
    let text = parsed
      .candidates
      .get(0)
      .map(|c| c.content.parts.iter().filter_map(|p| p.text.clone()).collect::<Vec<_>>().join("\n"))
      .unwrap_or_default();
    Ok(text)
  }
}

pub struct PerplexityClient {
  pub api_key: String,
}
impl PerplexityClient { pub fn new(api_key: String) -> Self { Self { api_key } } }
impl ApiClient for PerplexityClient {
  fn name(&self) -> &'static str { "perplexity" }
  fn template(&self) -> TemplateKey { TemplateKey::Perplexity }
  fn send_message(&mut self, model: &str, log: &MessageLog) -> Result<String> {
    use reqwest::blocking::Client as HttpClient;
    use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
    let payload = transform_messages(log.raw(), self.template())?;
    let body = serde_json::json!({
            "model": model,
            "max_tokens": 4096,
            "temperature": 0,
            "messages": payload
        });
    let http = HttpClient::new();
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
    #[derive(serde::Deserialize)]
    struct ChatChoiceMsg {
      content: String,
    }
    #[derive(serde::Deserialize)]
    struct ChatChoice {
      message: ChatChoiceMsg,
    }
    #[derive(serde::Deserialize)]
    struct ChatResp {
      choices: Vec<ChatChoice>,
    }
    let parsed: ChatResp = resp.json()?;
    let text = parsed
      .choices
      .get(0)
      .map(|c| c.message.content.clone())
      .unwrap_or_default();
    Ok(text)
  }
}

pub struct MistralClient {
  pub api_key: String,
}
impl MistralClient { pub fn new(api_key: String) -> Self { Self { api_key } } }
impl ApiClient for MistralClient {
  fn name(&self) -> &'static str { "mistral" }
  fn template(&self) -> TemplateKey { TemplateKey::Mistral }
  fn send_message(&mut self, model: &str, log: &MessageLog) -> Result<String> {
    use reqwest::blocking::Client as HttpClient;
    use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
    let payload = transform_messages(log.raw(), self.template())?;
    let body = serde_json::json!({
            "model": model,
            "max_tokens": 4096,
            "temperature": 0,
            "messages": payload
        });
    let http = HttpClient::new();
    let resp = http
      .post("https://api.mistral.ai/v1/chat/completions")
      .header(AUTHORIZATION, format!("Bearer {}", self.api_key))
      .header(CONTENT_TYPE, "application/json")
      .json(&body)
      .send()?;
    if !resp.status().is_success() {
      let status = resp.status();
      let txt = resp.text().unwrap_or_default();
      return Err(anyhow!("Mistral API error: {} - {}", status, txt));
    }
    #[derive(serde::Deserialize)]
    struct ChatChoiceMsg {
      content: String,
    }
    #[derive(serde::Deserialize)]
    struct ChatChoice {
      message: ChatChoiceMsg,
    }
    #[derive(serde::Deserialize)]
    struct ChatResp {
      choices: Vec<ChatChoice>,
    }
    let parsed: ChatResp = resp.json()?;
    let text = parsed
      .choices
      .get(0)
      .map(|c| c.message.content.clone())
      .unwrap_or_default();
    Ok(text)
  }
}

pub struct OllamaClient {}
impl OllamaClient { pub fn new() -> Self { Self {} } }
impl ApiClient for OllamaClient {
  fn name(&self) -> &'static str { "ollama" }
  fn template(&self) -> TemplateKey { TemplateKey::Ollama }
  fn send_message(&mut self, model: &str, log: &MessageLog) -> Result<String> {
    use reqwest::blocking::Client as HttpClient;
    // Build simple role+content messages for Ollama chat
    let msgs = build_ollama_messages(log)?;
    let body = serde_json::json!({
            "model": model,
            "messages": msgs,
            "stream": false
        });
    let http = HttpClient::new();
    let resp = http
      .post("http://localhost:11434/api/chat")
      .json(&body)
      .send()?;
    if !resp.status().is_success() {
      let status = resp.status();
      let txt = resp.text().unwrap_or_default();
      return Err(anyhow!("Ollama API error: {} - {}", status, txt));
    }
    #[derive(serde::Deserialize)]
    struct Message {
      content: String,
    }
    #[derive(serde::Deserialize)]
    struct Resp {
      message: Message,
    }
    let parsed: Resp = resp.json()?;
    Ok(parsed.message.content)
  }
}

fn build_ollama_messages(log: &MessageLog) -> Result<Vec<serde_json::Value>> {
  // Collapse user text blocks into <doc> segments like templates, then emit plain text
  let raw = log.raw();
  let mut out: Vec<serde_json::Value> = Vec::new();
  let mut user_buf = String::new();
  for m in raw.iter() {
    match m.role {
      Role::User => {
        use std::fmt::Write as _;
        match m.kind {
          MsgType::Text => { let _ = write!(user_buf, "<doc>{}</doc>", m.content); }
          MsgType::Image => { /* Images not handled in Ollama text fallback here */ }
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
  use reqwest::blocking::Client as HttpClient;
  #[derive(serde::Deserialize)]
  struct Tag {
    name: String,
  }
  #[derive(serde::Deserialize)]
  struct Tags {
    models: Vec<Tag>,
  }
  let http = HttpClient::new();
  let resp = http.get("http://localhost:11434/api/tags").send()?;
  if !resp.status().is_success() {
    let status = resp.status();
    let txt = resp.text().unwrap_or_default();
    return Err(anyhow!("Ollama tags error: {} - {}", status, txt));
  }
  let parsed: Tags = resp.json()?;
  Ok(parsed.models.into_iter().map(|t| t.name).collect())
}

pub fn transform_messages(raw: &Vec<Message>, tmpl: TemplateKey) -> Result<serde_json::Value> {
  match tmpl {
    TemplateKey::OpenAI => {
      let msgs: Vec<serde_json::Value> = raw
        .iter()
        .map(|m| {
          let role = match m.role {
            Role::Model => "assistant",
            Role::User => "user"
          };
          match m.kind {
            MsgType::Text => serde_json::json!({
                            "role": role,
                            "content": [{"type": "text", "text": m.content}]
                        }),
            MsgType::Image => serde_json::json!({
                            "role": role,
                            "content": [{"type": "image_url", "image_url": {"url": m.content}}]
                        }),
          }
        })
        .collect();
      Ok(serde_json::Value::Array(msgs))
    }
    TemplateKey::Anthropic | TemplateKey::Google | TemplateKey::Mistral | TemplateKey::Perplexity | TemplateKey::Ollama => {
      // Implementing the full doc-tag logic is possible; keep simple for initial port
      let mut out: Vec<serde_json::Value> = Vec::new();
      let mut user_buf = String::new();
      for m in raw.iter() {
        match m.role {
          Role::User => {
            use std::fmt::Write as _;
            let _ = write!(user_buf, "<doc>{}</doc>", m.content);
          }
          Role::Model => {
            if !user_buf.is_empty() {
              match tmpl {
                TemplateKey::Google => out.push(serde_json::json!({
                                    "role": "user",
                                    "parts": [{"text": strip_doc_tags_if_only_one_set(&user_buf)}]
                                })),
                TemplateKey::Perplexity | TemplateKey::Ollama => out.push(serde_json::json!({
                                    "role": "user",
                                    "content": [{"type": "text", "text": strip_doc_tags_if_only_one_set(&user_buf)}]
                                })),
                _ => out.push(serde_json::json!({
                                    "role": "user",
                                    "content": strip_doc_tags_if_only_one_set(&user_buf)
                                })),
              }
              user_buf.clear();
            }
            match tmpl {
              TemplateKey::Google => out.push(serde_json::json!({
                                "role": "model",
                                "parts": [{"text": m.content}]
                            })),
              TemplateKey::Perplexity | TemplateKey::Ollama => out.push(serde_json::json!({
                                "role": "assistant",
                                "content": [{"type": "text", "text": m.content}]
                            })),
              _ => out.push(serde_json::json!({
                                "role": "assistant",
                                "content": m.content
                            })),
            }
          }
        }
      }
      if !user_buf.is_empty() {
        match tmpl {
          TemplateKey::Google => out.push(serde_json::json!({
                        "role": "user",
                        "parts": [{"text": strip_doc_tags_if_only_one_set(&user_buf)}]
                    })),
          TemplateKey::Perplexity | TemplateKey::Ollama => out.push(serde_json::json!({
                        "role": "user",
                        "content": [{"type": "text", "text": strip_doc_tags_if_only_one_set(&user_buf)}]
                    })),
          _ => out.push(serde_json::json!({
                        "role": "user",
                        "content": strip_doc_tags_if_only_one_set(&user_buf)
                    })),
        }
      }
      if matches!(tmpl, TemplateKey::Google) {
        out.push(serde_json::json!({"role": "model", "parts": [{"text": "continue"}]}));
      }
      Ok(serde_json::Value::Array(out))
    }
  }
}

fn strip_doc_tags_if_only_one_set(s: &str) -> String {
  let count = s.matches("<doc>").count();
  if count == 1 {
    s.replace("<doc>", "").replace("</doc>", "")
  } else if count >= 2 {
    if let (Some(lo), Some(lc)) = (s.rfind("<doc>"), s.rfind("</doc>")) {
      let mut out = String::new();
      out.push_str(&s[..lo]);
      out.push_str(&s[lo + 5..lc]);
      out
    } else {
      s.to_string()
    }
  } else {
    s.to_string()
  }
}

pub struct ApiChoice {
  pub key: &'static str,
  pub label: &'static str,
  pub env: &'static str,
  pub model: &'static str,
}

pub const API_CHOICES: &[ApiChoice] = &[
  ApiChoice { key: "1", label: "gpt-5", env: "OPENAI_API_KEY", model: "gpt-5" },
  ApiChoice { key: "2", label: "gpt-4.1", env: "OPENAI_API_KEY", model: "gpt-4.1" },
  ApiChoice { key: "3", label: "gemini-1.5-pro-latest", env: "GOOGLE_AI_API_KEY", model: "gemini-1.5-pro-latest" },
  ApiChoice { key: "4", label: "claude-3-5-sonnet-20240620", env: "ANTHROPIC_API_KEY", model: "claude-3-5-sonnet-20240620" },
  ApiChoice { key: "5", label: "llama-3.1-sonar-large-128k-chat", env: "PERPLEXITY_API_KEY", model: "llama-3.1-sonar-large-128k-chat" },
  ApiChoice { key: "6", label: "mistral-medium", env: "MISTRAL_API_KEY", model: "mistral-medium" },
  ApiChoice { key: "7", label: "ollama", env: "OLLAMA_API_KEY", model: "ollama" },
];

pub enum Client {
  OpenAI(OpenAIClient),
  Google(GoogleClient),
  Anthropic(AnthropicClient),
  Perplexity(PerplexityClient),
  Mistral(MistralClient),
  Ollama(OllamaClient),
}

impl Client {
  pub fn name(&self) -> &'static str {
    match self {
      Client::OpenAI(_) => "openai",
      Client::Google(_) => "google",
      Client::Anthropic(_) => "anthropic",
      Client::Perplexity(_) => "perplexity",
      Client::Mistral(_) => "mistral",
      Client::Ollama(_) => "ollama",
    }
  }

  pub fn template(&self) -> TemplateKey {
    match self {
      Client::OpenAI(_) => TemplateKey::OpenAI,
      Client::Google(_) => TemplateKey::Google,
      Client::Anthropic(_) => TemplateKey::Anthropic,
      Client::Perplexity(_) => TemplateKey::Perplexity,
      Client::Mistral(_) => TemplateKey::Mistral,
      Client::Ollama(_) => TemplateKey::Ollama,
    }
  }

  pub fn send_message(&mut self, model: &str, log: &MessageLog) -> Result<String> {
    match self {
      Client::OpenAI(c) => c.send_message(model, log),
      Client::Google(c) => c.send_message(model, log),
      Client::Anthropic(c) => c.send_message(model, log),
      Client::Perplexity(c) => c.send_message(model, log),
      Client::Mistral(c) => c.send_message(model, log),
      Client::Ollama(c) => c.send_message(model, log),
    }
  }
}

pub fn create_client(choice_key: &str) -> Result<(Client, String)> {
  let cfg = API_CHOICES
    .iter()
    .find(|c| c.key == choice_key)
    .ok_or_else(|| anyhow!("Invalid API choice: {}", choice_key))?;
  let env_val = std::env::var(cfg.env)
    .map_err(|_| anyhow!("Missing API key for {}", cfg.env))?;

  let client = match cfg.key {
    "1" | "2" => Client::OpenAI(OpenAIClient::new(env_val)),
    "3" => Client::Google(GoogleClient::new(env_val)),
    "4" => Client::Anthropic(AnthropicClient::new(env_val)),
    "5" => Client::Perplexity(PerplexityClient::new(env_val)),
    "6" => Client::Mistral(MistralClient::new(env_val)),
    "7" => Client::Ollama(OllamaClient::new()),
    _ => unreachable!(),
  };
  Ok((client, cfg.model.to_string()))
}
