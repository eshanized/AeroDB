//! Backup subsystem for aerodb
//!
//! Backup produces a portable, self-contained archive suitable for restore.
//!
//! Per BACKUP.md:
//! - Atomic consistency
//! - Full durability
//! - Explicit integrity verification
//! - Deterministic restore
//! - Zero partial success
//!
//! # Archive Format
//!
//! ```text
//! backup.tar
//! ├── snapshot/
//! │   ├── storage.dat
//! │   ├── schemas/
//! │   └── manifest.json
//! ├── wal/
//! │   └── wal.log
//! └── backup_manifest.json
//! ```
//!
//! # Algorithm (BACKUP.md §4)
//!
//! 1. Acquire global execution lock
//! 2. fsync WAL
//! 3. Identify latest valid snapshot
//! 4. Copy snapshot → temp directory
//! 5. Copy WAL tail → temp directory
//! 6. Generate backup_manifest.json
//! 7. fsync temp directory
//! 8. Package temp directory into tar
//! 9. fsync backup.tar
//! 10. Release global execution lock
//!
//! # Important
//!
//! Backup is read-only.
//! Backup does NOT create snapshots.
//! Backup does NOT modify WAL.
//! Backup does NOT truncate WAL.

mod archive;
mod errors;
mod manifest;
mod packer;

pub use errors::{BackupError, BackupErrorCode, BackupResult, Severity};
pub use manifest::BackupManifest;

use std::path::Path;

use crate::snapshot::GlobalExecutionLock;
use crate::wal::WalWriter;

use archive::{cleanup_partial_archive, create_tar_archive};
use packer::{
    cleanup_temp_dir, copy_snapshot_to_temp, copy_wal_to_temp, create_temp_backup_dir,
    find_latest_snapshot, fsync_recursive, get_snapshot_id,
};

/// Backup ID type (equals SnapshotId per spec)
pub type BackupId = String;

/// Backup manager for creating backup archives.
///
/// This struct provides the public API for backup operations.
///
/// # Usage
///
/// ```ignore
/// let lock = GlobalExecutionLock::new();
/// let backup_id = BackupManager::create_backup(
///     &data_dir,
///     &output_path,
///     &lock,
/// )?;
/// ```
pub struct BackupManager;

