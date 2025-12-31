//! # AeroDB Serverless Functions Module
//!
//! Phase 12: Serverless Functions
//!
//! WebAssembly-based serverless functions with HTTP, database,
//! and scheduled triggers.

pub mod errors;
pub mod function;
pub mod registry;
pub mod trigger;
pub mod invoker;
pub mod scheduler;
pub mod runtime;

pub use errors::{FunctionError, FunctionResult};
pub use function::{Function, FunctionConfig};
pub use registry::FunctionRegistry;
pub use trigger::TriggerType;
pub use invoker::{Invoker, InvocationContext, InvocationResult};
pub use scheduler::Scheduler;
pub use runtime::{WasmRuntime, StubRuntime, RuntimeConfig, ExecutionContext, ExecutionResult};

