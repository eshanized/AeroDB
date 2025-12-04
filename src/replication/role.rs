//! Replication Role State Machine
//!
//! Per REPLICATION_MODEL.md §4:
//! - Uninitialized: No authority, no traffic
//! - PrimaryActive: Sole write authority
//! - ReplicaActive: Follows Primary
//! - ReplicationHalted: Invariant violation detected
//!
//! Per PHASE2_REPLICATION_INVARIANTS.md §2.1:
//! - At any moment, exactly one node may acknowledge writes
//! - If authority is unclear, writes must be rejected

use super::errors::{ReplicationError, ReplicationResult};

/// Replication role per REPLICATION_MODEL.md §2
///
/// A node is either a Primary (creates history) or a Replica (consumes history).
/// This is configured externally, never inferred.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplicationRole {
    /// Node is configured as the Primary
    /// May accept writes, assign CommitIds, emit WAL records
    Primary,
    
    /// Node is configured as a Replica
    /// Must reject writes, apply WAL from Primary only
    Replica,
}

/// Replication state per REPLICATION_MODEL.md §4
///
/// Each node exists in exactly one of these states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplicationState {
    /// No authoritative history, no WAL authority
    /// Node must not accept any traffic
    /// This state exists only before bootstrap
    Uninitialized,
    
    /// Node is the sole write authority
    /// Per PHASE2_REPLICATION_INVARIANTS.md §2.1:
    /// - May accept writes
    /// - May assign CommitIds
    /// - May emit WAL records
    /// - Must reject attempts to follow another Primary
    PrimaryActive,
    
    /// Node follows a specific Primary
    /// Per PHASE2_REPLICATION_INVARIANTS.md §2.1:
    /// - May apply WAL records
    /// - May serve reads (subject to REPLICATION_READ_SEMANTICS.md)
    /// - Must reject all writes
    /// - Must reject CommitId assignment
    ReplicaActive,
    
    /// Node has detected an invariant violation
    /// Per REPLICATION_MODEL.md §4.4:
    /// - WAL gap detected
    /// - Divergent history detected
    /// - Authority ambiguity detected
    ///
    /// In this state:
    /// - No reads allowed
    /// - No writes allowed
    /// - Explicit operator intervention required
    ReplicationHalted {
        /// Reason for halt
        reason: HaltReason,
    },
}

/// Reason for replication halt
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HaltReason {
    /// WAL gap detected during replication
    WalGapDetected,
    
    /// Divergent history detected
    HistoryDivergence,
    
    /// Authority ambiguity (e.g., dual primary suspected)
    AuthorityAmbiguity,
    
    /// WAL corruption during replication
    WalCorruption,
    
    /// Snapshot integrity failure
    SnapshotIntegrityFailure,
    
    /// Configuration error
    ConfigurationError,
}

impl ReplicationState {
    /// Create a new uninitialized state.
    pub fn new() -> Self {
        Self::Uninitialized
    }
    
    /// Transition to PrimaryActive.
    ///
    /// Per REPLICATION_MODEL.md §4.2:
    /// - Only valid from Uninitialized
    /// - External authority check must pass
    pub fn become_primary(self) -> ReplicationResult<Self> {
        match self {
            Self::Uninitialized => Ok(Self::PrimaryActive),
            Self::PrimaryActive => Ok(Self::PrimaryActive), // Idempotent
            Self::ReplicaActive => Err(ReplicationError::illegal_transition(
                "cannot transition from Replica to Primary without explicit reconfiguration"
            )),
            Self::ReplicationHalted { .. } => Err(ReplicationError::halted(
                "cannot transition from halted state without operator intervention"
            )),
        }
    }
    
    /// Transition to ReplicaActive.
    ///
    /// Per REPLICATION_MODEL.md §4.3:
    /// - Only valid from Uninitialized
    pub fn become_replica(self) -> ReplicationResult<Self> {
        match self {
            Self::Uninitialized => Ok(Self::ReplicaActive),
            Self::ReplicaActive => Ok(Self::ReplicaActive), // Idempotent
            Self::PrimaryActive => Err(ReplicationError::illegal_transition(
                "cannot transition from Primary to Replica without explicit reconfiguration"
            )),
            Self::ReplicationHalted { .. } => Err(ReplicationError::halted(
                "cannot transition from halted state without operator intervention"
            )),
        }
    }
    
