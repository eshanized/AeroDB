# Phase 11: File Storage Invariants

**Phase:** 11 - File Storage  
**Status:** Active

---

## Core Invariants

### FS-I1: Metadata Consistency
> File exists on disk ⟺ Metadata exists in database

### FS-I2: Atomic Operations
> Upload/delete are atomic (either both succeed or neither)

### FS-I3: RLS Enforcement
> All file access goes through RLS check

### FS-I4: Path Uniqueness
> No two objects in same bucket have same path

---

## Failure Invariants

### FS-F1: Upload Failure Cleanup
> Failed upload cleans up partial file

### FS-F2: Delete Failure Safe
> Delete failure leaves system in consistent state
