# Phase 1 Final Readiness Checklist — AeroDB

This checklist defines the **non-negotiable completion criteria** for AeroDB Phase 1.

Phase 1 is considered complete **only if every item below is satisfied**.

If any item is unchecked, Phase 1 is incomplete.

---

## 1. Core Data Safety

### Write-Ahead Log (WAL)

- [ ] Every acknowledged write is fsynced before success
- [ ] WAL append order is strictly monotonic
- [ ] WAL corruption is detected via checksum
- [ ] WAL corruption halts recovery (FATAL)
- [ ] WAL replay is deterministic
- [ ] WAL truncation occurs **only** during checkpoint

---

### Storage Engine

- [ ] Storage is append-only
- [ ] Full-document writes only (no deltas)
- [ ] Every record is checksummed
- [ ] Corruption is detected on read
- [ ] Corruption is never silently repaired
- [ ] Tombstones correctly represent deletes
- [ ] Reads never observe partial records

---

## 2. Schema Enforcement

- [ ] Schemas are mandatory for all writes
- [ ] Schema ID and version must be explicit
- [ ] Schema versions are immutable
- [ ] No implicit fields or coercion
- [ ] Extra fields are rejected
- [ ] Missing required fields are rejected
- [ ] `_id` is required and immutable
- [ ] Recovery fails if referenced schema is missing

---

## 3. Query Safety & Determinism

### Planner

- [ ] All queries are proven bounded before execution
- [ ] Queries without limit are rejected
- [ ] Filters on non-indexed fields are rejected
- [ ] Sorts on non-indexed fields are rejected
- [ ] Planner output is deterministic
- [ ] Tie-breaking rules are deterministic
- [ ] Explain output is stable

---

### Executor

- [ ] Execution follows plan exactly
- [ ] No runtime heuristics
- [ ] Deterministic result ordering
- [ ] Stable sort only
- [ ] Limit enforced physically
- [ ] Schema version filtering enforced
- [ ] Checksum validated on every read

---

## 4. Index Integrity

- [ ] Indexes are derived state only
- [ ] Indexes are rebuilt on startup
- [ ] Index rebuild halts on corruption
- [ ] Index ordering is deterministic
- [ ] Tombstoned documents are excluded
- [ ] No index persistence to disk

---

## 5. Recovery Correctness

- [ ] Recovery always runs on startup
- [ ] WAL replay starts at correct position
- [ ] Replay is sequential and deterministic
- [ ] Index rebuild runs after replay
- [ ] Storage checksums verified post-replay
- [ ] Schema references verified
- [ ] Recovery success/failure is explicit
- [ ] No partial recovery allowed

---

## 6. Snapshot

- [ ] Snapshot is read-only
- [ ] Snapshot fsyncs WAL before copy
- [ ] Snapshot copies storage byte-for-byte
- [ ] Snapshot copies schemas
- [ ] Snapshot has a manifest
- [ ] Manifest includes checksums
- [ ] Partial snapshots are cleaned up
- [ ] Snapshot does NOT truncate WAL

---

## 7. Checkpoint

- [ ] Checkpoint acquires global execution lock
- [ ] Checkpoint creates snapshot first
- [ ] Checkpoint writes marker before truncation
- [ ] WAL truncation is atomic
- [ ] Crash before truncation preserves WAL
- [ ] Crash after truncation recovers via snapshot
- [ ] Checkpoint errors are non-fatal
- [ ] Checkpoint ID equals snapshot ID

---

## 8. Backup

- [ ] Backup uses latest snapshot
- [ ] Backup includes WAL tail
- [ ] Backup is deterministic
- [ ] Backup has a manifest
- [ ] No compression used
- [ ] Backup is read-only
- [ ] Partial backups are cleaned up
- [ ] Backup errors do not affect serving

---

## 9. Restore

- [ ] Restore is offline-only
- [ ] Restore validates backup structure
- [ ] Restore validates snapshot checksums
- [ ] Restore validates WAL integrity
- [ ] Restore uses atomic directory replacement
- [ ] Original data preserved on failure
- [ ] Restore never rebuilds indexes
- [ ] Restore errors are FATAL

---

## 10. Observability

- [ ] Structured JSON logs
- [ ] Deterministic key ordering
- [ ] Lifecycle events logged
- [ ] WAL events logged
- [ ] Snapshot / checkpoint events logged
- [ ] Backup / restore events logged
- [ ] Recovery events logged
- [ ] Query lifecycle logged
- [ ] Metrics counters monotonic
- [ ] Observability has zero behavioral impact

---

## 11. Crash Testing

- [ ] Crash injection via environment variable
- [ ] Crash points implemented across subsystems
- [ ] WAL crash scenarios tested
- [ ] Storage mid-write crashes tested
- [ ] Snapshot crashes tested
- [ ] Checkpoint crashes tested
- [ ] Backup crashes tested
- [ ] Restore crashes tested
- [ ] Recovery determinism verified
- [ ] No flaky crash tests

---

## 12. Configuration Safety

- [ ] Unsafe configurations rejected
- [ ] WAL fsync cannot be disabled
- [ ] Checksums cannot be disabled
- [ ] Schemas cannot be bypassed
- [ ] Configuration validated at startup
- [ ] Configuration is immutable post-start

---

## 13. CLI & Boot

- [ ] `init` creates correct layout
- [ ] `start` runs full recovery before serving
- [ ] `query` and `explain` require recovery
- [ ] Clean shutdown marker handled correctly
- [ ] Startup failure halts immediately
- [ ] No background threads started implicitly

---

## 14. Tests & Discipline

- [ ] All unit tests passing
- [ ] All integration tests passing
- [ ] All crash tests passing
- [ ] No ignored tests
- [ ] No flaky tests
- [ ] Code matches governing docs
- [ ] No undocumented behavior

---

## Phase 1 Exit Criteria

Phase 1 is complete **only if**:

- All checklist items are checked
- Crash testing passes consistently
- No invariants are violated
- System behavior matches documentation exactly

---

## What Phase 1 Guarantees

- No acknowledged write is lost
- Corruption is always detected
- Recovery is deterministic
- Backups are restorable
- Restore is atomic
- Queries are bounded and safe
- Behavior is explicit and explainable

---

END PHASE1_READINESS.md
