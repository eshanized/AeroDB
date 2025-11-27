# AeroDB MVCC Implementation Status

## MVCC-01: Domain Foundations (Completed)

**Status**: Implemented & Tested  
**Focus**: Pure domain types, structural invariants, no runtime behavior.

### Module Structure
`src/mvcc/`
- `mod.rs`: Module exports
- `commit_id.rs`: Totally ordered commit identity
- `version.rs`: Immutable document version
- `version_chain.rs`: Version history container
- `read_view.rs`: Stable snapshot boundary

### Type Definitions

#### `CommitId`
- **Definition**: `pub struct CommitId(u64)` (Newtype)
- **Invariants**: Opaque, strictly totally ordered, no arithmetic.
- **Role**: Sole authority for visibility ordering.

#### `Version`
- **Definition**: Immutable struct.
- **Fields**:
  - `key: String`
  - `payload: VersionPayload` (Document or Tombstone)
  - `commit_id: CommitId`
- **Invariants**: Once created, never mutated. Tombstones are explicit.

#### `VersionChain`
- **Definition**: `pub struct VersionChain { key: String, versions: Vec<Version> }`
- **Role**: Pure data container for a document's history. No traversal logic implies no visibility decisions yet.

#### `ReadView`
- **Definition**: `pub struct ReadView { read_upper_bound: CommitId }`
- **Role**: Defines a stable snapshot cut. Immutable after construction.

### Test Coverage
- **22 unit tests** passing.
- Verified immutability, explicit construction, and lack of side effects.
