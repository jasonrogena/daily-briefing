use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tracing::{debug, info};

#[cfg(test)]
mod tests;

#[derive(Debug, Error)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Fever API authentication failed")]
    AuthFailed,
    #[error("Invalid response from Fever API: {0}")]
    InvalidResponse(String),
}

#[derive(Debug, Deserialize)]
struct FeverResponse {
    auth: u8,
    #[serde(default)]
    feeds: Vec<Feed>,
    #[serde(default)]
    items: Vec<Item>,
    #[serde(default)]
    #[allow(dead_code)]
    total_items: Option<u64>,
    #[serde(default)]
    unread_item_ids: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Feed {
    pub id: u64,
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Item {
    pub id: u64,
    pub feed_id: u64,
    pub title: String,
    #[serde(default)]
    pub author: String,
    pub html: String,
    pub url: String,
    pub is_read: u8,
    pub created_on_time: u64,
}

pub struct FeverClient {
    url: String,
    api_key: String,
    http: Client,
}

impl FeverClient {
    pub fn new(url: &str, username: &str, password: &str) -> Self {
        let raw = format!("{}:{}", username, password);
        let digest = md5::compute(raw.as_bytes());
        let api_key = format!("{:x}", digest);

        Self {
            url: url.to_string(),
            api_key,
            http: Client::new(),
        }
    }

    /// POST to the Fever endpoint.
    /// `ops` are appended as bare query flags (e.g. `?api&feeds`).
    /// `extra_query` are additional key=value query params (e.g. `since_id=42`).
    /// Only `api_key` goes in the form body.
    async fn post(
        &self,
        ops: &[&str],
        extra_query: &[(&str, &str)],
    ) -> Result<FeverResponse, Error> {
        let mut query: Vec<(&str, &str)> = vec![("api", "")];
        for op in ops {
            query.push((op, ""));
        }
        query.extend_from_slice(extra_query);

        let http_resp = self
            .http
            .post(&self.url)
            .query(&query)
            .form(&[("api_key", self.api_key.as_str())])
            .send()
            .await?;

        let status = http_resp.status();
        let raw = http_resp.text().await?;

        let resp: FeverResponse = serde_json::from_str(&raw).map_err(|e| {
            Error::InvalidResponse(format!("HTTP {status}, parse error: {e}, body: {raw:?}"))
        })?;

        if resp.auth != 1 {
            return Err(Error::AuthFailed);
        }

        Ok(resp)
    }

    pub async fn get_feeds(&self) -> Result<Vec<Feed>, Error> {
        debug!("Fetching feeds list");
        let resp = self.post(&["feeds"], &[]).await?;
        info!(count = resp.feeds.len(), "Fetched feeds");
        Ok(resp.feeds)
    }

    pub async fn get_unread_ids(&self) -> Result<Vec<u64>, Error> {
        let resp = self.post(&["unread_item_ids"], &[]).await?;
        let ids = resp
            .unread_item_ids
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.is_empty())
            .filter_map(|s| s.trim().parse::<u64>().ok())
            .collect();
        Ok(ids)
    }

    pub async fn get_items_since(&self, since_id: u64) -> Result<Vec<Item>, Error> {
        debug!(since_id, "Fetching items");
        let since_str = since_id.to_string();
        let resp = self
            .post(&["items"], &[("since_id", &since_str)])
            .await?;
        debug!(count = resp.items.len(), "Received items batch");
        Ok(resp.items)
    }

    /// Fetch all items within the last `hours` hours, with full pagination.
    /// When `only_unread` is true, items that are already marked read are excluded.
    /// Returns the formatted text content and the IDs of all collected items.
    pub async fn fetch_recent_items(
        &self,
        hours: u64,
        only_unread: bool,
    ) -> Result<(String, Vec<u64>), Error> {
        let cutoff = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .saturating_sub(hours * 3600);

        let unread_ids: Option<std::collections::HashSet<u64>> = if only_unread {
            let ids = self.get_unread_ids().await?;
            debug!(count = ids.len(), "Fetched unread item IDs");
            Some(ids.into_iter().collect())
        } else {
            None
        };

        let feeds = self.get_feeds().await?;
        let feed_map: HashMap<u64, String> = feeds.into_iter().map(|f| (f.id, f.title)).collect();

        let mut all_items: Vec<Item> = Vec::new();
        let mut since_id: u64 = 0;

        loop {
            let batch = self.get_items_since(since_id).await?;
            if batch.is_empty() {
                break;
            }
            let max_id = batch.iter().map(|i| i.id).max().unwrap_or(0);
            let recent: Vec<Item> = batch
                .into_iter()
                .filter(|i| i.created_on_time >= cutoff)
                .filter(|i| {
                    unread_ids
                        .as_ref()
                        .map(|set| set.contains(&i.id))
                        .unwrap_or(true)
                })
                .collect();
            all_items.extend(recent);
            if max_id <= since_id {
                break;
            }
            since_id = max_id;
        }

        if all_items.is_empty() {
            info!(hours, "No articles found in the configured time window");
            return Ok((
                String::from("No new articles in the configured time window."),
                vec![],
            ));
        }

        info!(count = all_items.len(), hours, "Found articles in window");

        let ids: Vec<u64> = all_items.iter().map(|i| i.id).collect();

        let mut parts: Vec<String> = Vec::with_capacity(all_items.len());
        for item in &all_items {
            let feed_title = feed_map
                .get(&item.feed_id)
                .map(|s| s.as_str())
                .unwrap_or("Unknown Feed");
            let plain_text = strip_html(&item.html);
            parts.push(format!(
                "Feed: {}\nTitle: {}\nURL: {}\n\n{}",
                feed_title, item.title, item.url, plain_text
            ));
        }

        Ok((parts.join("\n\n---\n\n"), ids))
    }

    /// Mark the given item IDs as read in the Fever API.
    pub async fn mark_items_as_read(&self, ids: &[u64]) -> Result<(), Error> {
        for id in ids {
            let id_str = id.to_string();
            self.post(&[], &[("mark", "item"), ("as", "read"), ("id", &id_str)])
                .await?;
        }
        info!(count = ids.len(), "Marked items as read");
        Ok(())
    }
}

/// Strip HTML tags from a string, returning plain text.
fn strip_html(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    // Collapse multiple blank lines
    let lines: Vec<&str> = out.lines().collect();
    let mut result = String::new();
    let mut blank_count = 0u32;
    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            blank_count += 1;
            if blank_count <= 1 {
                result.push('\n');
            }
        } else {
            blank_count = 0;
            result.push_str(trimmed);
            result.push('\n');
        }
    }
    result.trim().to_string()
}
