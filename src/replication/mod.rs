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
mod wal_sender;
mod wal_receiver;
mod snapshot_transfer;
mod replica_reads;

pub use role::{ReplicationRole, ReplicationState, HaltReason};
pub use authority::{AuthorityCheck, WriteAdmission, check_write_admission, check_commit_authority, check_dual_primary};
pub use errors::{ReplicationError, ReplicationResult, ReplicationErrorKind};
pub use wal_sender::{WalSender, WalPosition, WalRecordEnvelope};
pub use wal_receiver::{WalReceiver, ReceiveResult};
pub use snapshot_transfer::{SnapshotReceiver, SnapshotMetadata, SnapshotTransferState, SnapshotEligibility, SnapshotInstallResult, check_snapshot_eligibility};
pub use replica_reads::{ReplicaReadAdmission, ReadEligibility};



