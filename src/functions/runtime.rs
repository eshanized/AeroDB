//! # WASM Runtime
//!
//! WebAssembly runtime abstraction for serverless function execution.
//! Supports both stubbed (testing) and real WASM runtime backends.

use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use uuid::Uuid;

use super::errors::{FunctionError, FunctionResult};
use super::function::Function;
use crate::auth::rls::RlsContext;

/// Runtime configuration
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Maximum execution time in milliseconds
    pub timeout_ms: u64,

    /// Maximum memory in bytes
    pub max_memory_bytes: usize,

    /// Enable debug logging
    pub debug: bool,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 30_000,                  // 30 seconds
            max_memory_bytes: 128 * 1024 * 1024, // 128 MB
            debug: false,
        }
    }
}

/// Execution context for a function
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Invocation ID
    pub invocation_id: Uuid,

    /// Function being executed
    pub function_id: Uuid,

    /// User making the request
    pub user_id: Option<Uuid>,

    /// RLS context for database calls
    pub rls_context: RlsContext,

    /// Environment variables accessible to the function
    pub env: std::collections::HashMap<String, String>,
}

impl ExecutionContext {
    pub fn new(function: &Function, user_id: Option<Uuid>) -> Self {
        let rls_context = match user_id {
            Some(uid) => RlsContext::authenticated(uid),
            None => RlsContext::anonymous(),
        };

        Self {
            invocation_id: Uuid::new_v4(),
            function_id: function.id,
            user_id,
            rls_context,
            env: std::collections::HashMap::new(),
        }
    }

    pub fn with_env(mut self, key: &str, value: &str) -> Self {
        self.env.insert(key.to_string(), value.to_string());
        self
    }
}

/// Execution result with detailed metrics
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Invocation ID
    pub invocation_id: Uuid,

    /// Whether execution succeeded
    pub success: bool,

    /// Return value (if successful)
    pub result: Option<Value>,

    /// Error message (if failed)
    pub error: Option<String>,

    /// Execution time in milliseconds
    pub duration_ms: u64,

    /// Memory used in bytes
    pub memory_used: usize,

    /// Logs captured during execution
    pub logs: Vec<String>,
}

impl ExecutionResult {
    pub fn success(invocation_id: Uuid, result: Value, duration_ms: u64) -> Self {
        Self {
            invocation_id,
            success: true,
            result: Some(result),
            error: None,
            duration_ms,
            memory_used: 0,
            logs: Vec::new(),
        }
    }

    pub fn failure(invocation_id: Uuid, error: String, duration_ms: u64) -> Self {
        Self {
            invocation_id,
            success: false,
            result: None,
            error: Some(error),
            duration_ms,
            memory_used: 0,
            logs: Vec::new(),
        }
    }

    pub fn with_logs(mut self, logs: Vec<String>) -> Self {
        self.logs = logs;
        self
    }
}

/// Trait for WASM runtime implementations
pub trait WasmRuntime: Send + Sync {
    /// Execute a compiled WASM module with the given input
    fn execute(
        &self,
        function: &Function,
        input: Value,
        context: ExecutionContext,
        config: &RuntimeConfig,
    ) -> FunctionResult<ExecutionResult>;

    /// Check if the runtime is available
    fn is_available(&self) -> bool;

    /// Get runtime name for logging
    fn name(&self) -> &'static str;
}

/// Stubbed WASM runtime for testing (no actual WASM execution)
#[derive(Debug, Default)]
pub struct StubRuntime {
    /// Simulate failures for testing
    simulate_failure: bool,
}

impl StubRuntime {
    pub fn new() -> Self {
        Self {
            simulate_failure: false,
        }
    }

    pub fn with_failure(mut self) -> Self {
        self.simulate_failure = true;
        self
    }
}

