pub mod client;

use async_trait::async_trait;
use chrono::Utc;
use client::StateChange;
use daily_briefing_core::input::{BoxError, Input, InputData};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Home Assistant metrics client error: {0}")]
    Client(#[from] client::Error),
    #[error("No metrics data returned for any entity in either period")]
    NoData,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum EntityKind {
    #[default]
    Cumulative,
    Gauge,
}

#[derive(Debug, Clone)]
pub struct EntityConfig {
    pub id: String,
    pub label: Option<String>,
    pub kind: EntityKind,
}

#[derive(Debug, Clone)]
pub struct HaMetricsConfig {
    pub name: String,
    pub url: String,
    pub token: String,
    /// Length of each comparison period in hours.
    pub hours: u64,
    pub entities: Vec<EntityConfig>,
}

pub struct HaMetricsInput {
    config: HaMetricsConfig,
    client: client::HaMetricsClient,
}

impl HaMetricsInput {
    pub fn new(config: HaMetricsConfig) -> Self {
        let client = client::HaMetricsClient::new(&config.url, &config.token);
        Self { config, client }
    }
}

#[async_trait]
impl Input for HaMetricsInput {
    fn name(&self) -> &str {
        &self.config.name
    }

    async fn collect(&self) -> Result<InputData, BoxError> {
        let now = Utc::now();
        let period_a_start = now - chrono::Duration::hours(self.config.hours as i64);
        let period_b_start = now - chrono::Duration::hours(self.config.hours as i64 * 2);

        tracing::info!(
            input = self.config.name,
            "Fetching HA metrics history for {} entities over {}h windows",
            self.config.entities.len(),
            self.config.hours,
        );

        // Fetch one history window per entity covering both periods, then split.
        let mut history_a: HashMap<String, Vec<StateChange>> = HashMap::new();
        let mut history_b: HashMap<String, Vec<StateChange>> = HashMap::new();

        for entity in &self.config.entities {
            let full_history = self
                .client
                .get_history(period_b_start, now, &entity.id)
                .await?;

            let (b, a): (Vec<StateChange>, Vec<StateChange>) = full_history
                .into_iter()
                .partition(|s| s.last_changed < period_a_start);

            history_a.insert(entity.id.clone(), a);
            history_b.insert(entity.id.clone(), b);
        }

        let text = client::format_comparison(
            &self.config.entities,
            &history_a,
            &history_b,
            period_a_start,
            now,
            period_b_start,
            period_a_start,
        );

        if text.trim().is_empty() {
            return Err(Error::NoData.into());
        }

        Ok(InputData {
            source: self.config.name.clone(),
            content: text,
        })
    }
}
