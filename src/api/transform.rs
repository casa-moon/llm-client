use anyhow::Result;

use crate::message_log::{Message, MsgType, Role};
use crate::api::types::TemplateKey;

pub fn transform_messages(raw: &Vec<Message>, tmpl: TemplateKey) -> Result<serde_json::Value> {
  match tmpl {
    TemplateKey::OpenAI => {
      let msgs: Vec<serde_json::Value> = raw
        .iter()
        .map(|m| {
          let role = match m.role {
            Role::Model => "assistant",
            Role::User => "user",
          };
          match m.kind {
            MsgType::Text => serde_json::json!({
              "role": role,
              "content": [{"type": "text", "text": m.content}],
            }),
            MsgType::Image => serde_json::json!({
              "role": role,
              "content": [{"type": "image_url", "image_url": {"url": m.content}}],
            }),
          }
        })
        .collect();
      Ok(serde_json::Value::Array(msgs))
    }
    TemplateKey::Anthropic | TemplateKey::Google | TemplateKey::Mistral | TemplateKey::Perplexity | TemplateKey::Ollama => {
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
                  "parts": [{"text": strip_doc_tags_if_only_one_set(&user_buf)}],
                })),
                TemplateKey::Perplexity | TemplateKey::Ollama => out.push(serde_json::json!({
                  "role": "user",
                  "content": [{"type": "text", "text": strip_doc_tags_if_only_one_set(&user_buf)}],
                })),
                _ => out.push(serde_json::json!({
                  "role": "user",
                  "content": strip_doc_tags_if_only_one_set(&user_buf),
                })),
              }
              user_buf.clear();
            }
            match tmpl {
              TemplateKey::Google => out.push(serde_json::json!({
                "role": "model",
                "parts": [{"text": m.content}],
              })),
              TemplateKey::Perplexity | TemplateKey::Ollama => out.push(serde_json::json!({
                "role": "assistant",
                "content": [{"type": "text", "text": m.content}],
              })),
              _ => out.push(serde_json::json!({
                "role": "assistant",
                "content": m.content,
              })),
            }
          }
        }
      }
      if !user_buf.is_empty() {
        match tmpl {
          TemplateKey::Google => out.push(serde_json::json!({
            "role": "user",
            "parts": [{"text": strip_doc_tags_if_only_one_set(&user_buf)}],
          })),
          TemplateKey::Perplexity | TemplateKey::Ollama => out.push(serde_json::json!({
            "role": "user",
            "content": [{"type": "text", "text": strip_doc_tags_if_only_one_set(&user_buf)}],
          })),
          _ => out.push(serde_json::json!({
            "role": "user",
            "content": strip_doc_tags_if_only_one_set(&user_buf),
          })),
        }
      }
      // For Google Gemini, ensure the last message remains a user turn.
      // Do not append synthetic "model: continue" which can confuse turn-taking.
      Ok(serde_json::Value::Array(out))
    }
  }
}

pub(crate) fn strip_doc_tags_if_only_one_set(s: &str) -> String {
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
