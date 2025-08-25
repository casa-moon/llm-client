mod client;
pub mod clients;
mod factory;
pub mod traits;
mod transform;
pub mod types;

pub use client::Client;
pub use clients::ollama::list_ollama_models;
pub use factory::create_client;
pub use types::{ModelResponse, API_CHOICES};
