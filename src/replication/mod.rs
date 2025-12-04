//! Replication Subsystem
//!
//! Per PHASE2_REPLICATION_INVARIANTS.md:
//! - Single-writer invariant: exactly one Primary
//! - Commit authority invariant: only Primary assigns CommitId
//! - Replicas consume history, never create it
//! - Replication halts on any invariant violation
//!
//! Per REPLICATION_MODEL.md:
//! - Authority is externally configured, never inferred
//! - No leader election, no consensus, no timeouts
//! - Fail-stop on ambiguity

mod role;
mod authority;
mod errors;

pub use role::{ReplicationRole, ReplicationState};
pub use authority::{AuthorityCheck, WriteAdmission};
pub use errors::{ReplicationError, ReplicationResult};
