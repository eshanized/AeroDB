//! # HTTP Server
//!
//! Main HTTP server combining all endpoint routers.
//!
//! This is the unified entry point for the AeroDB dashboard API.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

use super::auth_routes::{auth_routes, AuthState};
use super::config::HttpServerConfig;
use super::observability_routes::{health_routes, observability_routes};

/// HTTP Server for AeroDB Dashboard
pub struct HttpServer {
    config: HttpServerConfig,
    router: Router,
}

impl HttpServer {
    /// Create a new HTTP server with default configuration
    pub fn new() -> Self {
        Self::with_config(HttpServerConfig::default())
    }

    /// Create a new HTTP server with custom configuration
    pub fn with_config(config: HttpServerConfig) -> Self {
        let router = Self::build_router(&config);
        Self { config, router }
    }

    /// Build the combined router with all endpoints
    fn build_router(config: &HttpServerConfig) -> Router {
        // Create shared state
        let auth_state = Arc::new(AuthState::new());

        // Configure CORS
        let cors = CorsLayer::new()
            .allow_origin(Any) // For development - TODO: restrict in production
            .allow_methods(Any)
            .allow_headers(Any);

        // Combine all routes
        Router::new()
            // Health check at root level
            .merge(health_routes())
            // Auth routes under /auth
            .nest("/auth", auth_routes(auth_state))
            // Observability routes under /observability
            .nest("/observability", observability_routes())
            // Apply CORS middleware
            .layer(cors)
    }

    /// Get the socket address
    pub fn socket_addr(&self) -> String {
        self.config.socket_addr()
    }

    /// Get the router (for testing)
    pub fn router(self) -> Router {
        self.router
    }

    /// Start the HTTP server (async)
    pub async fn start(self) -> Result<(), std::io::Error> {
        let addr: SocketAddr = self
            .config
            .socket_addr()
            .parse()
            .expect("Invalid socket address");

        println!("Starting AeroDB HTTP server on {}", addr);
        println!("Dashboard API available at http://{}", addr);
        println!("Health check: http://{}/health", addr);

        let listener = TcpListener::bind(addr).await?;
        axum::serve(listener, self.router).await?;

        Ok(())
    }
}

impl Default for HttpServer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let server = HttpServer::new();
        assert_eq!(server.socket_addr(), "0.0.0.0:54321");
    }

    #[test]
    fn test_server_with_custom_port() {
        let config = HttpServerConfig::with_port(8080);
        let server = HttpServer::with_config(config);
        assert_eq!(server.socket_addr(), "0.0.0.0:8080");
    }

    #[test]
    fn test_router_builds() {
        let server = HttpServer::new();
        let _router = server.router();
        // If we get here, router construction succeeded
    }
}
