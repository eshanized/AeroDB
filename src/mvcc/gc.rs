//! MVCC Garbage Collection
//!
//! Per MVCC_GC.md:
//! - A version may be removed only if it is provably invisible to all possible read views
//! - GC is correctness-driven, not performance-driven
//! - All GC events must be WAL-recorded
//!
//! Per MVCC_GC.md §4, a version V(commit_id = C) is reclaimable iff:
//! 1. C < visibility_lower_bound
//! 2. A newer version exists in the same version chain
//! 3. No snapshot requires V
//! 4. Recovery correctness is preserved without V
//!
//! This module provides:
//! - `VersionLifecycleState` - Lifecycle states per MVCC_GC.md §2
//! - `VisibilityFloor` - Tracks visibility lower bound
//! - `GcEligibility` - Applies all 4 eligibility rules

use super::{CommitId, ReadView, Version, VersionChain};

/// Version lifecycle states per MVCC_GC.md §2
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionLifecycleState {
    /// Version is visible to at least one possible read view
    /// Live versions are untouchable by GC
    Live,

    /// Version has been superseded by a newer committed version
    /// But may still be visible to some read views
    /// Obsolete does NOT mean reclaimable
    Obsolete,

    /// Version is not visible to ANY possible read view
    /// And not required by any snapshot or recovery
    /// Reclaimable is a proof-based state, not a heuristic
    Reclaimable,

    /// Version removal has been fully recorded
    /// Recovery will not resurrect it
    Collected,
}

/// Visibility floor per MVCC_GC.md §3
///
/// The visibility lower bound is defined by:
/// - The oldest active read view
/// - The oldest retained snapshot boundary
///
/// No version with commit_id >= visibility_lower_bound may be reclaimed.
#[derive(Debug, Clone)]
pub struct VisibilityFloor {
    /// Oldest active read view boundary
    oldest_read_view: Option<CommitId>,
    /// Oldest retained snapshot boundary
    oldest_snapshot_boundary: Option<CommitId>,
}

impl VisibilityFloor {
    /// Create a new visibility floor with no active views or snapshots.
    pub fn new() -> Self {
        Self {
            oldest_read_view: None,
            oldest_snapshot_boundary: None,
        }
    }

    /// Register an active read view.
    pub fn register_read_view(&mut self, view: ReadView) {
        let boundary = view.upper_bound();
        match self.oldest_read_view {
            None => self.oldest_read_view = Some(boundary),
            Some(current) if boundary < current => {
                self.oldest_read_view = Some(boundary);
            }
            _ => {}
        }
    }

    /// Unregister a read view (when query completes).
    /// For simplicity, this resets to None - in production, would track set of views.
    pub fn unregister_read_view(&mut self, _view: ReadView) {
        // In a full implementation, we'd track all active views
        // For now, this is a placeholder
        self.oldest_read_view = None;
    }

    /// Register a snapshot boundary.
    pub fn register_snapshot(&mut self, boundary: CommitId) {
        match self.oldest_snapshot_boundary {
            None => self.oldest_snapshot_boundary = Some(boundary),
            Some(current) if boundary < current => {
                self.oldest_snapshot_boundary = Some(boundary);
            }
            _ => {}
        }
    }

    /// Unregister a snapshot (when deleted).
    pub fn unregister_snapshot(&mut self, _boundary: CommitId) {
        // In a full implementation, we'd track all snapshots
        self.oldest_snapshot_boundary = None;
    }

    /// Compute the visibility lower bound.
    ///
    /// Per MVCC_GC.md §3.1:
    /// The visibility lower bound is the minimum of:
    /// - The oldest active read view
    /// - The oldest retained snapshot boundary
    pub fn visibility_lower_bound(&self) -> Option<CommitId> {
        match (self.oldest_read_view, self.oldest_snapshot_boundary) {
            (Some(rv), Some(sb)) => Some(if rv < sb { rv } else { sb }),
            (Some(rv), None) => Some(rv),
            (None, Some(sb)) => Some(sb),
            (None, None) => None,
        }
    }

