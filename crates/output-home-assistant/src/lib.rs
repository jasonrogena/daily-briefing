pub mod client;

use async_trait::async_trait;
use daily_briefing_core::input::BoxError;
use daily_briefing_core::output::Output;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Home Assistant client error: {0}")]
    Client(#[from] client::Error),
}

#[derive(Debug, Clone)]
pub struct HomeAssistantConfig {
    pub name: String,
    pub url: String,
    pub token: String,
    pub title: String,
    pub notification_id: String,
}

pub struct HomeAssistantOutput {
    config: HomeAssistantConfig,
    client: client::HaClient,
}

impl HomeAssistantOutput {
    pub fn new(config: HomeAssistantConfig) -> Self {
        let client = client::HaClient::new(&config.url, &config.token);
        Self { config, client }
    }
}

#[async_trait]
impl Output for HomeAssistantOutput {
    fn name(&self) -> &str {
        &self.config.name
    }

    async fn write(&self, content: &str) -> Result<(), BoxError> {
        self.client
            .create_notification(&self.config.notification_id, &self.config.title, content)
            .await?;
        Ok(())
    }
}
