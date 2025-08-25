use anyhow::{anyhow, Result};

use crate::api::client::Client;
use crate::api::clients::{
  AnthropicClient, GoogleClient, MistralClient, OllamaClient, OpenAIClient, PerplexityClient,
};
use crate::api::types::{ApiChoice, API_CHOICES};

pub fn create_client(choice_key: &str) -> Result<(Client, String)> {
  let cfg: &ApiChoice = API_CHOICES
    .iter()
    .find(|c| c.key == choice_key)
    .ok_or_else(|| anyhow!("Invalid API choice: {}", choice_key))?;

  let env_val = std::env::var(cfg.env).map_err(|_| anyhow!("Missing API key for {}", cfg.env))?;

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
