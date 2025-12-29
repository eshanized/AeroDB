# Phase 12: Serverless Functions Vision

**Phase:** 12 - Serverless Functions  
**Status:** Active

---

## Goal

Enable custom server-side logic via WebAssembly functions that can be triggered by HTTP requests, database events, or scheduled jobs.

---

## Philosophy

1. **Sandboxed Execution** - WASM provides safe isolation
2. **Database Triggers** - Functions can react to insert/update/delete
3. **HTTP Handlers** - Custom API endpoints
4. **Scheduled Jobs** - Cron-like execution
5. **Deterministic Core** - Function execution logged for replay

---

## Non-Goals

- Multi-language support (WASM only initially)
- Long-running processes (max execution time enforced)
- Direct disk access (functions use AeroDB APIs)

---

## Success Criteria

1. Functions deploy in < 5 seconds
2. Cold start < 100ms
3. Execution time limit: 30 seconds
4. Memory limit: 128MB per invocation
