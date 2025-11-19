//! Backup manifest handling
//!
//! Per BACKUP.md §3.3, the backup manifest records:
//! - backup_id: The backup identifier (equals snapshot_id)
//! - created_at: RFC3339 timestamp
//! - snapshot_id: The source snapshot ID
//! - wal_present: Whether WAL is included
//! - format_version: Always 1 for Phase 1
//!
//! Location inside archive: `backup_manifest.json`

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use super::errors::{BackupError, BackupResult};

/// Backup manifest data structure per BACKUP.md §3.3
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackupManifest {
    /// Backup ID (equals snapshot_id per spec)
    pub backup_id: String,

    /// Timestamp when backup was created (RFC3339 format)
    pub created_at: String,

    /// Source snapshot ID
    pub snapshot_id: String,

    /// Whether WAL is included in the backup
    pub wal_present: bool,

    /// Format version (always 1 for Phase 1)
    pub format_version: u8,
}

impl BackupManifest {
    /// Creates a new backup manifest
    ///
    /// Per spec, backup_id equals snapshot_id.
    pub fn new(snapshot_id: &str, wal_present: bool) -> Self {
        let created_at = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

        Self {
            backup_id: snapshot_id.to_string(),
            created_at,
            snapshot_id: snapshot_id.to_string(),
            wal_present,
            format_version: 1,
        }
    }

    /// Creates a backup manifest with explicit created_at timestamp
    pub fn with_timestamp(snapshot_id: &str, created_at: &str, wal_present: bool) -> Self {
        Self {
            backup_id: snapshot_id.to_string(),
            created_at: created_at.to_string(),
            snapshot_id: snapshot_id.to_string(),
            wal_present,
            format_version: 1,
        }
    }

    /// Serializes the manifest to JSON
    pub fn to_json(&self) -> BackupResult<String> {
        serde_json::to_string_pretty(self).map_err(|e| {
            BackupError::manifest_failed(format!("Failed to serialize backup manifest: {}", e))
        })
    }

    /// Deserializes the manifest from JSON
    pub fn from_json(json: &str) -> BackupResult<Self> {
        serde_json::from_str(json).map_err(|e| {
            BackupError::manifest_failed(format!("Failed to parse backup manifest: {}", e))
        })
    }

    /// Writes the manifest to a file with fsync
    pub fn write_to_file(&self, path: &Path) -> BackupResult<()> {
        let json = self.to_json()?;

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| {
                    BackupError::io_error(
                        format!("Failed to create manifest directory: {}", parent.display()),
                        e,
                    )
                })?;
            }
        }

        // Write to file
        let mut file = File::create(path).map_err(|e| {
            BackupError::manifest_failed_with_source(
                format!("Failed to create manifest file: {}", path.display()),
                e,
            )
        })?;

        file.write_all(json.as_bytes()).map_err(|e| {
            BackupError::manifest_failed_with_source(
                format!("Failed to write manifest file: {}", path.display()),
                e,
            )
        })?;

        // fsync is mandatory
        file.sync_all().map_err(|e| {
            BackupError::io_error(
                format!("Failed to fsync manifest file: {}", path.display()),
                e,
            )
        })?;

        Ok(())
    }

    /// Reads a manifest from a file
    pub fn read_from_file(path: &Path) -> BackupResult<Self> {
        let mut file = File::open(path).map_err(|e| {
            BackupError::manifest_failed_with_source(
                format!("Failed to open manifest file: {}", path.display()),
                e,
            )
        })?;

        let mut contents = String::new();
        file.read_to_string(&mut contents).map_err(|e| {
            BackupError::manifest_failed_with_source(
                format!("Failed to read manifest file: {}", path.display()),
                e,
            )
        })?;

        Self::from_json(&contents)
    }
}

/// fsync a directory
pub fn fsync_dir(path: &Path) -> BackupResult<()> {
    let dir = OpenOptions::new()
        .read(true)
        .open(path)
        .map_err(|e| BackupError::io_error_at_path(path, e))?;

    dir.sync_all()
        .map_err(|e| BackupError::io_error(format!("fsync directory failed: {}", path.display()), e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_manifest_creation() {
        let manifest = BackupManifest::new("20260204T163000Z", true);

        assert_eq!(manifest.backup_id, "20260204T163000Z");
        assert_eq!(manifest.snapshot_id, "20260204T163000Z");
        assert!(manifest.wal_present);
        assert_eq!(manifest.format_version, 1);
        // backup_id equals snapshot_id per spec
        assert_eq!(manifest.backup_id, manifest.snapshot_id);
    }

    #[test]
    fn test_manifest_with_timestamp() {
        let manifest = BackupManifest::with_timestamp(
            "20260204T163000Z",
            "2026-02-04T16:30:00Z",
            false,
        );

        assert_eq!(manifest.created_at, "2026-02-04T16:30:00Z");
        assert!(!manifest.wal_present);
    }

    #[test]
    fn test_manifest_json_roundtrip() {
        let manifest = BackupManifest::with_timestamp(
            "20260204T163000Z",
            "2026-02-04T16:30:00Z",
            true,
        );

        let json = manifest.to_json().unwrap();
        let parsed = BackupManifest::from_json(&json).unwrap();

        assert_eq!(manifest, parsed);
    }

    #[test]
    fn test_manifest_json_format_matches_spec() {
        let manifest = BackupManifest::with_timestamp(
            "20260204T120000Z",
            "2026-02-04T12:00:00Z",
            true,
        );

        let json = manifest.to_json().unwrap();

        // Per BACKUP.md §3.3, verify expected fields
        assert!(json.contains("\"backup_id\""));
        assert!(json.contains("\"created_at\""));
        assert!(json.contains("\"snapshot_id\""));
        assert!(json.contains("\"wal_present\""));
        assert!(json.contains("\"format_version\""));
        assert!(json.contains("\"20260204T120000Z\""));
        assert!(json.contains("true")); // wal_present
        assert!(json.contains("1")); // format_version
    }

    #[test]
    fn test_manifest_write_and_read() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join("backup_manifest.json");

        let manifest = BackupManifest::with_timestamp(
            "20260204T163000Z",
            "2026-02-04T16:30:00Z",
            true,
        );

        // Write
        manifest.write_to_file(&manifest_path).unwrap();

        // Verify file exists
        assert!(manifest_path.exists());

        // Read back
        let read_manifest = BackupManifest::read_from_file(&manifest_path).unwrap();

        assert_eq!(manifest, read_manifest);
    }

    #[test]
    fn test_backup_id_equals_snapshot_id() {
        let manifest = BackupManifest::new("snapshot_123", true);

        assert_eq!(manifest.backup_id, "snapshot_123");
        assert_eq!(manifest.snapshot_id, "snapshot_123");
        assert_eq!(manifest.backup_id, manifest.snapshot_id);
    }

    #[test]
    fn test_format_version_always_one() {
        let manifest1 = BackupManifest::new("id1", true);
        let manifest2 = BackupManifest::new("id2", false);

        assert_eq!(manifest1.format_version, 1);
        assert_eq!(manifest2.format_version, 1);
    }

    #[test]
    fn test_manifest_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("nonexistent.json");

        let result = BackupManifest::read_from_file(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_manifest_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("bad.json");

        fs::write(&path, "not valid json").unwrap();

        let result = BackupManifest::read_from_file(&path);
        assert!(result.is_err());
    }
}
