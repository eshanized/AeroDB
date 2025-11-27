Proceeding.
Below is **only** the next required document.

---

# PHASE2_ROADMAP.md

## AeroDB Phase 2 — MVCC Roadmap

### Status

* This roadmap governs **design sequencing only**
* No implementation begins until **all design documents are approved**
* Phase-1 behavior remains frozen throughout
* Performance work is explicitly deferred until correctness is fully specified

---

## 1. Phase-2 Entry Conditions (Already Met)

Phase 2 proceeds under the following confirmed conditions:

* Phase-1 is complete, tested, and frozen
* All Phase-1 invariants remain authoritative
* No backward-incompatible changes are permitted
* All new behavior must be spec-driven

---

## 2. Design-First Rule

Phase 2 follows a strict rule:

> **No code before specs.
> No optimization before invariants.
> No concurrency before determinism.**

Every document below must be approved before moving to the next stage.

---

## 3. MVCC Design Sequence

### Stage 1 — Conceptual Foundations (Design Only)

These documents define *what exists*, not *how it is implemented*.

1. **`MVCC.md`**

   * Conceptual MVCC model for AeroDB
   * Definition of versions, visibility, and timelines
   * Relationship to Phase-1 single-version model

2. **`MVCC_VISIBILITY.md`**

   * Formal definition of read views
   * Visibility rules for reads
   * Isolation semantics (explicitly named and bounded)

3. **`MVCC_WAL_INTERACTION.md`**

   * How MVCC state is represented in WAL
   * Commit ordering model
   * Recovery guarantees for version metadata

> Exit criteria:
> Every MVCC read and write outcome is explainable *without reference to implementation*.

---

### Stage 2 — State Lifecycle & Safety

These documents define how MVCC state evolves safely over time.

4. **`MVCC_SNAPSHOT_INTEGRATION.md`**

   * Interaction with snapshots and checkpoints
   * Valid MVCC cut definitions
   * Snapshot completeness requirements

5. **`MVCC_GC.md`**

   * Version lifecycle states
   * Eligibility for reclamation
   * Deterministic garbage collection rules

6. **`MVCC_FAILURE_MATRIX.md`**

   * Crash points
   * Power loss scenarios
   * WAL truncation interactions
   * Expected recovery outcomes

> Exit criteria:
> No MVCC state exists that cannot survive a crash and deterministic recovery.

---

### Stage 3 — Validation & Compatibility

These documents ensure MVCC does not destabilize AeroDB.

7. **`MVCC_COMPATIBILITY.md`**

   * Interaction with existing query planner/executor
   * Index implications
   * Schema versioning interactions

8. **`MVCC_TESTING_STRATEGY.md`**

   * Deterministic test categories
   * Crash-injection coverage
   * Visibility correctness tests

9. **`PHASE2_PROOFS.md`**

   * Argument-based proofs of:

     * Visibility correctness
     * Recovery determinism
     * GC safety

> Exit criteria:
> MVCC correctness can be argued independently of performance or hardware.

---

## 4. Implementation Gate

Implementation **may begin only when**:

* All MVCC design documents are approved
* Invariants are demonstrably preserved
* Crash behavior is fully specified
* WAL format changes (if any) are finalized

No partial implementation is permitted.

---

## 5. Performance Deferral Rule

Performance work is **explicitly forbidden** until:

* MVCC is fully implemented
* Correctness is proven by tests and crash harness
* Behavior is stable and spec-complete

Only then may the following be considered:

* Write batching
* Read path optimizations
* Index acceleration

All performance work must:

* Preserve invariants
* Be provably safe
* Be optional and spec-governed

---

## 6. Relationship to Replication (Future Phase-2 Work)

MVCC design must remain:

* Serializable
* Deterministic
* Replication-ready

However:

* No replication assumptions are allowed in MVCC behavior
* Replication will consume MVCC, not define it
