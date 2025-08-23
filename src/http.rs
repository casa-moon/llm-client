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

  let client = Client::builder()
    .timeout(Duration::from_secs(timeout_secs))
    .build()?;
  Ok(client)
}

