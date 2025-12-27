//! Phase 7 Control Plane API
//!
//! Per PHASE7_CONTROL_PLANE_ARCHITECTURE.md:
//! - Control Plane API validates request structure
//! - Enforces Phase 7 invariants
//! - Orchestrates confirmation flows
//! - Routes requests to kernel boundary adapter
//!
//! Per PHASE7_INVARIANTS.md §P7-S2:
//! - Phase 7 MUST NOT alter kernel timing, execution order, or durability
//! - Kernel behavior must be identical with or without Phase 7

mod commands;
mod confirmation;
mod errors;
mod handlers;
mod types;
mod authority;

pub use commands::{
    ControlPlaneCommand, InspectionCommand, DiagnosticCommand, ControlCommand,
};
pub use confirmation::{
    ConfirmationFlow, ConfirmationStatus, ConfirmationToken,
    EnhancedConfirmation, ConfirmationResult,
};
pub use errors::{
    ControlPlaneError, ControlPlaneErrorDomain, ControlPlaneResult,
};
pub use handlers::ControlPlaneHandler;
pub use types::{
    CommandRequest, CommandResponse, CommandOutcome,
    ClusterState, NodeState, ReplicationStatus, PromotionStateView,
};
pub use authority::{AuthorityLevel, AuthorityContext};