    /// Check if a version is below the visibility floor.
    ///
    /// Per MVCC_GC.md §3:
    /// commit_id < visibility_lower_bound
    pub fn is_below_floor(&self, commit_id: CommitId) -> bool {
        match self.visibility_lower_bound() {
            Some(floor) => commit_id < floor,
            None => false, // No floor means nothing is reclaimable yet
        }
    }
}

impl Default for VisibilityFloor {
    fn default() -> Self {
        Self::new()
    }
}

/// GC eligibility checker per MVCC_GC.md §4
///
/// Applies all 4 mandatory eligibility rules:
/// 1. C < visibility_lower_bound
/// 2. A newer version exists in the same chain
/// 3. No snapshot requires V
/// 4. Recovery correctness is preserved
pub struct GcEligibility;

impl GcEligibility {
    /// Check if a version is reclaimable.
    ///
    /// Per MVCC_GC.md §4, ALL four conditions are mandatory:
    /// 1. commit_id < visibility_lower_bound
    /// 2. A newer version exists in the chain
    /// 3. No snapshot requires this version (checked via floor)
    /// 4. Recovery correctness preserved (versions after checkpoint are safe)
    ///
    /// # Arguments
    ///
    /// * `version` - The version to check
    /// * `chain` - The version chain containing this version
    /// * `floor` - The visibility floor
    /// * `checkpoint_boundary` - Oldest commit required for recovery
    pub fn is_reclaimable(
        version: &Version,
        chain: &VersionChain,
        floor: &VisibilityFloor,
        checkpoint_boundary: Option<CommitId>,
    ) -> bool {
        let commit_id = version.commit_id();

        // Rule 1: C < visibility_lower_bound
        if !floor.is_below_floor(commit_id) {
            return false;
        }

        // Rule 2: A newer version exists in the chain
        if !Self::has_newer_version(version, chain) {
            return false;
        }

        // Rule 3: No snapshot requires V
        // This is implicitly checked by Rule 1 since snapshot boundaries
        // are included in visibility_lower_bound

        // Rule 4: Recovery correctness preserved
        if let Some(boundary) = checkpoint_boundary {
            if commit_id >= boundary {
                // Version is needed for recovery
                return false;
            }
        }

        true
    }

    /// Check if a newer version exists in the chain.
    fn has_newer_version(version: &Version, chain: &VersionChain) -> bool {
        let commit_id = version.commit_id();
        chain.versions().iter().any(|v| v.commit_id() > commit_id)
    }

    /// Compute the lifecycle state for a version.
    pub fn lifecycle_state(
        version: &Version,
        chain: &VersionChain,
        floor: &VisibilityFloor,
        checkpoint_boundary: Option<CommitId>,
    ) -> VersionLifecycleState {
        // Check if version is the newest in chain
        let is_newest = !Self::has_newer_version(version, chain);

        if is_newest {
            // Newest version is always Live
            return VersionLifecycleState::Live;
        }

        // Has newer version - check if still visible
        if !floor.is_below_floor(version.commit_id()) {
            // Could still be visible to some read views
            return VersionLifecycleState::Obsolete;
        }

        // Check all 4 rules
        if Self::is_reclaimable(version, chain, floor, checkpoint_boundary) {
            VersionLifecycleState::Reclaimable
        } else {
            VersionLifecycleState::Obsolete
        }
    }
}

/// GC record payload for WAL
///
/// Per MVCC_GC.md §5.1:
/// - Version removal must be WAL-recorded
/// - GC decisions are replayable
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GcRecordPayload {
    /// Collection containing the version
    pub collection_id: String,
    /// Document key
    pub document_id: String,
    /// Commit identity of the collected version
    pub collected_commit_id: u64,
}

impl GcRecordPayload {
    /// Create a new GC record payload.
    pub fn new(
        collection_id: impl Into<String>,
        document_id: impl Into<String>,
        collected_commit_id: u64,
    ) -> Self {
        Self {
            collection_id: collection_id.into(),
            document_id: document_id.into(),
            collected_commit_id,
        }
    }

