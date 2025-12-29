# Phase 11: Bucket Model

**Phase:** 11 - File Storage  
**Status:** Active

---

## Bucket Configuration

```rust
pub struct BucketConfig {
    pub name: String,
    pub public: bool,
    pub allowed_mime_types: Vec<String>,
    pub max_file_size: usize,
    pub owner_id: Option<Uuid>,
}
```

---

## Bucket Policies

| Policy | Read | Write | Delete |
|--------|------|-------|--------|
| Public | Anyone | Owner | Owner |
| Private | Owner | Owner | Owner |
| Authenticated | Authed | Owner | Owner |

---

## MIME Type Filtering

- Wildcards: `image/*`, `video/*`
- Specific: `application/pdf`, `text/plain`
- Empty = allow all
