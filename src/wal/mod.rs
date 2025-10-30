//! Write-Ahead Log (WAL) subsystem for aerodb
//!
//! The WAL is the authoritative durability mechanism in Phase 0.
//! No acknowledged write exists unless it is fully persisted in the WAL.
//!
//! # Design Principles
//!
//! - Durability over throughput
//! - Determinism over optimization
//! - Simplicity over cleverness
//! - Explicit failure over silent recovery
//!
//! # Invariants Enforced
//!
//! - D1: fsync before acknowledgment
//! - R1: WAL precedes all storage writes
//! - R2: Sequential deterministic replay
//! - R3: Explicit recovery success/failure
//! - K1: Checksums on every record
//! - K2: Halt-on-corruption policy
//! - C1: Full-document atomicity

mod checksum;
mod errors;
mod reader;
mod record;
mod writer;

pub use checksum::compute_checksum;
pub use errors::{WalError, WalResult};
pub use reader::WalReader;
pub use record::{RecordType, WalPayload, WalRecord};
pub use writer::WalWriter;
