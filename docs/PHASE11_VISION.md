# Phase 11: File Storage Vision

**Phase:** 11 - File Storage  
**Status:** Active

---

## Goal

Provide S3-compatible file storage with RLS-based access control, enabling applications to store and retrieve files with the same permission model as database records.

---

## Philosophy

1. **Metadata in Database** - File metadata stored in AeroDB for consistency
2. **RLS for Files** - Same permission model as collections
3. **Backend Abstraction** - Local filesystem or S3-compatible cloud
4. **Signed URLs** - Time-limited access for unauthenticated downloads
5. **Explicit Buckets** - No implicit storage locations

---

## Non-Goals

- Image processing/transcoding
- CDN integration (future)
- Streaming media optimization

---

## Success Criteria

1. Upload/download files up to 100MB
2. RLS policies enforced on file access
3. Signed URLs work for 1 hour
