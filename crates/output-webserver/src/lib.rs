pub mod server;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct WebServerConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

/// A single piece of content registered at an endpoint.
#[derive(Clone)]
pub struct ContentEntry {
    pub content_type: String,
    pub body: Vec<u8>,
}

/// Shared HTTP server. Outputs register content via [`WebServer::update`];
/// the server serves whatever is currently registered at each path.
pub struct WebServer {
    routes: Arc<RwLock<HashMap<String, ContentEntry>>>,
}

impl WebServer {
    pub fn new(config: WebServerConfig) -> Self {
        let routes: Arc<RwLock<HashMap<String, ContentEntry>>> =
            Arc::new(RwLock::new(HashMap::new()));

        let state = Arc::new(server::ServerState {
            routes: Arc::clone(&routes),
            username: config.username,
            password: config.password,
        });

        let host = config.host;
        let port = config.port;

        tokio::spawn(async move {
            if let Err(e) = server::serve(&host, port, state).await {
                tracing::error!("Web server failed: {}", e);
            }
        });

        Self { routes }
    }

    /// Register or replace content at `endpoint` (e.g. `"/"` or `"/briefing.wav"`).
    pub fn update(&self, endpoint: &str, entry: ContentEntry) {
        let path = if endpoint.starts_with('/') {
            endpoint.to_string()
        } else {
            format!("/{}", endpoint)
        };
        self.routes.write().unwrap().insert(path, entry);
    }
}
