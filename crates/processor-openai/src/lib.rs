pub mod client;

use async_trait::async_trait;
use daily_briefing_core::input::{BoxError, InputData};
use daily_briefing_core::processor::Processor;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("OpenAI client error: {0}")]
    Client(#[from] client::Error),
}

#[derive(Debug, Clone)]
pub struct OpenAiConfig {
    /// Base URL for the OpenAI-compatible API, e.g. `https://api.openai.com/v1`
    /// or a local endpoint like `http://192.168.10.3:8080/v1`.
    /// Defaults to `https://api.openai.com/v1` if `None`.
    pub base_url: Option<String>,
    /// API key. Pass `None` for local endpoints that don't require authentication.
    pub api_key: Option<String>,
    pub model: String,
    pub max_tokens: u32,
    pub prompt: String,
}

pub struct OpenAiProcessor {
    config: OpenAiConfig,
    client: client::OpenAiClient,
}

impl OpenAiProcessor {
    pub fn new(config: OpenAiConfig) -> Self {
        let client = client::OpenAiClient::new(
            config.base_url.as_deref(),
            config.api_key.as_deref(),
        );
        Self { config, client }
    }
}

#[async_trait]
impl Processor for OpenAiProcessor {
    async fn process(&self, inputs: &[InputData]) -> Result<String, BoxError> {
        let combined: String = inputs
            .iter()
            .map(|i| format!("=== {} ===\n{}", i.source, i.content))
            .collect::<Vec<_>>()
            .join("\n\n");

        let result = self
            .client
            .send_message(
                &self.config.model,
                self.config.max_tokens,
                &self.config.prompt,
                &combined,
            )
            .await?;

        Ok(result)
    }
}
