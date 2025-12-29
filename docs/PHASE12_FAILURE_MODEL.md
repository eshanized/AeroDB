# Phase 12: Failure Model

**Phase:** 12 - Serverless Functions  
**Status:** Active

---

## Error Types

| Error | HTTP | Recovery |
|-------|------|----------|
| FunctionNotFound | 404 | Return error |
| CompilationError | 400 | Return error |
| TimeoutError | 504 | Kill function |
| MemoryExceeded | 500 | Kill function |
| RuntimeError | 500 | Log and return |

---

## Retry Policy

- HTTP triggers: No automatic retry
- Database triggers: Retry 3 times with backoff
- Scheduled: Retry on next schedule
