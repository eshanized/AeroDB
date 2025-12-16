//! API Handlers
//!
//! Per DX_OBSERVABILITY_API.md §2 (Core Principles):
//! - OAPI-1: All endpoints are strictly read-only
//! - OAPI-2: Every response states which snapshot it observes
//! - OAPI-3: Responses MUST be deterministic
//!
//! Read-only, Phase 4, no semantic authority.

use super::response::{ApiError, ApiResponse, ObservedAt};
use serde::{Deserialize, Serialize};

// ============================================================================
// Status Endpoint Data (§5.1)
// ============================================================================

/// Database lifecycle state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleState {
    /// System is starting up.
    Booting,
    /// System is running normally.
    Running,
    /// System is recovering from crash.
    Recovering,
    /// System is shutting down.
    ShuttingDown,
}

/// Status response data per §5.1.
///
/// Returns:
/// - Lifecycle state (booting, running, recovering)
/// - Current CommitId high-water mark
/// - WAL durability boundary
/// - Phase enablement flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusData {
    /// Current lifecycle state.
    pub lifecycle_state: LifecycleState,
    /// Current CommitId high-water mark.
    pub commit_id_high_water: u64,
    /// WAL durability boundary (last fsync'd offset).
    pub wal_durability_boundary: u64,
    /// Phase 3 performance optimizations enabled.
    pub phase3_enabled: bool,
    /// Phase 4 observability enabled.
    pub phase4_enabled: bool,
}

// ============================================================================
// WAL Endpoint Data (§5.2)
// ============================================================================

/// WAL state response per §5.2.
///
/// Returns:
/// - WAL file identifiers
/// - Current append offset
/// - Last durable offset
/// - Checksum status
/// - Truncation point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalData {
    /// Active WAL file identifier.
    pub active_file_id: u64,
    /// Current append offset (may not be durable).
    pub append_offset: u64,
    /// Last durable (fsync'd) offset.
    pub durable_offset: u64,
    /// Whether all checksums are valid.
    pub checksum_valid: bool,
    /// Truncation point (oldest retained offset).
    pub truncation_point: Option<u64>,
    /// Number of WAL segments.
    pub segment_count: usize,
}

// ============================================================================
// MVCC Endpoint Data (§5.3)
// ============================================================================

/// MVCC state response per §5.3.
///
/// Returns:
/// - Oldest retained CommitId
/// - Latest committed CommitId
/// - Active snapshots
/// - GC watermark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MvccData {
    /// Oldest retained CommitId (versions older than this may be GC'd).
    pub oldest_retained_commit_id: u64,
    /// Latest committed CommitId.
    pub latest_commit_id: u64,
    /// Number of active snapshots.
    pub active_snapshot_count: usize,
    /// Active snapshot IDs.
    pub active_snapshot_ids: Vec<u64>,
    /// GC watermark CommitId.
    pub gc_watermark: Option<u64>,
}

// ============================================================================
// Snapshots Endpoint Data (§5.4)
// ============================================================================

/// Individual snapshot info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotInfo {
    /// Snapshot identifier.
    pub snapshot_id: u64,
    /// CommitId this snapshot observes.
    pub commit_id: u64,
    /// Snapshot source.
    pub source: SnapshotSource,
}

/// Snapshot creation source.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotSource {
    /// User-requested snapshot.
    User,
    /// Internal system snapshot.
    Internal,
    /// Recovery-created snapshot.
    Recovery,
}

/// Snapshots response per §5.4.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotsData {
    /// List of active snapshots.
    pub snapshots: Vec<SnapshotInfo>,
}

// ============================================================================
// Checkpoints Endpoint Data (§5.5)
// ============================================================================

/// Checkpoint state response per §5.5.
///
/// Returns:
/// - Last completed checkpoint CommitId
/// - Checkpoint durability status
/// - WAL range covered
/// - Pending checkpoint info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointsData {
    /// Last completed checkpoint CommitId.
    pub last_checkpoint_commit_id: Option<u64>,
    /// Whether last checkpoint is fully durable.
    pub last_checkpoint_durable: bool,
    /// WAL start offset covered by checkpoint.
    pub wal_range_start: Option<u64>,
    /// WAL end offset covered by checkpoint.
    pub wal_range_end: Option<u64>,
    /// Whether a checkpoint is currently pending.
    pub checkpoint_pending: bool,
}

