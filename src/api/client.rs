use anyhow::Result;

use crate::message_log::MessageLog;

use super::clients::{AnthropicClient, GoogleClient, MistralClient, OllamaClient, OpenAIClient, PerplexityClient};
use super::traits::ApiClient as _; // trait methods on enum dispatch
use super::types::{ModelResponse, TemplateKey};

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