impl WasmRuntime for StubRuntime {
    fn execute(
        &self,
        function: &Function,
        input: Value,
        context: ExecutionContext,
        config: &RuntimeConfig,
    ) -> FunctionResult<ExecutionResult> {
        let start = Instant::now();

        // Check enabled
        if !function.enabled {
            return Err(FunctionError::RuntimeError("Function is disabled".into()));
        }

        // Simulate timeout check
        if start.elapsed().as_millis() as u64 > config.timeout_ms {
            return Err(FunctionError::Timeout(config.timeout_ms));
        }

        // Simulate failure if requested
        if self.simulate_failure {
            let duration_ms = start.elapsed().as_millis() as u64;
            return Ok(ExecutionResult::failure(
                context.invocation_id,
                "Simulated failure for testing".into(),
                duration_ms,
            ));
        }

        // Stub: echo the input
        let result = json!({
            "success": true,
            "input": input,
            "function": function.name,
            "invocation_id": context.invocation_id.to_string(),
        });

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(
            ExecutionResult::success(context.invocation_id, result, duration_ms)
                .with_logs(vec!["[stub] Executed successfully".to_string()]),
        )
    }

    fn is_available(&self) -> bool {
        true // Stub is always available
    }

    fn name(&self) -> &'static str {
        "stub"
    }
}

/// Host functions that WASM modules can call
pub mod host {
    use super::*;

    /// Log a message (captured in execution logs)
    pub fn log(message: &str, logs: &mut Vec<String>) {
        logs.push(format!("[log] {}", message));
    }

    /// Get an environment variable
    pub fn env_get(key: &str, context: &ExecutionContext) -> Option<String> {
        context.env.get(key).cloned()
    }

    /// Query the database (placeholder - would integrate with executor)
    pub fn db_query(
        _collection: &str,
        _filter: Value,
        _context: &ExecutionContext,
    ) -> FunctionResult<Vec<Value>> {
        // In real implementation, this would:
        // 1. Parse the filter into a QueryParams
        // 2. Call the executor with RLS context
        // 3. Return the results

        // For now, return empty result
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::super::trigger::TriggerType;
    use super::*;

    fn create_test_function() -> Function {
        Function::new(
            "test-func".to_string(),
            TriggerType::http("/test".to_string()),
            vec![0, 97, 115, 109], // Minimal WASM header
        )
    }

    #[test]
    fn test_stub_runtime_execute() {
        let runtime = StubRuntime::new();
        let function = create_test_function();
        let context = ExecutionContext::new(&function, None);
        let config = RuntimeConfig::default();

        let input = json!({"message": "hello"});
        let result = runtime.execute(&function, input, context, &config).unwrap();

        assert!(result.success);
        assert!(result.result.is_some());
        assert!(!result.logs.is_empty());
    }

    #[test]
    fn test_stub_runtime_disabled_function() {
        let runtime = StubRuntime::new();
        let mut function = create_test_function();
        function.enabled = false;

        let context = ExecutionContext::new(&function, None);
        let config = RuntimeConfig::default();

        let result = runtime.execute(&function, json!({}), context, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_stub_runtime_failure() {
        let runtime = StubRuntime::new().with_failure();
        let function = create_test_function();
        let context = ExecutionContext::new(&function, None);
        let config = RuntimeConfig::default();

        let result = runtime
            .execute(&function, json!({}), context, &config)
            .unwrap();
        assert!(!result.success);
        assert!(result.error.is_some());
    }

    #[test]
    fn test_execution_context_with_env() {
        let function = create_test_function();
        let context = ExecutionContext::new(&function, None)
            .with_env("API_KEY", "secret")
            .with_env("DEBUG", "true");

        assert_eq!(context.env.get("API_KEY"), Some(&"secret".to_string()));
        assert_eq!(context.env.get("DEBUG"), Some(&"true".to_string()));
    }

    #[test]
    fn test_host_log() {
        let mut logs = Vec::new();
        host::log("test message", &mut logs);
        assert_eq!(logs.len(), 1);
        assert!(logs[0].contains("test message"));
    }

    #[test]
    fn test_host_env_get() {
        let function = create_test_function();
        let context = ExecutionContext::new(&function, None).with_env("MY_VAR", "my_value");

        assert_eq!(
            host::env_get("MY_VAR", &context),
            Some("my_value".to_string())
        );
        assert_eq!(host::env_get("MISSING", &context), None);
    }
}
