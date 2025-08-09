mod session;
mod message_log;
mod api;
mod ui;
mod extractor;
mod markdown;

use anyhow::Result;
use dotenvy::{dotenv, from_path};
use std::path::Path;

fn main() -> Result<()> {
    // Load environment variables from .env if present
    let _ = dotenv();
    // Also try loading from project javascript/.env for parity with JS app
    if Path::new("javascript/.env").exists() {
        let _ = from_path("javascript/.env");
    }

    ui::run_app()
}