    /// Transition to ReplicationHalted.
    ///
    /// Per REPLICATION_MODEL.md §4.4:
    /// - Valid from any state
    /// - Once halted, cannot resume without operator intervention
    pub fn halt(self, reason: HaltReason) -> Self {
        Self::ReplicationHalted { reason }
    }
    
    /// Check if this state allows writes.
    ///
    /// Per REPLICATION_MODEL.md §7:
    /// - Only PrimaryActive may accept writes
    pub fn can_write(&self) -> bool {
        matches!(self, Self::PrimaryActive)
    }
    
    /// Check if this state allows reads.
    ///
    /// Per REPLICATION_MODEL.md §8:
    /// - Primary and ReplicaActive may serve reads
    /// - ReplicationHalted must refuse reads
    pub fn can_read(&self) -> bool {
        matches!(self, Self::PrimaryActive | Self::ReplicaActive)
    }
    
    /// Check if this state is halted.
    pub fn is_halted(&self) -> bool {
        matches!(self, Self::ReplicationHalted { .. })
    }
    
    /// Check if this is Primary.
    pub fn is_primary(&self) -> bool {
        matches!(self, Self::PrimaryActive)
    }
    
    /// Check if this is Replica.
    pub fn is_replica(&self) -> bool {
        matches!(self, Self::ReplicaActive)
    }
    
    /// Get halt reason if halted.
    pub fn halt_reason(&self) -> Option<HaltReason> {
        match self {
            Self::ReplicationHalted { reason } => Some(*reason),
            _ => None,
        }
    }
}

impl Default for ReplicationState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uninitialized_can_become_primary() {
        let state = ReplicationState::new();
        let result = state.become_primary();
        assert!(result.is_ok());
        assert!(result.unwrap().is_primary());
    }

    #[test]
    fn test_uninitialized_can_become_replica() {
        let state = ReplicationState::new();
        let result = state.become_replica();
        assert!(result.is_ok());
        assert!(result.unwrap().is_replica());
    }

    #[test]
    fn test_replica_cannot_become_primary_directly() {
        // Per REPLICATION_MODEL.md §5: Implicit Promotion is forbidden
        let state = ReplicationState::ReplicaActive;
        let result = state.become_primary();
        assert!(result.is_err());
    }

    #[test]
    fn test_primary_cannot_become_replica_directly() {
        let state = ReplicationState::PrimaryActive;
        let result = state.become_replica();
        assert!(result.is_err());
    }

    #[test]
    fn test_halted_cannot_transition() {
        let state = ReplicationState::ReplicationHalted {
            reason: HaltReason::WalGapDetected,
        };
        
        assert!(state.become_primary().is_err());
        assert!(state.become_replica().is_err());
    }

    #[test]
    fn test_only_primary_can_write() {
        // Per PHASE2_REPLICATION_INVARIANTS.md §2.1
        assert!(ReplicationState::PrimaryActive.can_write());
        assert!(!ReplicationState::ReplicaActive.can_write());
        assert!(!ReplicationState::Uninitialized.can_write());
        assert!(!ReplicationState::ReplicationHalted {
            reason: HaltReason::WalGapDetected,
        }.can_write());
    }

    #[test]
    fn test_halted_cannot_read() {
        // Per REPLICATION_MODEL.md §4.4
        let halted = ReplicationState::ReplicationHalted {
            reason: HaltReason::AuthorityAmbiguity,
        };
        assert!(!halted.can_read());
    }

    #[test]
    fn test_primary_and_replica_can_read() {
        assert!(ReplicationState::PrimaryActive.can_read());
        assert!(ReplicationState::ReplicaActive.can_read());
    }

    #[test]
    fn test_any_state_can_halt() {
        let states = vec![
            ReplicationState::Uninitialized,
            ReplicationState::PrimaryActive,
            ReplicationState::ReplicaActive,
        ];
        
        for state in states {
            let halted = state.halt(HaltReason::WalCorruption);
            assert!(halted.is_halted());
            assert_eq!(halted.halt_reason(), Some(HaltReason::WalCorruption));
        }
    }
}