    /// Serialize to bytes for WAL storage.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        // Collection ID (length-prefixed)
        let collection_bytes = self.collection_id.as_bytes();
        buf.extend_from_slice(&(collection_bytes.len() as u32).to_le_bytes());
        buf.extend_from_slice(collection_bytes);

        // Document ID (length-prefixed)
        let doc_bytes = self.document_id.as_bytes();
        buf.extend_from_slice(&(doc_bytes.len() as u32).to_le_bytes());
        buf.extend_from_slice(doc_bytes);

        // Collected commit ID
        buf.extend_from_slice(&self.collected_commit_id.to_le_bytes());

        buf
    }

    /// Deserialize from bytes.
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 16 {
            return None;
        }

        let mut offset = 0;

        // Collection ID
        let collection_len = u32::from_le_bytes(data[offset..offset + 4].try_into().ok()?) as usize;
        offset += 4;
        if data.len() < offset + collection_len {
            return None;
        }
        let collection_id = String::from_utf8(data[offset..offset + collection_len].to_vec()).ok()?;
        offset += collection_len;

        // Document ID
        if data.len() < offset + 4 {
            return None;
        }
        let doc_len = u32::from_le_bytes(data[offset..offset + 4].try_into().ok()?) as usize;
        offset += 4;
        if data.len() < offset + doc_len {
            return None;
        }
        let document_id = String::from_utf8(data[offset..offset + doc_len].to_vec()).ok()?;
        offset += doc_len;

        // Collected commit ID
        if data.len() < offset + 8 {
            return None;
        }
        let collected_commit_id = u64::from_le_bytes(data[offset..offset + 8].try_into().ok()?);

        Some(Self {
            collection_id,
            document_id,
            collected_commit_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::VersionPayload;

    fn make_version(key: &str, commit: u64) -> Version {
        let json = serde_json::json!({"id": key});
        let bytes = serde_json::to_vec(&json).unwrap();
        let payload = VersionPayload::Document(bytes);
        Version::new(key.to_string(), payload, CommitId::new(commit))
    }

    // === VisibilityFloor Tests ===

    #[test]
    fn test_empty_floor_has_no_bound() {
        let floor = VisibilityFloor::new();
        assert!(floor.visibility_lower_bound().is_none());
    }

    #[test]
    fn test_floor_with_read_view() {
        let mut floor = VisibilityFloor::new();
        floor.register_read_view(ReadView::new(CommitId::new(10)));

        assert_eq!(floor.visibility_lower_bound(), Some(CommitId::new(10)));
    }

    #[test]
    fn test_floor_with_snapshot() {
        let mut floor = VisibilityFloor::new();
        floor.register_snapshot(CommitId::new(5));

        assert_eq!(floor.visibility_lower_bound(), Some(CommitId::new(5)));
    }

    #[test]
    fn test_floor_uses_minimum() {
        let mut floor = VisibilityFloor::new();
        floor.register_read_view(ReadView::new(CommitId::new(10)));
        floor.register_snapshot(CommitId::new(5));

        // Should use the smaller boundary
        assert_eq!(floor.visibility_lower_bound(), Some(CommitId::new(5)));
    }

    #[test]
    fn test_is_below_floor() {
        let mut floor = VisibilityFloor::new();
        floor.register_snapshot(CommitId::new(10));

        assert!(floor.is_below_floor(CommitId::new(5)));  // Below
        assert!(floor.is_below_floor(CommitId::new(9)));  // Below
        assert!(!floor.is_below_floor(CommitId::new(10))); // At floor
        assert!(!floor.is_below_floor(CommitId::new(15))); // Above
    }

    // === GcEligibility Tests ===

    #[test]
    fn test_newest_version_not_reclaimable() {
        let v1 = make_version("doc1", 5);
        let mut chain = VersionChain::new("doc1".to_string());
        chain.push(v1.clone());

        let mut floor = VisibilityFloor::new();
        floor.register_snapshot(CommitId::new(10));

        // Newest version is never reclaimable
        assert!(!GcEligibility::is_reclaimable(&v1, &chain, &floor, None));
    }

    #[test]
    fn test_old_version_with_newer_is_reclaimable() {
        let v1 = make_version("doc1", 5);
        let v2 = make_version("doc1", 15);
        let mut chain = VersionChain::new("doc1".to_string());
        chain.push(v1.clone());
        chain.push(v2);

        let mut floor = VisibilityFloor::new();
        floor.register_snapshot(CommitId::new(10));

        // v1 is below floor and has newer version
        assert!(GcEligibility::is_reclaimable(&v1, &chain, &floor, None));
    }

    #[test]
    fn test_version_above_floor_not_reclaimable() {
        let v1 = make_version("doc1", 8);
        let v2 = make_version("doc1", 15);
        let mut chain = VersionChain::new("doc1".to_string());
        chain.push(v1.clone());
        chain.push(v2);

        let mut floor = VisibilityFloor::new();
        floor.register_snapshot(CommitId::new(5));

        // v1 is above floor (8 >= 5)
        assert!(!GcEligibility::is_reclaimable(&v1, &chain, &floor, None));
    }

    #[test]
    fn test_version_needed_for_recovery_not_reclaimable() {
        let v1 = make_version("doc1", 5);
        let v2 = make_version("doc1", 15);
        let mut chain = VersionChain::new("doc1".to_string());
        chain.push(v1.clone());
        chain.push(v2);

        let mut floor = VisibilityFloor::new();
        floor.register_snapshot(CommitId::new(10));

        // v1 is needed for recovery (checkpoint at 3)
        assert!(!GcEligibility::is_reclaimable(
            &v1,
            &chain,
            &floor,
            Some(CommitId::new(3))
        ));
    }

    #[test]
    fn test_lifecycle_state_live() {
        let v1 = make_version("doc1", 5);
        let mut chain = VersionChain::new("doc1".to_string());
        chain.push(v1.clone());

        let floor = VisibilityFloor::new();

        assert_eq!(
            GcEligibility::lifecycle_state(&v1, &chain, &floor, None),
            VersionLifecycleState::Live
        );
    }

    #[test]
    fn test_lifecycle_state_obsolete() {
        let v1 = make_version("doc1", 8);
        let v2 = make_version("doc1", 15);
        let mut chain = VersionChain::new("doc1".to_string());
        chain.push(v1.clone());
        chain.push(v2);

        let mut floor = VisibilityFloor::new();
        floor.register_snapshot(CommitId::new(5));

        assert_eq!(
            GcEligibility::lifecycle_state(&v1, &chain, &floor, None),
            VersionLifecycleState::Obsolete
        );
    }

    #[test]
    fn test_lifecycle_state_reclaimable() {
        let v1 = make_version("doc1", 3);
        let v2 = make_version("doc1", 15);
        let mut chain = VersionChain::new("doc1".to_string());
        chain.push(v1.clone());
        chain.push(v2);

        let mut floor = VisibilityFloor::new();
        floor.register_snapshot(CommitId::new(10));

        assert_eq!(
            GcEligibility::lifecycle_state(&v1, &chain, &floor, None),
            VersionLifecycleState::Reclaimable
        );
    }

    // === GcRecordPayload Tests ===

    #[test]
    fn test_gc_payload_roundtrip() {
        let payload = GcRecordPayload::new("users", "doc123", 42);
        let bytes = payload.to_bytes();
        let parsed = GcRecordPayload::from_bytes(&bytes).unwrap();

        assert_eq!(payload, parsed);
    }

    #[test]
    fn test_gc_payload_fields() {
        let payload = GcRecordPayload::new("orders", "order-456", 999);

        assert_eq!(payload.collection_id, "orders");
        assert_eq!(payload.document_id, "order-456");
        assert_eq!(payload.collected_commit_id, 999);
    }
}
