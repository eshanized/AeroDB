//! Promotion Subsystem
//!
//! Phase 6: Failover & Promotion
//!
//! Per PHASE6_VISION.md:
//! - Explicit, correctness-preserving failover and promotion
//! - Correctness over availability
//! - Determinism over automation
//! - Explicit authority over heuristics
//!
//! Per PHASE6_STATE_MACHINE.md:
//! - States are explicit and enumerable
//! - Transitions are event-driven, never inferred
//! - All transitions are deterministic
//! - No background or time-based transitions
//! - All authority changes are atomic
//! - All failures are explicit
//!
//! This module is ORTHOGONAL to Phase 5 replication.
//! It observes and constrains Phase 5 transitions but does not replace them.

mod state;
mod errors;
mod request;
mod controller;
mod validator;
mod transition;
mod marker;
mod integration;
mod observability;
#[cfg(test)]
mod crash_tests;

pub use state::{PromotionState, DenialReason};
pub use errors::{PromotionError, PromotionResult, PromotionErrorKind};
pub use request::{PromotionRequest, PromotionRequestResult};
pub use controller::PromotionController;
pub use validator::{PromotionValidator, ValidationResult, ValidationContext};
pub use transition::{AuthorityTransitionManager, TransitionResult, TransitionFailureReason};
pub use marker::{DurableMarker, AuthorityMarker};
pub use integration::{ReplicationIntegration, RebindResult};
pub use observability::{PromotionEvent, PromotionExplanation, PromotionOutcome, InvariantCheck, PromotionObserver};
