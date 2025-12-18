# Phase 5 Stage 1: Replication Configuration & Role Declaration

## Status: IN PROGRESS (Core Complete)

This document tracks the implementation details of Phase 5 Stage 1.

---

## 1. Implementation Overview

**Goal:** Establish *static* replication identity per `PHASE5_IMPLEMENTATION_ORDER.md` §Stage 1.

**Key Achievements:**
- Created `ReplicationConfig` struct with strict validation.
- Enhanced `ReplicationState` with `Disabled` state (P5-I16 compliance).
- Added `replica_id` (UUID) to `ReplicaActive` state.
- Updated all authority checks to handle `Disabled` state safely.

---

## 2. Detailed Changes

### A. Configuration (`src/replication/config.rs`)
New `ReplicationConfig` struct responsible for static identity:
- `enabled: bool`: Defaults to `false` (Standalone Primary mode).
- `role: ReplicationRole`: `Primary` or `Replica`.
- `replica_id: Option<Uuid>`: Auto-generated for replicas, forbidden for primaries.
- `primary_address: Option<String>`: Required for replicas.

**Validation Rules:**
- Replicas MUST have `primary_address`.
- Primaries MUST NOT have `primary_address` or `replica_id`.

### B. Role State Machine (`src/replication/role.rs`)
Updated `ReplicationState` enum:
```rust
pub enum ReplicationState {
    /// Replication is disabled (default per P5-I16).
    /// Acts as standalone Primary.
    Disabled, 
    
    Uninitialized,
    
    /// Sole write authority
    PrimaryActive,
    
    /// Follower role
    ReplicaActive {
        /// Unique replica identifier
        replica_id: Uuid,
    },
    
    ReplicationHalted { reason: HaltReason },
}
```

**New Capabilities:**
- `Disabled` state behaves like a Primary for writes/reads but has no replication overhead.
- `state_name()` method added for future DX observability (returns "disabled", "primary_active", etc.).

### C. Authority Enforcement (`src/replication/authority.rs`)
Updated authority checks to respect `Disabled` state:
- `check_write_admission`: Admitted if `PrimaryActive` or `Disabled`.
- `check_commit_authority`: Allowed if `PrimaryActive` or `Disabled`.
- `check_dual_primary`: `Disabled` nodes do not claim replication authority (returns `NotAuthorized`).

### D. Replica Reads (`src/replication/replica_reads.rs`)
- Updated read admission logic to support `ReplicaActive` with UUID.
- Ensures read safety checks remain valid with new state variants.

---

## 3. Verification Results

**Unit Tests:**
All 120 replication module tests are passing.

```
running 120 tests
...
test result: ok. 120 passed; 0 failed
```

**Key Test Cases Covered:**
- Default config is disabled.
- Replica checks refuse writes.
- Invalid config combinations are rejected.
- State transitions enforce invariants (e.g., no implicit promotion).
- `Disabled` state allows normal primary operations (writes/reads).

---

## 4. Remaining Work (Stage 1)

1. **Startup Integration**:
   - Load `ReplicationConfig` from CLI/configuration.
   - Initialize `ReplicationState` based on config.

2. **DX Observability**:
   - Wire `/v1/replication` endpoint to `ReplicationState`.
   - Expose `role`, `state_name`, `replica_id`, and `blocking_reason`.
