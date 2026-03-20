use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::{DateTime, Utc};
use pulldown_cmark::{html::push_html, Options, Parser};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

#[cfg(test)]
mod tests;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to bind to {addr}: {source}")]
    Bind {
        addr: String,
        source: std::io::Error,
    },
    #[error("Server error: {0}")]
    Serve(std::io::Error),
}

pub struct PageContent {
    pub markdown: String,
    pub updated_at: Option<DateTime<Utc>>,
}

impl PageContent {
    pub fn empty() -> Self {
        Self {
            markdown: String::new(),
            updated_at: None,
        }
    }

    pub fn new(markdown: &str) -> Self {
        Self {
            markdown: markdown.to_string(),
            updated_at: Some(Utc::now()),
        }
    }
}

pub struct ServerState {
    pub content: Arc<RwLock<PageContent>>,
    pub username: String,
    pub password: String,
    pub title: String,
}

fn check_basic_auth(headers: &HeaderMap, username: &str, password: &str) -> bool {
    let expected = format!("{}:{}", username, password);
    headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Basic "))
        .and_then(|encoded| STANDARD.decode(encoded).ok())
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .map(|creds| creds == expected)
        .unwrap_or(false)
}

fn markdown_to_html(markdown: &str) -> String {
    let parser = Parser::new_ext(markdown, Options::all());
    let mut html = String::new();
    push_html(&mut html, parser);
    html
}

fn render_page(title: &str, content: &PageContent) -> String {
    let updated = match content.updated_at {
        Some(t) => t.format("%Y-%m-%d %H:%M UTC").to_string(),
        None => "never".to_string(),
    };

    let body = if content.markdown.is_empty() {
        "<p class=\"empty\">No content yet — waiting for the first pipeline run.</p>".to_string()
    } else {
        markdown_to_html(&content.markdown)
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

async fn index(State(state): State<Arc<ServerState>>, headers: HeaderMap) -> Response {
    if !check_basic_auth(&headers, &state.username, &state.password) {
        return (
            StatusCode::UNAUTHORIZED,
            [(
                header::WWW_AUTHENTICATE,
                r#"Basic realm="Daily Summary""#,
            )],
            "Unauthorized",
        )
            .into_response();
    }

    let content = state.content.read().await;
    Html(render_page(&state.title, &content)).into_response()
}

pub async fn serve(host: &str, port: u16, state: Arc<ServerState>) -> Result<(), Error> {
    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|source| Error::Bind {
            addr: addr.clone(),
            source,
        })?;

    info!("Web server listening on http://{}", addr);

    let app = Router::new().route("/", get(index)).with_state(state);

    axum::serve(listener, app)
        .await
        .map_err(Error::Serve)?;

    Ok(())
}
