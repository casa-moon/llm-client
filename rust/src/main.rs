mod api;
mod extractor;
mod message_log;
mod session;
mod ui;

use anyhow::Result;
use dotenvy::{dotenv, from_path};
use std::path::Path;

fn main() -> Result<()> {
  // Load environment variables from .env if present
  let _ = dotenv();
  
  ui::run_app()
}
