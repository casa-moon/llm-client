pub mod clients;
mod client;
mod factory;
mod transform;
pub mod traits;
pub mod types;

pub use client::Client;
pub use factory::create_client;
pub use clients::ollama::list_ollama_models;
pub use types::{ModelResponse, API_CHOICES};
