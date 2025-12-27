//! # AeroDB REST API Module
//!
//! Phase 9: Auto-generated REST API from schema
//!
//! This module provides HTTP endpoints for CRUD operations
//! on all collections, with RLS enforcement.

pub mod errors;
pub mod filter;
pub mod parser;
pub mod handler;
pub mod response;
pub mod server;

pub use errors::{RestError, RestResult};
pub use filter::{FilterExpr, FilterOperator};
pub use parser::QueryParams;
pub use handler::RestHandler;
pub use server::RestServer;
