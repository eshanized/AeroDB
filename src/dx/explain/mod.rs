//! Explanation Engine
//!
//! Per DX_EXPLANATION_MODEL.md §2:
//! - Explanations are structured proofs over observed state
//! - Not summaries, guesses, or narratives
//!
//! Per DX_EXPLANATION_MODEL.md §3:
//! - An explanation IS: structured record, sequence of facts, traceable path
//! - An explanation IS NOT: human-friendly story, heuristic guess
//!
//! Read-only, Phase 4, no semantic authority.

pub mod checkpoint;
pub mod model;
pub mod query;
pub mod recovery;
pub mod replication;
pub mod rules;
pub mod visibility;

pub use model::{
    Conclusion, Evidence, Explanation, ExplanationType, RuleApplication, RuleEvaluation,
};
pub use rules::RuleRegistry;
