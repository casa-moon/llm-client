use anyhow::Result;

use crate::message_log::MessageLog;
use crate::api::types::{ModelResponse, TemplateKey};

pub trait ApiClient {
  fn template(&self) -> TemplateKey;
  fn send_message(&mut self, model: &str, log: &MessageLog) -> Result<ModelResponse>;
}
