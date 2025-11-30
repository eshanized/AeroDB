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
//! - `VersionStorage` - Commit-bound version persistence
//! - `Visibility` - Deterministic snapshot isolation

mod commit_authority;
mod commit_id;
mod read_view;
mod version;
mod version_chain;
mod version_storage;
mod visibility;

pub use commit_authority::{CommitAuthority, CommitAuthorityError};
pub use commit_id::CommitId;
pub use read_view::ReadView;
pub use version::{Version, VersionPayload};
pub use version_chain::VersionChain;
pub use version_storage::{
    PersistedVersion, VersionExpectations, VersionStorageError, VersionStorageResult, VersionValidator,
};
pub use visibility::{Visibility, VisibilityResult};
