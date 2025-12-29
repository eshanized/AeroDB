# Phase 12: Function Model

**Phase:** 12 - Serverless Functions  
**Status:** Active

---

## Function Definition

```rust
pub struct Function {
    pub id: Uuid,
    pub name: String,
    pub trigger: TriggerType,
    pub wasm_hash: String,
    pub config: FunctionConfig,
    pub created_at: DateTime<Utc>,
}

pub struct FunctionConfig {
    pub timeout_ms: u64,
    pub memory_mb: u32,
    pub env: HashMap<String, String>,
}
```

---

## Trigger Types

```rust
pub enum TriggerType {
    Http { path: String, method: HttpMethod },
    Database { collection: String, event: EventType },
    Schedule { cron: String },
    Webhook { secret: String },
}
```

---

## Invocation Context

```rust
pub struct InvocationContext {
    pub function_id: Uuid,
    pub trigger: TriggerType,
    pub payload: Value,
    pub user_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
}
```
