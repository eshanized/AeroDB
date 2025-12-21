//! Authority Transition Manager
//!
//! Per PHASE6_ARCHITECTURE.md §3.3:
//! - Applies an approved authority transition
//! - Ensures atomicity of authority change
//! - Integrates with existing replication state
//!
//! Constraints:
//! - Must not overlap authority
//! - Must be crash-safe
//! - Must leave recovery unambiguous
//!
//! Per PHASE6_INVARIANTS.md §P6-A2:
//! Promotion MUST be atomic with respect to authority.
//! All-or-nothing. No intermediate state where two nodes are writable.

use crate::replication::{ReplicationState, HaltReason};
use super::errors::{PromotionError, PromotionResult, PromotionErrorKind};
use uuid::Uuid;

/// Result of an authority transition.
/// 
/// Per PHASE6_STATE_MACHINE.md §8:
/// - AuthorityTransitioning → Atomic outcome enforced
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransitionResult {
    /// Transition completed successfully.
    /// Authority has been atomically transferred.
    Completed {
        new_primary_id: Uuid,
    },
    
    /// Transition failed and was rolled back.
    /// No authority change occurred.
    Failed {
        reason: TransitionFailureReason,
    },
}

impl TransitionResult {
    /// Check if transition completed successfully.
    pub fn is_completed(&self) -> bool {
        matches!(self, Self::Completed { .. })
    }
}

/// Reason for transition failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransitionFailureReason {
    /// Replica is not in correct state for transition.
    InvalidReplicaState,
    
    /// Authority rebinding failed.
    RebindFailed,
    
    /// Transition was interrupted (crash recovery).
    Interrupted,
}

/// Authority Transition Manager
/// 
/// Per PHASE6_ARCHITECTURE.md §3.3:
/// - Applies an approved authority transition
/// - Ensures atomicity of authority change
/// - Integrates with existing replication state
/// 
/// This component performs state transition ONLY after validation succeeds.
pub struct AuthorityTransitionManager {
    /// Whether a transition is currently in progress
    transition_in_progress: bool,
    
    /// ID of replica being promoted (if transition in progress)
    promoting_replica_id: Option<Uuid>,
    
    /// Marker for atomic commit (simulated)
    /// In real implementation, this would be a durable marker
    atomic_marker_set: bool,
}

impl Default for AuthorityTransitionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthorityTransitionManager {
    /// Create a new authority transition manager.
    pub fn new() -> Self {
        Self {
            transition_in_progress: false,
            promoting_replica_id: None,
            atomic_marker_set: false,
        }
    }
    
    /// Check if a transition is in progress.
    pub fn is_transition_in_progress(&self) -> bool {
        self.transition_in_progress
    }
    
    /// Begin authority transition.
    /// 
    /// Per PHASE6_INVARIANTS.md §P6-A2:
    /// Authority transfer is atomic. All-or-nothing.
    /// 
    /// Per PHASE6_FAILURE_MODEL.md §3.4:
    /// Crash during authority transition → Atomic outcome enforced.
    /// Recovery observes either old or new authority, never mixed.
    pub fn begin_transition(
        &mut self,
        replica_id: Uuid,
        current_state: &ReplicationState,
    ) -> PromotionResult<()> {
        // Validate replica state
        match current_state {
            ReplicationState::ReplicaActive { replica_id: rid } => {
                if *rid != replica_id {
                    return Err(PromotionError::new(
                        PromotionErrorKind::InvalidReplica,
                        "replica ID mismatch"
                    ));
                }
            }
            _ => {
                return Err(PromotionError::new(
                    PromotionErrorKind::AuthorityTransitionFailed,
                    "replica not in ReplicaActive state"
                ));
            }
        }
        
        if self.transition_in_progress {
            return Err(PromotionError::new(
                PromotionErrorKind::AuthorityTransitionFailed,
                "transition already in progress"
            ));
        }
        
        // Mark transition as in progress
        self.transition_in_progress = true;
        self.promoting_replica_id = Some(replica_id);
        
        Ok(())
    }
    
    /// Apply the authority transition atomically.
    /// 
    /// Per PHASE6_INVARIANTS.md §P6-A2:
    /// No intermediate state where two nodes are writable.
    /// 
    /// Per PHASE6_INVARIANTS.md §P6-F2:
    /// After promotion completes → new authority is authoritative.
    /// 
    /// Returns the new ReplicationState for the promoted replica.
    pub fn apply_transition(&mut self) -> PromotionResult<ReplicationState> {
        if !self.transition_in_progress {
            return Err(PromotionError::new(
                PromotionErrorKind::AuthorityTransitionFailed,
                "no transition in progress"
            ));
        }
        
        // Set atomic marker (in real implementation, this would be durable)
        // Per P6-A2: This is the point of no return
        self.atomic_marker_set = true;
        
        // Authority rebinding complete
        // The replica is now PrimaryActive
        let new_state = ReplicationState::PrimaryActive;
        
        // Clear transition state
        self.transition_in_progress = false;
        
        Ok(new_state)
    }
    
    /// Complete the transition and return the new primary ID.
    pub fn complete_transition(&mut self) -> PromotionResult<Uuid> {
        let replica_id = self.promoting_replica_id
            .take()
            .ok_or_else(|| PromotionError::new(
                PromotionErrorKind::AuthorityTransitionFailed,
                "no replica ID for transition"
            ))?;
        
        // Clear atomic marker
        self.atomic_marker_set = false;
        
        Ok(replica_id)
    }
    
