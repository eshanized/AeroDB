//! Snapshot manifest structure and serialization
//!
//! Per SNAPSHOT.md §3.3:
//! The manifest.json is the authoritative snapshot descriptor.
//!
//! Format:
//! ```json
//! {
//!   "snapshot_id": "20260204T113000Z",
//!   "created_at": "2026-02-04T11:30:00Z",
//!   "storage_checksum": "crc32:deadbeef",
//!   "schema_checksums": {
//!     "user_v1.json": "crc32:abcd1234"
//!   },
//!   "format_version": 1
//! }
//! ```

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::errors::{SnapshotError, SnapshotResult};

/// Snapshot manifest per SNAPSHOT.md §3.3
///
/// This is the authoritative snapshot descriptor containing:
/// - Snapshot identification
/// - Creation timestamp
/// - Integrity checksums for all files
/// - Format version
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SnapshotManifest {
    /// Snapshot ID in RFC3339 basic format (YYYYMMDDTHHMMSSZ)
    pub snapshot_id: String,

    /// Creation timestamp in RFC3339 format (YYYY-MM-DDTHH:MM:SSZ)
    pub created_at: String,

    /// CRC32 checksum of storage.dat (format: "crc32:XXXXXXXX")
    pub storage_checksum: String,

    /// CRC32 checksums of schema files (filename -> checksum)
    pub schema_checksums: HashMap<String, String>,

    /// Manifest format version (always 1 in Phase 1)
    pub format_version: u8,
}

impl SnapshotManifest {
    /// Creates a new snapshot manifest.
    ///
    /// # Arguments
    ///
    /// * `snapshot_id` - Snapshot ID in RFC3339 basic format
    /// * `created_at` - Creation timestamp in RFC3339 format
    /// * `storage_checksum` - Formatted CRC32 checksum of storage.dat
    /// * `schema_checksums` - Map of schema filename to formatted checksum
    pub fn new(
        snapshot_id: impl Into<String>,
        created_at: impl Into<String>,
        storage_checksum: impl Into<String>,
        schema_checksums: HashMap<String, String>,
    ) -> Self {
        Self {
            snapshot_id: snapshot_id.into(),
            created_at: created_at.into(),
            storage_checksum: storage_checksum.into(),
            schema_checksums,
            format_version: 1,
        }
    }

    /// Serializes the manifest to pretty-printed JSON.
    ///
    /// # Errors
    ///
    /// Returns `SnapshotError::manifest_error` if serialization fails.
    pub fn to_json(&self) -> SnapshotResult<String> {
        serde_json::to_string_pretty(self).map_err(|e| {
            SnapshotError::manifest_error(format!("Failed to serialize manifest: {}", e))
        })
    }

    /// Deserializes a manifest from JSON.
    ///
    /// # Arguments
    ///
    /// * `json` - JSON string to parse
    ///
    /// # Errors
    ///
    /// Returns `SnapshotError::manifest_error` if parsing fails.
    pub fn from_json(json: &str) -> SnapshotResult<Self> {
        serde_json::from_str(json).map_err(|e| {
            SnapshotError::manifest_error(format!("Failed to parse manifest: {}", e))
        })
    }

    /// Writes the manifest to a file with fsync.
    ///
    /// Per SNAPSHOT.md §4:
    /// - fsync manifest.json after write
    ///
    /// # Arguments
    ///
    /// * `path` - Path to write the manifest to
    ///
    /// # Errors
    ///
    /// Returns `SnapshotError::manifest_io_error` if write or fsync fails.
    pub fn write_to_file(&self, path: &Path) -> SnapshotResult<()> {
        let json = self.to_json()?;

        let mut file = File::create(path).map_err(|e| {
            SnapshotError::manifest_io_error(
                format!("Failed to create manifest file: {}", path.display()),
                e,
            )
        })?;

        file.write_all(json.as_bytes()).map_err(|e| {
            SnapshotError::manifest_io_error(
                format!("Failed to write manifest: {}", path.display()),
                e,
            )
        })?;

        // fsync is mandatory per SNAPSHOT.md
        file.sync_all().map_err(|e| {
            SnapshotError::manifest_io_error(
                format!("Failed to fsync manifest: {}", path.display()),
                e,
            )
        })?;

        Ok(())
    }

