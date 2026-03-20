use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, info};

#[cfg(test)]
mod tests;

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";

#[derive(Debug, Error)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Anthropic API error: {0}")]
    Api(String),
    #[error("Empty response from Anthropic API")]
    EmptyResponse,
}

#[derive(Debug, Serialize)]
struct RequestBody<'a> {
    model: &'a str,
    max_tokens: u32,
    messages: Vec<Message<'a>>,
}

#[derive(Debug, Serialize)]
struct Message<'a> {
    role: &'a str,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ResponseBody {
    content: Vec<ContentBlock>,
    #[serde(default)]
    error: Option<ApiError>,
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiError {
    message: String,
}

pub struct AnthropicClient {
    api_key: String,
    http: Client,
}

impl AnthropicClient {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            http: Client::new(),
        }
    }

    pub async fn send_message(
        &self,
        model: &str,
        max_tokens: u32,
        prompt: &str,
        content: &str,
    ) -> Result<String, Error> {
        info!(model, max_tokens, "Sending request to Anthropic API");
        debug!(content_chars = content.len(), "Content length");
        let body = RequestBody {
            model,
            max_tokens,
            messages: vec![Message {
                role: "user",
                content: format!("{}\n\n{}", prompt, content),
            }],
        };

        let resp = self
            .http
            .post(ANTHROPIC_API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        let response_body: ResponseBody = resp.json().await?;

        if let Some(err) = response_body.error {
            return Err(Error::Api(format!("HTTP {}: {}", status, err.message)));
        }

        let text = response_body
            .content
            .into_iter()
            .find(|b| b.block_type == "text")
            .and_then(|b| b.text)
            .ok_or(Error::EmptyResponse)?;

        info!(chars = text.len(), "Received response from Anthropic API");
        Ok(text)
    }
}
