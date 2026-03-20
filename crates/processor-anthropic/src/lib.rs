pub mod client;

use async_trait::async_trait;
use daily_briefing_core::input::{BoxError, InputData};
use daily_briefing_core::processor::Processor;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Anthropic client error: {0}")]
    Client(#[from] client::Error),
}

#[derive(Debug, Clone)]
pub struct AnthropicConfig {
    pub api_key: String,
    pub model: String,
    pub max_tokens: u32,
    pub prompt: String,
}

pub struct AnthropicProcessor {
    config: AnthropicConfig,
    client: client::AnthropicClient,
}

impl AnthropicProcessor {
    pub fn new(config: AnthropicConfig) -> Self {
        let client = client::AnthropicClient::new(&config.api_key);
        Self { config, client }
    }
}

#[async_trait]
impl Processor for AnthropicProcessor {
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
