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
//!
//! # Phase 5 Implementation
//!
//! Per PHASE5_INVARIANTS.md §P5-I16:
//! - Replication MUST be disableable at startup
//! - Disabling MUST NOT affect primary behavior
//!
//! # Phase 3 Optimizations
//!
//! - Fast Read: Pre-validated snapshot reuse on replicas (optional, disabled by default)

mod config;
mod role;
mod authority;
mod errors;
mod fast_read;
mod wal_sender;
mod wal_receiver;
mod snapshot_transfer;
mod replica_reads;
mod failure_matrix;
mod recovery;
mod compatibility;

pub use config::ReplicationConfig;
pub use role::{ReplicationRole, ReplicationState, HaltReason};
pub use authority::{AuthorityCheck, WriteAdmission, check_write_admission, check_commit_authority, check_dual_primary};
pub use errors::{ReplicationError, ReplicationResult, ReplicationErrorKind};
pub use fast_read::{FastReadConfig, FastReadManager, FastReadResult, FastReadStats, ReplicaReadPath, ReplicaSafetyState, SafetyCheck, SafetyValidator, SafetyViolation};
pub use wal_sender::{WalSender, WalPosition, WalRecordEnvelope};
pub use wal_receiver::{WalReceiver, ReceiveResult};
pub use snapshot_transfer::{SnapshotReceiver, SnapshotMetadata, SnapshotTransferState, SnapshotEligibility, SnapshotInstallResult, check_snapshot_eligibility};
pub use replica_reads::{ReplicaReadAdmission, ReadEligibility};
pub use failure_matrix::{ReplicationCrashPoint, FailureOutcome, FailureState};
pub use recovery::{PrimaryRecovery, ReplicaRecovery, RecoveryValidation, ReplicaResumeState};
pub use compatibility::{CompatibilityCheck, CompatibilityAssertion, Phase1Compatibility, MvccCompatibility};







