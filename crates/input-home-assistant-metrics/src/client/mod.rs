#[cfg(test)]
mod tests;

use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use thiserror::Error;
use tracing::debug;

use crate::{EntityConfig, EntityKind};

#[derive(Debug, Error)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Home Assistant API error: {0}")]
    ApiError(String),
}

#[derive(Debug, Clone, Deserialize)]
pub struct StateChange {
    pub state: String,
    pub last_changed: DateTime<Utc>,
}

pub struct HaMetricsClient {
    http: Client,
    base_url: String,
    token: String,
}

impl HaMetricsClient {
    pub fn new(base_url: &str, token: &str) -> Self {
        Self {
            http: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            token: token.to_string(),
        }
    }

    /// Fetch the state history for a single entity over [start, end).
    /// Uses the `/api/history/period/{start}` endpoint with minimal_response=true.
    pub async fn get_history(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        entity_id: &str,
    ) -> Result<Vec<StateChange>, Error> {
        // Use Z suffix to avoid percent-encoding issues with + in the path segment.
        let start_str = start.format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let url = format!("{}/api/history/period/{}", self.base_url, start_str);

        debug!(
            url = %url,
            entity = entity_id,
            start = %start.format("%Y-%m-%d %H:%M UTC"),
            end = %end.format("%Y-%m-%d %H:%M UTC"),
            "Fetching HA entity history"
        );

        let resp = self
            .http
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            // reqwest encodes query values automatically
            .query(&[
                ("end_time", end.to_rfc3339()),
                ("filter_entity_id", entity_id.to_string()),
                ("minimal_response", "true".to_string()),
            ])
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(Error::ApiError(format!("HTTP {status}: {body}")));
        }

        // Response: [[{state, last_changed}, ...]] — outer list is per entity.
        let data: Vec<Vec<StateChange>> = resp.json().await?;
        debug!(entity = entity_id, points = data.first().map(|v| v.len()).unwrap_or(0), "Received history");
        Ok(data.into_iter().next().unwrap_or_default())
    }
}

// ── Period summary ────────────────────────────────────────────────────────────

/// Return the latest numeric value from a slice of state history.
/// Used for gauge sensors (e.g. battery level) where the current reading matters.
pub fn compute_period_latest(changes: &[StateChange]) -> Option<f64> {
    changes.iter().rev().find_map(|c| c.state.parse::<f64>().ok())
}

/// Compute the total change from a slice of state history.
///
/// Handles two sensor patterns:
/// - **Resetting** (e.g. `_current_hour`): value accumulates then drops to near
///   zero at each reset boundary. The total is the sum of each segment's peak.
/// - **Non-resetting** (ever-increasing): total = last value − first value.
///
/// A reset is detected when a value drops to less than 50% of the previous peak.
pub fn compute_period_total(changes: &[StateChange]) -> Option<f64> {
    let values: Vec<f64> = changes
        .iter()
        .filter_map(|c| c.state.parse::<f64>().ok())
        .collect();

    if values.is_empty() {
        return None;
    }

    let has_resets = values.windows(2).any(|w| w[1] < w[0] * 0.5);

    if has_resets {
        // Resetting sensor: sum the peak of each monotonic segment.
        let mut total = 0.0f64;
        let mut segment_peak = values[0];
        for &v in &values[1..] {
            if v < segment_peak * 0.5 {
                total += segment_peak;
                segment_peak = v;
            } else {
                segment_peak = segment_peak.max(v);
            }
        }
        total += segment_peak;
        Some(total)
    } else {
        // Non-resetting sensor: last − first (floor at 0).
        Some((values.last().unwrap() - values.first().unwrap()).max(0.0))
    }
}

// ── Formatting ────────────────────────────────────────────────────────────────

pub fn format_comparison(
    entities: &[EntityConfig],
    history_a: &HashMap<String, Vec<StateChange>>,
    history_b: &HashMap<String, Vec<StateChange>>,
    period_a_start: DateTime<Utc>,
    period_a_end: DateTime<Utc>,
    period_b_start: DateTime<Utc>,
    period_b_end: DateTime<Utc>,
) -> String {
    let fmt = |dt: DateTime<Utc>| dt.format("%Y-%m-%d %H:%M UTC").to_string();

    let mut out = String::new();
    out.push_str("Home Assistant Metrics Report\n");
    out.push_str("=============================\n\n");
    out.push_str(&format!(
        "Current period:  {} → {}\n",
        fmt(period_a_start),
        fmt(period_a_end)
    ));
    out.push_str(&format!(
        "Previous period: {} → {}\n\n",
        fmt(period_b_start),
        fmt(period_b_end)
    ));

    for entity in entities {
        let label = entity.label.as_deref().unwrap_or(&entity.id);
        out.push_str(&format!("{} ({}):\n", label, entity.id));

        match entity.kind {
            EntityKind::Gauge => {
                // Use the latest reading from the most recent period, falling back to
                // the previous period if the sensor hasn't reported in the current window.
                let val = history_a
                    .get(&entity.id)
                    .and_then(|h| compute_period_latest(h))
                    .or_else(|| history_b.get(&entity.id).and_then(|h| compute_period_latest(h)));
                match val {
                    Some(v) => out.push_str(&format!("  Reading: {:.1}\n", v)),
                    None => out.push_str("  No data available.\n"),
                }
            }
            EntityKind::Cumulative => {
                let total_a = history_a.get(&entity.id).and_then(|h| compute_period_total(h));
                let total_b = history_b.get(&entity.id).and_then(|h| compute_period_total(h));
                match (total_a, total_b) {
                    (Some(a), Some(b)) => {
                        let delta = a - b;
                        let pct = if b != 0.0 { delta / b * 100.0 } else { 0.0 };
                        let sign = if delta >= 0.0 { "+" } else { "" };
                        out.push_str(&format!("  Current:  {:.0} Wh\n", a));
                        out.push_str(&format!("  Previous: {:.0} Wh\n", b));
                        out.push_str(&format!(
                            "  Change:   {}{:.0} Wh ({}{:.1}%)\n",
                            sign, delta, sign, pct
                        ));
                    }
                    (Some(a), None) => {
                        out.push_str(&format!("  Current:  {:.0} Wh\n", a));
                        out.push_str("  Previous: no data\n");
                    }
                    (None, Some(b)) => {
                        out.push_str("  Current:  no data\n");
                        out.push_str(&format!("  Previous: {:.0} Wh\n", b));
                    }
                    (None, None) => {
                        out.push_str("  No data available for either period.\n");
                    }
                }
            }
        }

        out.push('\n');
    }

    out
}
