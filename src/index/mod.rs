//! Index Manager subsystem for aerodb
//!
//! Indexes are derived, in-memory-only state rebuilt from Storage on startup.
//!
//! # Design Principles
//!
//! - Derived state: Indexes mirror storage, never the source of truth
//! - In-memory only: No persistence
//! - Deterministic: BTreeMap iteration order, sorted offsets
//!
//! # Invariants
//!
//! - Indexes rebuilt on startup from storage
//! - Updates occur AFTER storage writes
//! - Lookup returns sorted offsets ascending

mod btree;
mod errors;
mod manager;

pub use btree::{IndexKey, IndexTree};
pub use errors::{IndexError, IndexErrorCode, IndexResult};
pub use manager::IndexManager;
