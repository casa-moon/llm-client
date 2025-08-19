mod api;
mod extractor;
mod http;
mod message_log;
mod session;
mod spinner;
mod ui;

use anyhow::Result;
use dotenvy::from_path;
use std::path::PathBuf;

fn main() -> Result<()> {
  let home_env = dirs::home_dir()
    .map(|home| home.join(".llm-client/.env"))
    .unwrap_or(PathBuf::from("/nonexistent")); // fallback to a non-existent path

  from_path(".env").or_else(|_| from_path(&home_env))?;
  ui::run_app()
}
