# AeroDB MVCC Implementation Status

## Phase 2: MVCC Implementation

---

## MVCC-01: Domain Foundations ✓

**Status**: Complete

### Types Implemented

| Type | Location | Description |
|------|----------|-------------|
| `CommitId` | `mvcc/commit_id.rs` | Opaque, totally ordered commit identity |
| `Version` | `mvcc/version.rs` | Immutable document version |
| `VersionPayload` | `mvcc/version.rs` | Explicit Document/Tombstone enum |
| `VersionChain` | `mvcc/version_chain.rs` | Document history container |
| `ReadView` | `mvcc/read_view.rs` | Stable snapshot boundary |

---

## MVCC-02: Commit Identity WAL Authority ✓

**Status**: Complete

### WAL Additions

| Type | Description |
|------|-------------|
| `RecordType::MvccCommit` | WAL record type (value 3) |
| `MvccCommitPayload` | Commit identity payload |
| `MvccCommitRecord` | Complete commit record with checksum |

### Commit Authority

| Component | Location | Description |
|-----------|----------|-------------|
| `CommitAuthority` | `mvcc/commit_authority.rs` | WAL-based commit identity assignment |
| `CommitAuthorityError` | `mvcc/commit_authority.rs` | Non-monotonic/out-of-order errors |

### Crash Points

| Point | Purpose |
|-------|---------|
| `mvcc_before_commit_record` | Before commit identity persisted |
| `mvcc_after_commit_record` | After append, before fsync |
| `mvcc_after_commit_fsync` | After durable commit |

### Core Principle

> **The WAL is the sole source of truth for commit ordering.**

- No in-memory counters
- No atomic integers
- No clock usage
- If not durably recorded, it does not exist

---

## Test Results

```
test result: ok. 433 passed; 0 failed
```

---

## Next Steps

- MVCC-03: Version Persistence
- MVCC-04: Read View and Visibility Logic
