mod api;
mod http;
mod extractor;
mod message_log;
mod session;
mod ui;

use anyhow::Result;
use dotenvy::{dotenv};

fn main() -> Result<()> {
  // Load environment variables from .env if present
  let _ = dotenv();
  
  ui::run_app()
}
