# Phase 11: File Storage Architecture

**Phase:** 11 - File Storage  
**Status:** Active

---

## Module Structure

```
src/file_storage/
├── mod.rs           # Module entry
├── errors.rs        # Storage errors
├── bucket.rs        # Bucket management
├── file.rs          # File operations
├── permissions.rs   # RLS integration
├── backend.rs       # Backend trait
├── local.rs         # Local filesystem backend
└── signed_url.rs    # Signed URL generation
```

---

## Data Model

### Buckets (stored in AeroDB)
```json
{
  "id": "uuid",
  "name": "avatars",
  "public": false,
  "allowed_mime_types": ["image/*"],
  "max_file_size": 5242880,
  "created_at": "2026-01-01T00:00:00Z"
}
```

### Objects (metadata in AeroDB)
```json
{
  "id": "uuid",
  "bucket_id": "uuid",
  "path": "users/abc/avatar.png",
  "size": 12345,
  "content_type": "image/png",
  "owner_id": "uuid",
  "created_at": "2026-01-01T00:00:00Z"
}
```

---

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST | /storage/v1/object/{bucket}/{path} | Upload |
| GET | /storage/v1/object/{bucket}/{path} | Download |
| DELETE | /storage/v1/object/{bucket}/{path} | Delete |
| POST | /storage/v1/bucket | Create bucket |
| GET | /storage/v1/bucket | List buckets |
