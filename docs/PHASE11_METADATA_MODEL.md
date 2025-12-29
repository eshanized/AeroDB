# Phase 11: Metadata Model

**Phase:** 11 - File Storage  
**Status:** Active

---

## Storage Objects Collection

Objects stored in `_storage_objects` system collection:

```rust
pub struct StorageObject {
    pub id: Uuid,
    pub bucket_id: Uuid,
    pub path: String,
    pub size: u64,
    pub content_type: String,
    pub checksum: String,  // SHA-256
    pub owner_id: Option<Uuid>,
    pub metadata: Value,   // Custom metadata
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

---

## Invariants

- **FS-M1:** Metadata always in sync with file existence
- **FS-M2:** Path is unique within bucket
- **FS-M3:** Checksum verified on read (optional)
