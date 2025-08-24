use anyhow::Result;
use reqwest::blocking::Client;
use std::time::Duration;

// Builds a blocking HTTP client with a default request timeout.
// Timeout can be overridden via REQUEST_TIMEOUT_SECS env var.
pub fn http_client() -> Result<Client> {
  let timeout_secs: u64 = std::env::var("REQUEST_TIMEOUT_SECS")
    .ok()
    .and_then(|s| s.parse::<u64>().ok())
    .unwrap_or(60);

  let default_ua = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0 Safari/537.36 llm-client/0.1";
  let client = Client::builder()
    .timeout(Duration::from_secs(timeout_secs))
    .user_agent(default_ua)
    .build()?;
  Ok(client)
}
