use serde_json::Value;

#[derive(Debug, Clone, Copy)]
pub enum TemplateKey {
  OpenAI,
  Anthropic,
  Google,
  Mistral,
  Perplexity,
  Ollama,
}

#[derive(Debug, Clone)]
pub struct ModelResponse {
  pub raw: Value,
  pub text: String,
}

pub struct ApiChoice {
  pub key: &'static str,
  pub label: &'static str,
  pub env: &'static str,
  pub model: &'static str,
}

pub const API_CHOICES: &[ApiChoice] = &[
  ApiChoice { key: "1", label: "gpt-5-nano", env: "OPENAI_API_KEY", model: "gpt-5-nano" },
  ApiChoice { key: "2", label: "gpt-5", env: "OPENAI_API_KEY", model: "gpt-5" },
  ApiChoice { key: "3", label: "gemini-2.5-pro", env: "GOOGLE_AI_API_KEY", model: "gemini-1.5-pro" },
  ApiChoice { key: "4", label: "claude-3-5-sonnet-20240620", env: "ANTHROPIC_API_KEY", model: "claude-3-5-sonnet-20240620" },
  ApiChoice { key: "5", label: "llama-3.1-sonar-large-128k-chat", env: "PERPLEXITY_API_KEY", model: "llama-3.1-sonar-large-128k-chat" },
  ApiChoice { key: "6", label: "mistral-medium", env: "MISTRAL_API_KEY", model: "mistral-medium" },
  ApiChoice { key: "7", label: "ollama", env: "OLLAMA_API_KEY", model: "ollama" },
];

