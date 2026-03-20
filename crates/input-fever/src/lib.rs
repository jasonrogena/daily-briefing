pub mod client;

use async_trait::async_trait;
use daily_briefing_core::input::{BoxError, Input, InputData};
use thiserror::Error;
use tracing::warn;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Fever client error: {0}")]
    Client(#[from] client::Error),
    #[error("No items found in the configured time window")]
    NoItems,
}

#[derive(Debug, Clone)]
pub struct FeverConfig {
    pub name: String,
    pub url: String,
    pub username: String,
    pub password: String,
    pub hours: u64,
    /// If true, collected items are marked as read after fetching.
    pub mark_as_read: bool,
}

pub struct FeverInput {
    config: FeverConfig,
    client: client::FeverClient,
}

impl FeverInput {
    pub fn new(config: FeverConfig) -> Self {
        let client = client::FeverClient::new(&config.url, &config.username, &config.password);
        Self { config, client }
    }
}

#[async_trait]
impl Input for FeverInput {
    fn name(&self) -> &str {
        &self.config.name
    }

    async fn collect(&self) -> Result<InputData, BoxError> {
        let (text, ids) = self
            .client
            .fetch_recent_items(self.config.hours, self.config.mark_as_read)
            .await?;

        if self.config.mark_as_read && !ids.is_empty() {
            if let Err(e) = self.client.mark_items_as_read(&ids).await {
                warn!(input = self.config.name, "Failed to mark items as read: {}", e);
            }
        }

        Ok(InputData {
            source: self.config.name.clone(),
            content: text,
        })
    }
}
