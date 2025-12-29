# Phase 11: Failure Model

**Phase:** 11 - File Storage  
**Status:** Active

---

## Error Types

| Error | HTTP | Action |
|-------|------|--------|
| BucketNotFound | 404 | Return error |
| ObjectNotFound | 404 | Return error |
| Unauthorized | 403 | Return error |
| FileTooLarge | 413 | Reject upload |
| InvalidMimeType | 415 | Reject upload |
| StorageFull | 507 | Reject upload |

---

## Recovery

- Orphaned files cleaned up by background job
- Metadata without file = error on access
