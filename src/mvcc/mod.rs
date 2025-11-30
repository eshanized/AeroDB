//! MVCC Domain Types
//!
//! Per MVCC.md and PHASE2_INVARIANTS.md:
//! - Defines MVCC vocabulary in code
//! - Encodes invariants structurally
//!
//! This module provides:
//! - `CommitId` - Totally ordered commit identity
//! - `Version` - Immutable document version
//! - `VersionChain` - Version history for a document
//! - `ReadView` - Stable snapshot boundary
//! - `CommitAuthority` - WAL-based commit identity assignment

mod commit_authority;
mod commit_id;
mod read_view;
mod version;
mod version_chain;

pub use commit_authority::{CommitAuthority, CommitAuthorityError};
pub use commit_id::CommitId;
pub use read_view::ReadView;
pub use version::{Version, VersionPayload};
pub use version_chain::VersionChain;
