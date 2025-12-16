//! Observability API
//!
//! Per DX_OBSERVABILITY_API.md:
//! - Read-only endpoints for state inspection
//! - Snapshot-explicit responses
//! - Deterministic output
//!
//! Read-only, Phase 4, no semantic authority.

pub mod handlers;
pub mod response;
pub mod server;

pub use response::{ApiResponse, ObservedAt, SnapshotType};
pub use server::ObservabilityServer;
