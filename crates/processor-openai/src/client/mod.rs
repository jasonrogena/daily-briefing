use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, info};

#[cfg(test)]
mod tests;

const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";

#[derive(Debug, Error)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("OpenAI API error: {0}")]
    Api(String),
    #[error("Empty response from OpenAI API")]
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
    content: &'a str,
}

#[derive(Debug, Deserialize)]
struct ResponseBody {
    #[serde(default)]
    choices: Vec<Choice>,
    #[serde(default)]
    error: Option<ApiError>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ChoiceMessage,
}

#[derive(Debug, Deserialize)]
struct ChoiceMessage {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiError {
    message: String,
}

pub struct OpenAiClient {
    base_url: String,
    api_key: Option<String>,
    http: Client,
}

impl OpenAiClient {
    pub fn new(base_url: Option<&str>, api_key: Option<&str>) -> Self {
        Self {
            base_url: base_url
                .unwrap_or(DEFAULT_BASE_URL)
                .trim_end_matches('/')
                .to_string(),
            api_key: api_key.map(|s| s.to_string()),
            http: Client::new(),
        }
    }

    pub async fn send_message(
        &self,
        model: &str,
        max_tokens: u32,
        system_prompt: &str,
        content: &str,
    ) -> Result<String, Error> {
        let url = format!("{}/chat/completions", self.base_url);
        info!(model, max_tokens, "Sending request to OpenAI API");
        debug!(content_chars = content.len(), "Content length");

        let body = RequestBody {
            model,
            max_tokens,
            messages: vec![
                Message {
                    role: "system",
                    content: system_prompt,
                },
                Message {
                    role: "user",
                    content,
                },
            ],
        };

        let mut req = self.http.post(&url);
        if let Some(key) = &self.api_key {
            req = req.bearer_auth(key);
        }
        let resp = req.json(&body).send().await?;

        let status = resp.status();
        let response_body: ResponseBody = resp.json().await?;

        if let Some(err) = response_body.error {
            return Err(Error::Api(format!("HTTP {}: {}", status, err.message)));
        }

        let text = response_body
            .choices
            .into_iter()
            .next()
            .and_then(|c| c.message.content)
            .ok_or(Error::EmptyResponse)?;

        info!(chars = text.len(), "Received response from OpenAI API");
        Ok(text)
    }
}
