//! MVCC Domain Types
//!
//! Per MVCC.md and PHASE2_INVARIANTS.md:
//! - Defines MVCC vocabulary in code
//! - Encodes invariants structurally
//! - NO runtime behavior in this module
//!
//! This module provides:
//! - `CommitId` - Totally ordered commit identity
//! - `Version` - Immutable document version
//! - `VersionChain` - Version history for a document
//! - `ReadView` - Stable snapshot boundary

mod commit_id;
mod read_view;
mod version;
mod version_chain;

pub use commit_id::CommitId;
pub use read_view::ReadView;
pub use version::{Version, VersionPayload};
pub use version_chain::VersionChain;
