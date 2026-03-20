use reqwest::Client;
use serde::Serialize;
use thiserror::Error;
use tracing::{debug, info};

#[cfg(test)]
mod tests;

#[derive(Debug, Error)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Home Assistant API error: HTTP {status}")]
    Api { status: u16 },
}

#[derive(Debug, Serialize)]
struct CreateNotificationRequest<'a> {
    notification_id: &'a str,
    title: &'a str,
    message: &'a str,
}

pub struct HaClient {
    base_url: String,
    token: String,
    http: Client,
}

impl HaClient {
    pub fn new(base_url: &str, token: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            token: token.to_string(),
            http: Client::new(),
        }
    }

    pub async fn create_notification(
        &self,
        notification_id: &str,
        title: &str,
        message: &str,
    ) -> Result<(), Error> {
        let url = format!(
            "{}/api/services/persistent_notification/create",
            self.base_url
        );
        info!(notification_id, title, "Creating Home Assistant notification");
        debug!(chars = message.len(), message, "Notification message");

        let body = CreateNotificationRequest {
            notification_id,
            title,
            message,
        };

        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            return Err(Error::Api {
                status: status.as_u16(),
            });
        }

        debug!(notification_id, "Notification created successfully");
        Ok(())
    }
}
