use anyhow::Result;

use crate::message_log::MessageLog;
use crate::api::clients::{AnthropicClient, GoogleClient, MistralClient, OllamaClient, OpenAIClient, PerplexityClient};
use crate::api::traits::ApiClient as _; // trait methods on enum dispatch
use crate::api::types::{ModelResponse};

pub enum Client {
  OpenAI(OpenAIClient),
  Google(GoogleClient),
  Anthropic(AnthropicClient),
  Perplexity(PerplexityClient),
  Mistral(MistralClient),
  Ollama(OllamaClient),
}

impl Client {
  pub fn send_message(&mut self, model: &str, log: &MessageLog) -> Result<ModelResponse> {
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

// create_client moved to src/api/factory.rs
