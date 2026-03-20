#[cfg(test)]
mod tests;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use daily_briefing_core::input::BoxError;
use daily_briefing_core::output::Output;
use webserver::{ContentEntry, WebServer};
use pulldown_cmark::{html::push_html, Options, Parser};
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Webpage output error: {0}")]
    Internal(String),
}

pub struct WebpageConfig {
    pub name: String,
    /// Endpoint path this output is served at, e.g. `"/"`.
    pub endpoint: String,
    pub title: String,
}

pub struct WebpageOutput {
    name: String,
    endpoint: String,
    title: String,
    server: Arc<WebServer>,
}

impl WebpageOutput {
    pub fn new(config: WebpageConfig, server: Arc<WebServer>) -> Self {
        Self {
            name: config.name,
            endpoint: config.endpoint,
            title: config.title,
            server,
        }
    }
}

#[async_trait]
impl Output for WebpageOutput {
    fn name(&self) -> &str {
        &self.name
    }

    async fn write(&self, content: &str) -> Result<(), BoxError> {
        let html = render_page(&self.title, content, Utc::now());
        self.server.update(
            &self.endpoint,
            ContentEntry {
                content_type: "text/html; charset=utf-8".to_string(),
                body: html.into_bytes(),
            },
        );
        Ok(())
    }
}

// ── Rendering ────────────────────────────────────────────────────────────────

pub fn markdown_to_html(markdown: &str) -> String {
    let parser = Parser::new_ext(markdown, Options::all());
    let mut html = String::new();
    push_html(&mut html, parser);
    html
}

pub fn render_page(title: &str, markdown: &str, updated_at: DateTime<Utc>) -> String {
    let updated = updated_at.format("%Y-%m-%d %H:%M UTC").to_string();

    let body = if markdown.is_empty() {
        "<p class=\"empty\">No content yet — waiting for the first pipeline run.</p>".to_string()
    } else {
        markdown_to_html(markdown)
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{title}</title>
  <style>
    *, *::before, *::after {{ box-sizing: border-box; }}
    body {{
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      max-width: 860px;
      margin: 0 auto;
      padding: 2rem 1rem;
      background: #f5f5f5;
      color: #222;
      line-height: 1.7;
    }}
    article {{
      background: #fff;
      border-radius: 8px;
      padding: 2rem 2.5rem;
      box-shadow: 0 1px 4px rgba(0,0,0,.08);
    }}
    h1 {{ margin-top: 0; font-size: 1.6rem; }}
    h2 {{ font-size: 1.2rem; margin-top: 1.5rem; }}
    h3 {{ font-size: 1rem; }}
    .meta {{ color: #888; font-size: 0.85rem; margin-bottom: 1.5rem; border-bottom: 1px solid #eee; padding-bottom: 1rem; }}
    .empty {{ color: #aaa; font-style: italic; }}
    pre {{ background: #f8f8f8; padding: 1rem; border-radius: 4px; overflow-x: auto; font-size: 0.875rem; }}
    code {{ font-family: 'SF Mono', Consolas, 'Liberation Mono', monospace; font-size: 0.9em; }}
    a {{ color: #0066cc; }}
    hr {{ border: none; border-top: 1px solid #eee; margin: 1.5rem 0; }}
    ul, ol {{ padding-left: 1.5rem; }}
    blockquote {{ border-left: 3px solid #ddd; margin: 0; padding-left: 1rem; color: #555; }}
  </style>
</head>
<body>
  <article>
    <h1>{title}</h1>
    <p class="meta">Last updated: {updated}</p>
    {body}
  </article>
</body>
</html>"#,
        title = title,
        updated = updated,
        body = body,
    )
}
