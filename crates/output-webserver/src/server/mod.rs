#[cfg(test)]
mod tests;

use axum::{
    extract::{Request, State},
    http::{header, HeaderMap, StatusCode},
    response::Response,
    Router,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::info;

use crate::ContentEntry;

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

pub struct ServerState {
    pub routes: Arc<RwLock<HashMap<String, ContentEntry>>>,
    pub username: String,
    pub password: String,
}

pub(crate) fn check_basic_auth(headers: &HeaderMap, username: &str, password: &str) -> bool {
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

async fn handle(State(state): State<Arc<ServerState>>, req: Request) -> Response {
    if !check_basic_auth(req.headers(), &state.username, &state.password) {
        return Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .header(header::WWW_AUTHENTICATE, "Basic realm=\"daily-briefing\"")
            .body(axum::body::Body::empty())
            .unwrap();
    }

    let path = req.uri().path().to_string();
    let entry = state.routes.read().unwrap().get(&path).cloned();

    match entry {
        Some(e) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, e.content_type)
            .body(axum::body::Body::from(e.body))
            .unwrap(),
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(axum::body::Body::from("Not found"))
            .unwrap(),
    }
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

    let app = Router::new().fallback(handle).with_state(state);

    axum::serve(listener, app).await.map_err(Error::Serve)?;

    Ok(())
}
