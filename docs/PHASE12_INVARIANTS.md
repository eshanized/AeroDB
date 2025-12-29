# Phase 12: Serverless Invariants

**Phase:** 12 - Serverless Functions  
**Status:** Active

---

## Core Invariants

### FN-I1: Sandboxed Execution
> Functions cannot access host filesystem or network directly

### FN-I2: Resource Limits
> Functions terminated when exceeding time/memory limits

### FN-I3: Idempotent Triggers
> Same trigger event produces same invocation (deterministic)

---

## Failure Invariants

### FN-F1: Timeout Cleanup
> Timed-out functions are killed and resources freed

### FN-F2: Error Isolation
> Function errors don't crash the server
