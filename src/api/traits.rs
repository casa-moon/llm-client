use anyhow::Result;

use crate::message_log::MessageLog;

use super::types::{ModelResponse, TemplateKey};

pub trait ApiClient {
  fn name(&self) -> &'static str;
  fn template(&self) -> TemplateKey;
  fn send_message(&mut self, model: &str, log: &MessageLog) -> Result<ModelResponse>;
}

