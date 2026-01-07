//! # AeroDB HTTP Server Module
//!
//! Phase 13.5: Dashboard HTTP Server Integration
//!
//! This module provides an HTTP-based API server for the AeroDB dashboard.
//! It combines all endpoint routers into a unified Axum server.
//!
//! # Endpoints
//!
//! - `/health` - Health check
//! - `/rest/v1/*` - REST API for database operations
//! - `/auth/*` - Authentication endpoints
//! - `/observability/*` - Metrics and monitoring

pub mod config;
pub mod server;
pub mod auth_routes;
pub mod observability_routes;

pub use config::HttpServerConfig;
pub use server::HttpServer;
