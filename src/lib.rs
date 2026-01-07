//! aerodb - A strict, deterministic, self-hostable database
//!
//! Phase 0: Minimum Viable Infrastructure
//! Phase 8: Authentication & Authorization (BaaS)
//! Phase 9: Auto-Generated REST API

pub mod api;
pub mod auth; // Phase 8: Authentication & Authorization
pub mod backup;
pub mod checkpoint;
pub mod cli;
pub mod crash_point;
pub mod dx;
pub mod executor;
pub mod file_storage; // Phase 11: File Storage
pub mod functions; // Phase 12: Serverless Functions
pub mod http_server;
pub mod index;
pub mod mvcc;
pub mod observability;
pub mod performance;
pub mod planner;
pub mod promotion;
pub mod realtime; // Phase 10: Real-Time Subscriptions
pub mod recovery;
pub mod replication;
pub mod rest_api; // Phase 9: REST API
pub mod restore;
pub mod schema;
pub mod snapshot;
pub mod storage;
pub mod wal; // Phase 13.5: Dashboard HTTP Server