    /// Abort a transition that hasn't been applied atomically yet.
    /// 
    /// Per PHASE6_FAILURE_MODEL.md §3.3:
    /// Failure before authority transition → Promotion is NOT applied.
    pub fn abort_transition(&mut self) -> PromotionResult<()> {
        if self.atomic_marker_set {
            // Cannot abort after atomic marker is set
            return Err(PromotionError::new(
                PromotionErrorKind::AuthorityTransitionFailed,
                "cannot abort after atomic marker is set"
            ));
        }
        
        self.transition_in_progress = false;
        self.promoting_replica_id = None;
        
        Ok(())
    }
    
    /// Recover authority state after crash.
    /// 
    /// Per PHASE6_FAILURE_MODEL.md §3.4:
    /// Recovery MUST observe either old or new authority state.
    /// Never a mixed or ambiguous state.
    /// 
    /// Per PHASE6_STATE_MACHINE.md §8:
    /// AuthorityTransitioning → Atomic outcome enforced.
    pub fn recover_after_crash(&self) -> (bool, Option<Uuid>) {
        // If atomic marker was set, the transition was committed
        if self.atomic_marker_set {
            (true, self.promoting_replica_id)
        } else {
            // Transition was not committed, authority unchanged
            (false, None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn test_uuid() -> Uuid {
        Uuid::new_v4()
    }
    
    #[test]
    fn test_manager_starts_idle() {
        let manager = AuthorityTransitionManager::new();
        assert!(!manager.is_transition_in_progress());
    }
    
    #[test]
    fn test_begin_transition_valid_replica() {
        let mut manager = AuthorityTransitionManager::new();
        let replica_id = test_uuid();
        let state = ReplicationState::ReplicaActive { replica_id };
        
        let result = manager.begin_transition(replica_id, &state);
        assert!(result.is_ok());
        assert!(manager.is_transition_in_progress());
    }
    
    #[test]
    fn test_begin_transition_wrong_replica_id() {
        let mut manager = AuthorityTransitionManager::new();
        let replica_id = test_uuid();
        let other_id = test_uuid();
        let state = ReplicationState::ReplicaActive { replica_id: other_id };
        
        let result = manager.begin_transition(replica_id, &state);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_begin_transition_invalid_state() {
        let mut manager = AuthorityTransitionManager::new();
        let replica_id = test_uuid();
        let state = ReplicationState::Disabled;
        
        let result = manager.begin_transition(replica_id, &state);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_apply_transition_returns_primary_active() {
        let mut manager = AuthorityTransitionManager::new();
        let replica_id = test_uuid();
        let state = ReplicationState::ReplicaActive { replica_id };
        
        manager.begin_transition(replica_id, &state).unwrap();
        
        let new_state = manager.apply_transition().unwrap();
        assert_eq!(new_state, ReplicationState::PrimaryActive);
    }
    
    #[test]
    fn test_complete_transition_returns_replica_id() {
        let mut manager = AuthorityTransitionManager::new();
        let replica_id = test_uuid();
        let state = ReplicationState::ReplicaActive { replica_id };
        
        manager.begin_transition(replica_id, &state).unwrap();
        manager.apply_transition().unwrap();
        
        let new_primary_id = manager.complete_transition().unwrap();
        assert_eq!(new_primary_id, replica_id);
    }
    
    #[test]
    fn test_abort_transition_before_apply() {
        let mut manager = AuthorityTransitionManager::new();
        let replica_id = test_uuid();
        let state = ReplicationState::ReplicaActive { replica_id };
        
        manager.begin_transition(replica_id, &state).unwrap();
        
        let result = manager.abort_transition();
        assert!(result.is_ok());
        assert!(!manager.is_transition_in_progress());
    }
    
    #[test]
    fn test_cannot_abort_after_apply() {
        let mut manager = AuthorityTransitionManager::new();
        let replica_id = test_uuid();
        let state = ReplicationState::ReplicaActive { replica_id };
        
        manager.begin_transition(replica_id, &state).unwrap();
        manager.apply_transition().unwrap();
        
        let result = manager.abort_transition();
        assert!(result.is_err());
    }
    
    #[test]
    fn test_recovery_before_atomic_marker() {
        let manager = AuthorityTransitionManager::new();
        
        let (was_atomic, id) = manager.recover_after_crash();
        assert!(!was_atomic);
        assert!(id.is_none());
    }
    
    #[test]
    fn test_full_transition_lifecycle() {
        let mut manager = AuthorityTransitionManager::new();
        let replica_id = test_uuid();
        let state = ReplicationState::ReplicaActive { replica_id };
        
        // Begin
        manager.begin_transition(replica_id, &state).unwrap();
        assert!(manager.is_transition_in_progress());
        
        // Apply (atomic)
        let new_state = manager.apply_transition().unwrap();
        assert_eq!(new_state, ReplicationState::PrimaryActive);
        
        // Complete
        let new_primary_id = manager.complete_transition().unwrap();
        assert_eq!(new_primary_id, replica_id);
        
        // Manager is now idle
        assert!(!manager.is_transition_in_progress());
    }
}
