//! # AeroDB File Storage Module
//!
//! Phase 11: File Storage
//!
//! S3-compatible file storage with RLS-based access control.

pub mod errors;
pub mod bucket;
pub mod file;
pub mod permissions;
pub mod backend;
pub mod local;
pub mod signed_url;

pub use errors::{StorageError, StorageResult};
pub use bucket::{Bucket, BucketConfig};
pub use file::{StorageObject, FileService};
pub use permissions::StoragePermissions;
pub use backend::StorageBackend;
pub use local::LocalBackend;
pub use signed_url::SignedUrlGenerator;
