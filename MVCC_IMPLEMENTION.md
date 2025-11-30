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

### Visibility Rule (MVCC_VISIBILITY.md §3)

```
1. Filter: V.commit_id ≤ R.read_upper_bound
2. Select: Version with LARGEST commit_id
3. If tombstone → invisible
```

---

## Test Results

```
test result: ok. 461 passed; 0 failed
```

---

## Next Steps

- MVCC-05: Write-Write Conflict Detection
