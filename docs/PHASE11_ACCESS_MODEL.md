# Phase 11: Access Model

**Phase:** 11 - File Storage  
**Status:** Active

---

## Access Control

### Bucket-Level
- Public buckets: anyone can read
- Private buckets: owner only

### Object-Level (RLS)
- Same RLS policies as database collections
- owner_id field checked against request context

---

## Signed URLs

Pre-authenticated URLs for temporary access:

```
/storage/v1/object/sign/{bucket}/{path}?token={jwt}&expires={timestamp}
```

- Default expiry: 1 hour
- Max expiry: 7 days
- Single-use or multi-use (configurable)

---

## Authentication

1. JWT in Authorization header
2. Token query parameter (for signed URLs)
3. Anonymous for public buckets
