# AeroDB MVCC Implementation Status

## Phase 2: MVCC Implementation

---

## MVCC-01: Domain Foundations ✓

| Type | Description |
|------|-------------|
| `CommitId` | Opaque, totally ordered commit identity |
| `Version` | Immutable document version |
| `VersionPayload` | Explicit Document/Tombstone enum |
| `VersionChain` | Document history container |
| `ReadView` | Stable snapshot boundary |

---

## MVCC-02: Commit Identity WAL Authority ✓

| Component | Description |
|-----------|-------------|
| `RecordType::MvccCommit` | WAL record type (value 3) |
| `MvccCommitPayload` | Commit identity payload |
| `MvccCommitRecord` | Complete commit record |
| `CommitAuthority` | WAL-based commit identity assignment |

---

## MVCC-03: Version Persistence ✓

| Component | Description |
|-----------|-------------|
| `RecordType::MvccVersion` | WAL record type (value 4) |
| `MvccVersionPayload` | Version with commit binding |
| `MvccVersionRecord` | Complete version record |
| `VersionValidator` | Cross-validation for recovery |

---

## MVCC-04: Read Views & Visibility ✓

| Component | Description |
|-----------|-------------|
| `Visibility` | Stateless visibility resolver |
| `VisibilityResult` | Visible/Invisible result enum |
| `visible_version()` | Apply exact MVCC_VISIBILITY.md rules |
| `current_snapshot()` | Create ReadView from CommitAuthority |

---

## MVCC-05: MVCC-Aware Snapshots ✓

| Component | Description |
|-----------|-------------|
| `SnapshotManifest.commit_boundary` | MVCC cut point in manifest |
| `with_mvcc_boundary()` | Create Phase-2 manifest |
| `create_mvcc_snapshot()` | Snapshot with MVCC boundary |
| `create_mvcc_checkpoint()` | Checkpoint with MVCC boundary |

### Manifest Format

| Version | MVCC | Description |
|---------|------|-------------|
| 1 | No | Phase-1 (no commit_boundary) |
| 2 | Yes | Phase-2 (with commit_boundary) |

---

## Test Results

```
test result: ok. 491 passed; 0 failed
```

---

## Next Steps

- MVCC-06: Write-Write Conflict Detection
