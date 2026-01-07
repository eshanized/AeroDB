//! # AeroDB REST API Module
//!
//! Phase 9: Auto-generated REST API from schema
//!
//! This module provides HTTP endpoints for CRUD operations
//! on all collections, with RLS enforcement.

pub mod database;
pub mod errors;
pub mod filter;
pub mod generator;
pub mod handler;
pub mod parser;
pub mod response;
pub mod server;

pub use database::DatabaseFacade;
pub use errors::{RestError, RestResult};
pub use filter::{FilterExpr, FilterOperator};
pub use handler::RestHandler;
pub use parser::QueryParams;
pub use server::RestServer;
