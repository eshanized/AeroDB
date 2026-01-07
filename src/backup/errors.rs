//! Backup-specific error types
//!
//! Per ERRORS.md, backup errors follow the standard error model:
//! - Structured error codes in AERO_CATEGORY_NAME format
//! - Clear severity levels
//! - No silent failures
//!
//! All backup errors are ERROR severity (not FATAL) per spec.
//! Backup failure does NOT corrupt serving state.

use std::fmt;
use std::io;

/// Error severity levels per ERRORS.md
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Operation failed but system is healthy
    Error,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "ERROR"),
        }
    }
}

/// Backup error codes per ERRORS.md format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackupErrorCode {
    /// General backup creation failure
    AeroBackupFailed,
    /// I/O failure during backup
    AeroBackupIo,
    /// Manifest read/write failure
    AeroBackupManifest,
}

impl BackupErrorCode {
    /// Returns the string representation per ERRORS.md format
    pub fn as_str(&self) -> &'static str {
        match self {
            BackupErrorCode::AeroBackupFailed => "AERO_BACKUP_FAILED",
            BackupErrorCode::AeroBackupIo => "AERO_BACKUP_IO",
            BackupErrorCode::AeroBackupManifest => "AERO_BACKUP_MANIFEST",
        }
    }

    /// Returns the severity level for this error code
    pub fn severity(&self) -> Severity {
        // All backup errors are ERROR severity per spec
        Severity::Error
    }
}

impl fmt::Display for BackupErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Backup error with full context
#[derive(Debug)]
pub struct BackupError {
    /// Error code following AERO_CATEGORY_NAME format
    code: BackupErrorCode,
    /// Human-readable error message
    message: String,
    /// Optional underlying IO error
    source: Option<io::Error>,
}

impl BackupError {
    /// Creates a new backup error
    fn new(code: BackupErrorCode, message: impl Into<String>, source: Option<io::Error>) -> Self {
        Self {
            code,
            message: message.into(),
            source,
        }
    }

    /// Creates a general backup failure error
    pub fn failed(message: impl Into<String>) -> Self {
        Self::new(BackupErrorCode::AeroBackupFailed, message, None)
    }

    /// Creates a backup failure with source error
    pub fn failed_with_source(message: impl Into<String>, source: io::Error) -> Self {
        Self::new(BackupErrorCode::AeroBackupFailed, message, Some(source))
    }

    /// Creates an I/O error during backup
    pub fn io_error(message: impl Into<String>, source: io::Error) -> Self {
        Self::new(BackupErrorCode::AeroBackupIo, message, Some(source))
    }

    /// Creates an I/O error at a specific path
    pub fn io_error_at_path(path: &std::path::Path, source: io::Error) -> Self {
        Self::io_error(format!("I/O error at {}", path.display()), source)
    }

    /// Creates a manifest error
    pub fn manifest_failed(message: impl Into<String>) -> Self {
        Self::new(BackupErrorCode::AeroBackupManifest, message, None)
    }

    /// Creates a manifest error with source
    pub fn manifest_failed_with_source(message: impl Into<String>, source: io::Error) -> Self {
        Self::new(BackupErrorCode::AeroBackupManifest, message, Some(source))
    }

    /// Returns the error code
    pub fn code(&self) -> BackupErrorCode {
        self.code
    }

    /// Returns the error message
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the severity of this error
    pub fn severity(&self) -> Severity {
        self.code.severity()
    }

    /// Returns whether this error is fatal (requires process termination)
    /// Backup errors are never fatal - they are ERROR severity
    pub fn is_fatal(&self) -> bool {
        false
    }
}

impl fmt::Display for BackupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {}: {}",
            self.code.severity(),
            self.code,
            self.message
        )?;
        if let Some(ref source) = self.source {
            write!(f, " (caused by: {})", source)?;
        }
        Ok(())
    }
}

impl std::error::Error for BackupError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e as &(dyn std::error::Error + 'static))
    }
}

/// Result type for backup operations
pub type BackupResult<T> = Result<T, BackupError>;

/// Convert from snapshot errors to backup errors
impl From<crate::snapshot::SnapshotError> for BackupError {
    fn from(err: crate::snapshot::SnapshotError) -> Self {
        BackupError::failed(format!("Snapshot error: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes_match_spec() {
        assert_eq!(
            BackupErrorCode::AeroBackupFailed.as_str(),
            "AERO_BACKUP_FAILED"
        );
        assert_eq!(BackupErrorCode::AeroBackupIo.as_str(), "AERO_BACKUP_IO");
        assert_eq!(
            BackupErrorCode::AeroBackupManifest.as_str(),
            "AERO_BACKUP_MANIFEST"
        );
    }

    #[test]
    fn test_all_errors_are_error_severity() {
        let codes = [
            BackupErrorCode::AeroBackupFailed,
            BackupErrorCode::AeroBackupIo,
            BackupErrorCode::AeroBackupManifest,
        ];

        for code in codes {
            assert_eq!(code.severity(), Severity::Error);
        }
    }

    #[test]
    fn test_backup_errors_not_fatal() {
        let err = BackupError::failed("test error");
        assert!(!err.is_fatal());
        assert_eq!(err.severity(), Severity::Error);
    }

    #[test]
    fn test_error_display_contains_required_fields() {
        let err = BackupError::failed("test message");
        let display = format!("{}", err);

        assert!(display.contains("ERROR"));
        assert!(display.contains("AERO_BACKUP_FAILED"));
        assert!(display.contains("test message"));
    }

    #[test]
    fn test_io_error() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err = BackupError::io_error("could not read file", io_err);

        assert_eq!(err.code(), BackupErrorCode::AeroBackupIo);
        assert!(!err.is_fatal());
    }

    #[test]
    fn test_manifest_error() {
        let err = BackupError::manifest_failed("invalid manifest");

        assert_eq!(err.code(), BackupErrorCode::AeroBackupManifest);
        assert!(!err.is_fatal());
    }

    #[test]
    fn test_error_with_source() {
        let io_err = io::Error::new(io::ErrorKind::Other, "disk full");
        let err = BackupError::failed_with_source("backup failed", io_err);

        let display = format!("{}", err);
        assert!(display.contains("caused by"));
        assert!(display.contains("disk full"));
    }
}
