#[cfg(test)]
mod tests;

use serde::Deserialize;
use std::fs;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to read config file '{path}': {source}")]
    ReadFile {
        path: String,
        source: std::io::Error,
    },
    #[error("Failed to parse config: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("Config validation error: {0}")]
    Validation(String),
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub inputs: Vec<InputConfig>,
    pub processor: ProcessorConfig,
    pub outputs: Vec<OutputConfig>,
    pub schedule: Option<ScheduleConfig>,
    pub webserver: Option<WebServerConfig>,
}

impl Config {
    pub fn new(path: &str) -> Result<Self, Error> {
        let contents = fs::read_to_string(path).map_err(|source| Error::ReadFile {
            path: path.to_string(),
            source,
        })?;
        let config: Config = toml::from_str(&contents)?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), Error> {
        if self.inputs.is_empty() {
            return Err(Error::Validation(
                "At least one input must be configured".to_string(),
            ));
        }
        if self.outputs.is_empty() {
            return Err(Error::Validation(
                "At least one output must be configured".to_string(),
            ));
        }
        let needs_webserver = self.outputs.iter().any(|o| {
            matches!(o, OutputConfig::Webpage(_) | OutputConfig::Speech(_))
        });
        if needs_webserver && self.webserver.is_none() {
            return Err(Error::Validation(
                "webpage and speech outputs require a [webserver] section".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct ScheduleConfig {
    /// Standard 5-field cron expression (min hour dom month dow).
    /// Examples: "0 8 * * *" (daily at 08:00), "0 6 * * 1-5" (weekdays at 06:00).
    pub cron: String,
}

// ── Web server ────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct WebServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    pub port: u16,
    /// Name of the environment variable that holds the basic auth username.
    pub username_env: String,
    /// Name of the environment variable that holds the basic auth password.
    pub password_env: String,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

// ── Inputs ──────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InputConfig {
    Fever(FeverInputConfig),
    HomeAssistantMetrics(HaMetricsInputConfig),
}

#[derive(Debug, Deserialize)]
pub struct FeverInputConfig {
    pub name: String,
    pub url: String,
    /// Name of the environment variable that holds the Fever API username.
    pub username_env: String,
    /// Name of the environment variable that holds the Fever API password.
    pub password_env: String,
    #[serde(default = "default_hours")]
    pub hours: u64,
    /// If true, collected items are marked as read in the Fever API after fetching (default: false).
    #[serde(default)]
    pub mark_as_read: bool,
}

fn default_hours() -> u64 {
    24
}

#[derive(Debug, Deserialize)]
pub struct HaMetricsInputConfig {
    pub name: String,
    pub url: String,
    /// Name of the environment variable that holds the HA long-lived access token.
    pub token_env: String,
    #[serde(default = "default_hours")]
    pub hours: u64,
    pub entities: Vec<HaMetricsEntityConfig>,
}

#[derive(Debug, Deserialize)]
pub struct HaMetricsEntityConfig {
    pub id: String,
    pub label: Option<String>,
    /// `"cumulative"` (default) for sensors like energy counters where the delta matters.
    /// `"gauge"` for instantaneous sensors like battery level where the latest value matters.
    #[serde(default)]
    pub kind: EntityKind,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EntityKind {
    #[default]
    Cumulative,
    Gauge,
}

// ── Processor ────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProcessorConfig {
    Anthropic(AnthropicProcessorConfig),
}

impl ProcessorConfig {
    pub fn type_name(&self) -> &str {
        match self {
            ProcessorConfig::Anthropic(_) => "anthropic",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AnthropicProcessorConfig {
    /// Name of the environment variable that holds the Anthropic API key.
    pub api_key_env: String,
    pub model: String,
    pub max_tokens: u32,
    pub prompt: String,
}

// ── Outputs ──────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutputConfig {
    HomeAssistant(HomeAssistantOutputConfig),
    Webpage(WebpageOutputConfig),
    Speech(SpeechOutputConfig),
}

#[derive(Debug, Deserialize)]
pub struct HomeAssistantOutputConfig {
    pub name: String,
    pub url: String,
    /// Name of the environment variable that holds the HA long-lived access token.
    pub token_env: String,
    pub title: String,
    pub notification_id: String,
}

#[derive(Debug, Deserialize)]
pub struct WebpageOutputConfig {
    pub name: String,
    /// Endpoint path served by the shared web server, e.g. `"/"`.
    pub endpoint: String,
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct SpeechOutputConfig {
    pub name: String,
    /// Endpoint path served by the shared web server, e.g. `"/briefing.wav"`.
    pub endpoint: String,
    /// Path to the Piper `.onnx` voice model file.
    pub model: String,
    /// Speaker ID for multi-speaker models (omit for single-speaker models).
    pub speaker: Option<u32>,
}