// ============================================================================
// Indexes Endpoint Data (§5.6)
// ============================================================================

/// Index info per §5.6.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexInfo {
    /// Index identifier.
    pub index_id: String,
    /// Index type.
    pub index_type: String,
    /// Build status.
    pub build_status: IndexBuildStatus,
    /// Entry count.
    pub entry_count: u64,
}

/// Index build status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IndexBuildStatus {
    /// Index is fully built.
    Complete,
    /// Index is currently building.
    Building,
    /// Index is being rebuilt.
    Rebuilding,
    /// Index build failed.
    Failed,
}

/// Indexes response per §5.6.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexesData {
    /// List of indexes.
    pub indexes: Vec<IndexInfo>,
}

// ============================================================================
// Replication Endpoint Data (§5.7)
// ============================================================================

/// Replication role.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReplicationRole {
    /// Primary (writer) node.
    Primary,
    /// Replica (read-only) node.
    Replica,
    /// Standalone (no replication).
    Standalone,
}

/// Replication state response per §5.7.
///
/// Conditional: only returned if replication is enabled.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationData {
    /// Node role.
    pub role: ReplicationRole,
    /// WAL prefix position (replica's applied CommitId).
    pub wal_prefix_commit_id: Option<u64>,
    /// Replica lag in CommitIds.
    pub replica_lag: Option<u64>,
    /// Whether snapshot bootstrap is in progress.
    pub snapshot_bootstrap_active: bool,
    /// Whether replication is enabled.
    pub replication_enabled: bool,
}

// ============================================================================
// Handler Placeholder Functions
// ============================================================================

/// Get current status.
///
/// Read-only, Phase 4, no semantic authority.
/// Per DX_OBSERVABILITY_API.md §5.1: Must NOT trigger refresh.
#[allow(dead_code)]
pub fn handle_status(
    lifecycle: LifecycleState,
    commit_id: u64,
    wal_durable: u64,
) -> ApiResponse<StatusData> {
    let data = StatusData {
        lifecycle_state: lifecycle,
        commit_id_high_water: commit_id,
        wal_durability_boundary: wal_durable,
        phase3_enabled: true, // Would be read from actual config
        phase4_enabled: true,
    };
    ApiResponse::new(ObservedAt::live(commit_id), data)
}

/// Get WAL state.
///
/// Read-only, Phase 4, no semantic authority.
#[allow(dead_code)]
pub fn handle_wal(commit_id: u64, wal_data: WalData) -> ApiResponse<WalData> {
    ApiResponse::new(ObservedAt::live(commit_id), wal_data)
}

/// Get MVCC state.
///
/// Read-only, Phase 4, no semantic authority.
/// Per DX_OBSERVABILITY_API.md §5.3: Must NOT trigger GC or snapshot creation.
#[allow(dead_code)]
pub fn handle_mvcc(commit_id: u64, mvcc_data: MvccData) -> ApiResponse<MvccData> {
    ApiResponse::new(ObservedAt::live(commit_id), mvcc_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_response_deterministic() {
        let resp1 = handle_status(LifecycleState::Running, 100, 95);
        let resp2 = handle_status(LifecycleState::Running, 100, 95);
        
        // Per OAPI-3: Given identical state, responses MUST be identical
        assert_eq!(resp1.data.lifecycle_state, resp2.data.lifecycle_state);
        assert_eq!(resp1.data.commit_id_high_water, resp2.data.commit_id_high_water);
    }

    #[test]
    fn test_response_includes_commit_id() {
        let resp = handle_status(LifecycleState::Running, 42, 40);
        
        // Per OAPI-2: Every response MUST state which snapshot it observes
        assert_eq!(resp.observed_at.commit_id, 42);
    }

    #[test]
    fn test_lifecycle_states() {
        assert_eq!(
            serde_json::to_string(&LifecycleState::Running).unwrap(),
            "\"running\""
        );
        assert_eq!(
            serde_json::to_string(&LifecycleState::Recovering).unwrap(),
            "\"recovering\""
        );
    }
}