impl BackupManager {
    /// Create a backup archive.
    ///
    /// Per BACKUP.md §4, this follows the exact sequence:
    /// 1. Acquire global execution lock (caller responsibility, verified by lock param)
    /// 2. fsync WAL
    /// 3. Identify latest valid snapshot
    /// 4. Copy snapshot → temp directory
    /// 5. Copy WAL tail → temp directory
    /// 6. Generate backup_manifest.json
    /// 7. fsync temp directory
    /// 8. Package temp directory into tar
    /// 9. fsync backup.tar
    /// 10. Release lock (caller responsibility)
    ///
    /// # Arguments
    ///
    /// * `data_dir` - Root data directory
    /// * `output_path` - Path for the output backup.tar
    /// * `wal` - Reference to WAL writer (for fsync)
    /// * `lock` - Marker proving the caller holds the global execution lock
    ///
    /// # Returns
    ///
    /// The backup ID on success (equals snapshot ID).
    ///
    /// # Errors
    ///
    /// Returns `BackupError` on any failure:
    /// - No valid snapshot found
    /// - I/O error during copy
    /// - Archive creation failure
    ///
    /// Backup failure does NOT corrupt serving state.
    ///
    /// # Forbidden Operations
    ///
    /// This function does NOT:
    /// - Create snapshots
    /// - Modify WAL
    /// - Truncate WAL
    /// - Modify storage
    /// - Rebuild indexes
    /// - Spawn threads
    /// - Perform async IO
    pub fn create_backup(
        data_dir: &Path,
        output_path: &Path,
        wal: &WalWriter,
        _lock: &GlobalExecutionLock,
    ) -> Result<BackupId, BackupError> {
        // Step 2: fsync WAL to ensure all pending writes are durable
        wal.fsync().map_err(|e| {
            BackupError::failed(format!("Failed to fsync WAL: {}", e))
        })?;

        // Step 3: Identify latest valid snapshot
        let snapshots_dir = data_dir.join("snapshots");
        let snapshot_dir = find_latest_snapshot(&snapshots_dir)?;
        let snapshot_id = get_snapshot_id(&snapshot_dir)?;

        // Create temp directory
        let temp_dir = create_temp_backup_dir(data_dir)?;

        // Use a closure to ensure cleanup on error
        let result = (|| -> BackupResult<BackupId> {
            // Step 4: Copy snapshot → temp directory
            copy_snapshot_to_temp(&snapshot_dir, &temp_dir)?;

            // Step 5: Copy WAL tail → temp directory
            let wal_dir = data_dir.join("wal");
            let wal_present = copy_wal_to_temp(&wal_dir, &temp_dir)?;

            // Step 6: Generate backup_manifest.json
            let manifest = BackupManifest::new(&snapshot_id, wal_present);
            manifest.write_to_file(&temp_dir.join("backup_manifest.json"))?;

            // Step 7: fsync temp directory
            fsync_recursive(&temp_dir)?;

            // Step 8: Package temp directory into tar
            create_tar_archive(&temp_dir, output_path)?;

            // Step 9: fsync backup.tar (already done in create_tar_archive)

            Ok(snapshot_id.clone())
        })();

        // Cleanup temp directory
        cleanup_temp_dir(&temp_dir);

        // On error, also cleanup partial archive
        if result.is_err() {
            cleanup_partial_archive(output_path);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wal::WalWriter;
    use std::fs::{self, File};
    use std::io::Write;
    use tar::Archive;
    use tempfile::TempDir;

    fn setup_test_environment() -> (TempDir, std::path::PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path().to_path_buf();

        // Create WAL directory
        let wal_dir = data_dir.join("wal");
        fs::create_dir_all(&wal_dir).unwrap();

        // Create storage.dat
        let storage_path = data_dir.join("storage.dat");
        let mut storage_file = File::create(&storage_path).unwrap();
        storage_file.write_all(b"test storage data").unwrap();
        storage_file.sync_all().unwrap();

        // Create schema directory
        let schema_dir = data_dir.join("metadata").join("schemas");
        fs::create_dir_all(&schema_dir).unwrap();

        let schema_path = schema_dir.join("user_v1.json");
        let mut schema_file = File::create(&schema_path).unwrap();
        schema_file.write_all(br#"{"name": "user", "version": 1}"#).unwrap();
        schema_file.sync_all().unwrap();

        // Initialize WAL
        let mut wal = WalWriter::open(&data_dir).unwrap();
        use crate::wal::{RecordType, WalPayload};
        let payload = WalPayload::new("test", "doc1", "schema", "v1", b"{}".to_vec());
        wal.append(RecordType::Insert, payload).unwrap();

        (temp_dir, storage_path)
    }

    fn create_test_snapshot(data_dir: &std::path::Path, snapshot_id: &str) -> std::path::PathBuf {
        let snapshot_dir = data_dir.join("snapshots").join(snapshot_id);
        fs::create_dir_all(&snapshot_dir).unwrap();

        // Create storage.dat
        let storage_path = snapshot_dir.join("storage.dat");
        let mut f = File::create(&storage_path).unwrap();
        f.write_all(b"test storage data").unwrap();
        f.sync_all().unwrap();

        // Create manifest.json
        let manifest_path = snapshot_dir.join("manifest.json");
        let mut f = File::create(&manifest_path).unwrap();
        f.write_all(format!(r#"{{"snapshot_id":"{}"}}"#, snapshot_id).as_bytes()).unwrap();
        f.sync_all().unwrap();

        // Create schemas directory
        let schemas_dir = snapshot_dir.join("schemas");
        fs::create_dir_all(&schemas_dir).unwrap();

        let schema_path = schemas_dir.join("user_v1.json");
        let mut f = File::create(&schema_path).unwrap();
        f.write_all(br#"{"name":"user"}"#).unwrap();
        f.sync_all().unwrap();

        snapshot_dir
    }

    #[test]
    fn test_backup_manager_create_backup() {
        let (temp_dir, _) = setup_test_environment();
        let data_dir = temp_dir.path();

        // Create a snapshot
        create_test_snapshot(data_dir, "20260204T163000Z");

        // Create WAL
        let wal = WalWriter::open(data_dir).unwrap();
        let lock = GlobalExecutionLock::new();

        let output_path = data_dir.join("backup.tar");

        let result = BackupManager::create_backup(
            data_dir,
            &output_path,
            &wal,
            &lock,
        );

        assert!(result.is_ok());
        assert!(output_path.exists());

        let backup_id = result.unwrap();
        assert_eq!(backup_id, "20260204T163000Z");
    }

    #[test]
    fn test_backup_id_equals_snapshot_id() {
        let (temp_dir, _) = setup_test_environment();
        let data_dir = temp_dir.path();

        create_test_snapshot(data_dir, "20260204T163000Z");

        let wal = WalWriter::open(data_dir).unwrap();
        let lock = GlobalExecutionLock::new();

        let output_path = data_dir.join("backup.tar");

        let backup_id = BackupManager::create_backup(
            data_dir,
            &output_path,
            &wal,
            &lock,
        ).unwrap();

        // Backup ID must equal snapshot ID
        assert_eq!(backup_id, "20260204T163000Z");
    }

    #[test]
    fn test_backup_selects_latest_snapshot() {
        let (temp_dir, _) = setup_test_environment();
        let data_dir = temp_dir.path();

        // Create multiple snapshots
        create_test_snapshot(data_dir, "20260101T100000Z");
        create_test_snapshot(data_dir, "20260102T100000Z");
        create_test_snapshot(data_dir, "20260103T100000Z");

        let wal = WalWriter::open(data_dir).unwrap();
        let lock = GlobalExecutionLock::new();

        let output_path = data_dir.join("backup.tar");

        let backup_id = BackupManager::create_backup(
            data_dir,
            &output_path,
            &wal,
            &lock,
        ).unwrap();

        // Should select the latest snapshot
        assert_eq!(backup_id, "20260103T100000Z");
    }

    #[test]
    fn test_backup_archive_contains_required_files() {
        let (temp_dir, _) = setup_test_environment();
        let data_dir = temp_dir.path();

        create_test_snapshot(data_dir, "20260204T163000Z");

        let wal = WalWriter::open(data_dir).unwrap();
        let lock = GlobalExecutionLock::new();

        let output_path = data_dir.join("backup.tar");

        BackupManager::create_backup(
            data_dir,
            &output_path,
            &wal,
            &lock,
        ).unwrap();

        // Read archive and verify contents
        let file = File::open(&output_path).unwrap();
        let mut archive = Archive::new(file);

        let entries: Vec<String> = archive
            .entries()
            .unwrap()
            .map(|e| e.unwrap().path().unwrap().to_string_lossy().to_string())
            .collect();

        // Check required files per BACKUP.md §3
        assert!(entries.iter().any(|e| e.contains("snapshot")));
        assert!(entries.iter().any(|e| e.contains("storage.dat")));
        assert!(entries.iter().any(|e| e.contains("manifest.json")));
        assert!(entries.iter().any(|e| e.contains("schemas")));
        assert!(entries.iter().any(|e| e.contains("backup_manifest.json")));
    }

    #[test]
    fn test_backup_manifest_in_archive() {
        let (temp_dir, _) = setup_test_environment();
        let data_dir = temp_dir.path();

        create_test_snapshot(data_dir, "20260204T163000Z");

        let wal = WalWriter::open(data_dir).unwrap();
        let lock = GlobalExecutionLock::new();

        let output_path = data_dir.join("backup.tar");

        BackupManager::create_backup(
            data_dir,
            &output_path,
            &wal,
            &lock,
        ).unwrap();

        // Extract and verify backup manifest
        let file = File::open(&output_path).unwrap();
        let mut archive = Archive::new(file);

        for entry in archive.entries().unwrap() {
            let mut entry = entry.unwrap();
            let path = entry.path().unwrap().to_string_lossy().to_string();

            if path == "backup_manifest.json" {
                let mut contents = String::new();
                std::io::Read::read_to_string(&mut entry, &mut contents).unwrap();

                let manifest: BackupManifest = serde_json::from_str(&contents).unwrap();

                assert_eq!(manifest.backup_id, "20260204T163000Z");
                assert_eq!(manifest.snapshot_id, "20260204T163000Z");
                assert_eq!(manifest.format_version, 1);
                return;
            }
        }

        panic!("backup_manifest.json not found in archive");
    }

    #[test]
    fn test_backup_failure_no_snapshot() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path();

        // Create snapshots directory but no snapshots
        fs::create_dir_all(data_dir.join("snapshots")).unwrap();
        fs::create_dir_all(data_dir.join("wal")).unwrap();

        let wal = WalWriter::open(data_dir).unwrap();
        let lock = GlobalExecutionLock::new();

        let output_path = data_dir.join("backup.tar");

        let result = BackupManager::create_backup(
            data_dir,
            &output_path,
            &wal,
            &lock,
        );

        assert!(result.is_err());
        // No partial archive should exist
        assert!(!output_path.exists());
    }

    #[test]
    fn test_backup_cleanup_on_failure() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path();

        // No snapshots directory at all
        fs::create_dir_all(data_dir.join("wal")).unwrap();

        let wal = WalWriter::open(data_dir).unwrap();
        let lock = GlobalExecutionLock::new();

        let output_path = data_dir.join("backup.tar");

        let result = BackupManager::create_backup(
            data_dir,
            &output_path,
            &wal,
            &lock,
        );

        assert!(result.is_err());

        // Temp directory should be cleaned up
        assert!(!data_dir.join(".backup_temp").exists());
    }

    #[test]
    fn test_backup_error_not_fatal() {
        let err = BackupError::failed("test");
        assert!(!err.is_fatal());
    }

    #[test]
    fn test_lock_required() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path();

        fs::create_dir_all(data_dir.join("snapshots")).unwrap();
        fs::create_dir_all(data_dir.join("wal")).unwrap();

        // The lock marker must be provided - compile-time check
        let wal = WalWriter::open(data_dir).unwrap();
        let lock = GlobalExecutionLock::new();

        let output_path = data_dir.join("backup.tar");

        let _ = BackupManager::create_backup(
            data_dir,
            &output_path,
            &wal,
            &lock,
        );

        // If this compiles, the test passes
    }
}
