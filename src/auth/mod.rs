//! # AeroDB Auth Module
//!
//! Phase 8: Authentication & Authorization
//!
//! This module provides user authentication, session management,
//! JWT tokens, and Row-Level Security for AeroDB.

pub mod errors;
pub mod crypto;
pub mod user;
pub mod session;
pub mod jwt;
pub mod rls;
pub mod api;

pub use errors::{AuthError, AuthResult};
pub use user::{User, UserRepository};
pub use session::{Session, SessionManager};
pub use jwt::{JwtManager, JwtClaims};
pub use rls::{RlsContext, RlsEnforcer, RlsPolicy};
