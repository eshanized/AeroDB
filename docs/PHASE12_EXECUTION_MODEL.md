# Phase 12: Execution Model

**Phase:** 12 - Serverless Functions  
**Status:** Active

---

## Execution Lifecycle

1. **Load** - Function WASM loaded from cache/storage
2. **Instantiate** - WASM module instantiated with limits
3. **Invoke** - Entry point called with context
4. **Collect** - Result/logs collected
5. **Cleanup** - Resources released

---

## Resource Limits

| Resource | Default | Max |
|----------|---------|-----|
| Timeout | 10s | 30s |
| Memory | 64MB | 128MB |
| CPU cycles | 1B | 10B |

---

## Host Functions

Functions can call AeroDB APIs:

- `db_query(sql)` - Execute query
- `db_insert(collection, doc)` - Insert document
- `http_fetch(url, options)` - HTTP request
- `log(level, message)` - Logging
