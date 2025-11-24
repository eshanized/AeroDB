//! aerodb - A strict, deterministic, self-hostable database
//!
//! Phase 0: Minimum Viable Infrastructure

pub mod api;
pub mod backup;
pub mod checkpoint;
pub mod cli;
pub mod crash_point;
pub mod executor;
pub mod index;
pub mod observability;
pub mod planner;
pub mod recovery;
pub mod restore;
pub mod schema;
pub mod snapshot;
pub mod storage;
pub mod wal;