    /// Reads a manifest from a file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to read the manifest from
    ///
    /// # Errors
    ///
    /// Returns `SnapshotError` if read or parse fails.
    pub fn read_from_file(path: &Path) -> SnapshotResult<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            SnapshotError::manifest_io_error(
                format!("Failed to read manifest: {}", path.display()),
                e,
            )
        })?;

        Self::from_json(&content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_manifest() -> SnapshotManifest {
        let mut schema_checksums = HashMap::new();
        schema_checksums.insert("user_v1.json".to_string(), "crc32:abcd1234".to_string());
        schema_checksums.insert("order_v1.json".to_string(), "crc32:12345678".to_string());

        SnapshotManifest::new(
            "20260204T113000Z",
            "2026-02-04T11:30:00Z",
            "crc32:deadbeef",
            schema_checksums,
        )
    }

    #[test]
    fn test_manifest_creation() {
        let manifest = create_test_manifest();
        assert_eq!(manifest.snapshot_id, "20260204T113000Z");
        assert_eq!(manifest.created_at, "2026-02-04T11:30:00Z");
        assert_eq!(manifest.storage_checksum, "crc32:deadbeef");
        assert_eq!(manifest.format_version, 1);
        assert_eq!(manifest.schema_checksums.len(), 2);
    }

    #[test]
    fn test_manifest_format_version_always_one() {
        let manifest = SnapshotManifest::new(
            "test",
            "test",
            "crc32:00000000",
            HashMap::new(),
        );
        assert_eq!(manifest.format_version, 1);
    }

    #[test]
    fn test_manifest_to_json() {
        let manifest = create_test_manifest();
        let json = manifest.to_json().unwrap();

        assert!(json.contains("\"snapshot_id\": \"20260204T113000Z\""));
        assert!(json.contains("\"created_at\": \"2026-02-04T11:30:00Z\""));
        assert!(json.contains("\"storage_checksum\": \"crc32:deadbeef\""));
        assert!(json.contains("\"format_version\": 1"));
        assert!(json.contains("\"schema_checksums\""));
    }

    #[test]
    fn test_manifest_json_roundtrip() {
        let original = create_test_manifest();
        let json = original.to_json().unwrap();
        let parsed = SnapshotManifest::from_json(&json).unwrap();

        assert_eq!(original, parsed);
    }

    #[test]
    fn test_manifest_write_and_read_file() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join("manifest.json");

        let original = create_test_manifest();
        original.write_to_file(&manifest_path).unwrap();

        // Verify file exists
        assert!(manifest_path.exists());

        // Read back and verify
        let loaded = SnapshotManifest::read_from_file(&manifest_path).unwrap();
        assert_eq!(original, loaded);
    }

    #[test]
    fn test_manifest_invalid_json() {
        let result = SnapshotManifest::from_json("not valid json");
        assert!(result.is_err());
    }

    #[test]
    fn test_manifest_missing_file() {
        let path = Path::new("/nonexistent/path/manifest.json");
        let result = SnapshotManifest::read_from_file(path);
        assert!(result.is_err());
    }

    #[test]
    fn test_manifest_json_matches_spec_format() {
        // Verify the JSON output format matches SNAPSHOT.md §3.3 exactly
        let mut schema_checksums = HashMap::new();
        schema_checksums.insert("user_v1.json".to_string(), "crc32:abcd1234".to_string());

        let manifest = SnapshotManifest::new(
            "20260204T113000Z",
            "2026-02-04T11:30:00Z",
            "crc32:deadbeef",
            schema_checksums,
        );

        let json = manifest.to_json().unwrap();

        // Parse as generic JSON to verify structure
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["snapshot_id"], "20260204T113000Z");
        assert_eq!(parsed["created_at"], "2026-02-04T11:30:00Z");
        assert_eq!(parsed["storage_checksum"], "crc32:deadbeef");
        assert_eq!(parsed["format_version"], 1);
        assert!(parsed["schema_checksums"].is_object());
        assert_eq!(parsed["schema_checksums"]["user_v1.json"], "crc32:abcd1234");
    }
}
