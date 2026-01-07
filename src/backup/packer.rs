//! File collection and temp directory handling for backup
//!
//! Per BACKUP.md §4:
//! - Copy snapshot → temp directory
//! - Copy WAL tail → temp directory
//! - fsync temp directory
//!
//! Backup is read-only. No modifications to source files.

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use super::errors::{BackupError, BackupResult};

/// Locate the latest valid snapshot directory
///
/// Per BACKUP.md §3.1, we need the latest valid snapshot.
/// Snapshots are identified by their ID (RFC3339 basic format: YYYYMMDDTHHMMSSZ).
pub fn find_latest_snapshot(snapshots_dir: &Path) -> BackupResult<PathBuf> {
    if !snapshots_dir.exists() {
        return Err(BackupError::failed(format!(
            "Snapshots directory does not exist: {}",
            snapshots_dir.display()
        )));
    }

    let mut snapshots: Vec<PathBuf> = Vec::new();

    let entries = fs::read_dir(snapshots_dir).map_err(|e| {
        BackupError::io_error(
            format!(
                "Failed to read snapshots directory: {}",
                snapshots_dir.display()
            ),
            e,
        )
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| BackupError::io_error("Failed to read snapshot entry", e))?;

        let path = entry.path();
        if path.is_dir() {
            // Verify it has a manifest.json (valid snapshot)
            let manifest_path = path.join("manifest.json");
            if manifest_path.exists() {
                snapshots.push(path);
            }
        }
    }

    if snapshots.is_empty() {
        return Err(BackupError::failed("No valid snapshots found"));
    }

    // Sort by name (which is timestamp-based) to get latest
    snapshots.sort();

    // Return the latest (last after sorting)
    Ok(snapshots.pop().unwrap())
}

/// Get the snapshot ID from a snapshot directory path
pub fn get_snapshot_id(snapshot_dir: &Path) -> BackupResult<String> {
    snapshot_dir
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .ok_or_else(|| BackupError::failed("Invalid snapshot directory name"))
}

/// Copy a file from source to destination with fsync
fn copy_file_with_fsync(src: &Path, dst: &Path) -> BackupResult<()> {
    // Create parent directories if needed
    if let Some(parent) = dst.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| {
                BackupError::io_error(
                    format!("Failed to create directory: {}", parent.display()),
                    e,
                )
            })?;
        }
    }

    // Read source file
    let mut src_file = File::open(src).map_err(|e| BackupError::io_error_at_path(src, e))?;
    let mut contents = Vec::new();
    src_file
        .read_to_end(&mut contents)
        .map_err(|e| BackupError::io_error_at_path(src, e))?;

    // Write to destination
    let mut dst_file = File::create(dst).map_err(|e| BackupError::io_error_at_path(dst, e))?;
    dst_file
        .write_all(&contents)
        .map_err(|e| BackupError::io_error_at_path(dst, e))?;

    // fsync
    dst_file
        .sync_all()
        .map_err(|e| BackupError::io_error_at_path(dst, e))?;

    Ok(())
}

/// Copy a directory recursively with fsync
fn copy_dir_recursive(src: &Path, dst: &Path) -> BackupResult<()> {
    if !src.exists() {
        return Err(BackupError::failed(format!(
            "Source directory does not exist: {}",
            src.display()
        )));
    }

    // Create destination directory
    fs::create_dir_all(dst).map_err(|e| {
        BackupError::io_error(format!("Failed to create directory: {}", dst.display()), e)
    })?;

    let entries = fs::read_dir(src).map_err(|e| BackupError::io_error_at_path(src, e))?;

    for entry in entries {
        let entry = entry.map_err(|e| BackupError::io_error_at_path(src, e))?;
        let src_path = entry.path();
        let file_name = entry.file_name();
        let dst_path = dst.join(&file_name);

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            copy_file_with_fsync(&src_path, &dst_path)?;
        }
    }

    // fsync the directory itself
    let dir = OpenOptions::new()
        .read(true)
        .open(dst)
        .map_err(|e| BackupError::io_error_at_path(dst, e))?;

    dir.sync_all().map_err(|e| {
        BackupError::io_error(format!("Failed to fsync directory: {}", dst.display()), e)
    })?;

    Ok(())
}

/// Copy snapshot directory to temp
///
/// Per BACKUP.md §3.1:
/// - storage.dat
/// - schemas/
/// - manifest.json
/// - Indexes excluded
pub fn copy_snapshot_to_temp(snapshot_dir: &Path, temp_dir: &Path) -> BackupResult<()> {
    let snapshot_dest = temp_dir.join("snapshot");
    fs::create_dir_all(&snapshot_dest).map_err(|e| {
        BackupError::io_error(
            format!(
                "Failed to create snapshot temp dir: {}",
                snapshot_dest.display()
            ),
            e,
        )
    })?;

    // Copy storage.dat
    let storage_src = snapshot_dir.join("storage.dat");
    if storage_src.exists() {
        let storage_dst = snapshot_dest.join("storage.dat");
        copy_file_with_fsync(&storage_src, &storage_dst)?;
    }

    // Copy manifest.json
    let manifest_src = snapshot_dir.join("manifest.json");
    if manifest_src.exists() {
        let manifest_dst = snapshot_dest.join("manifest.json");
        copy_file_with_fsync(&manifest_src, &manifest_dst)?;
    } else {
        return Err(BackupError::failed("Snapshot manifest.json not found"));
    }

    // Copy schemas directory
    let schemas_src = snapshot_dir.join("schemas");
    if schemas_src.exists() && schemas_src.is_dir() {
        let schemas_dst = snapshot_dest.join("schemas");
        copy_dir_recursive(&schemas_src, &schemas_dst)?;
    }

    Ok(())
}

/// Copy WAL directory to temp (if present)
///
/// Per BACKUP.md §3.2:
/// - Contains WAL tail after snapshot
/// - Byte-for-byte copy
/// - fsync before packaging
///
/// Returns whether WAL was present and copied.
pub fn copy_wal_to_temp(wal_dir: &Path, temp_dir: &Path) -> BackupResult<bool> {
    let wal_file = wal_dir.join("wal.log");

    if !wal_file.exists() {
        // No WAL file, return false
        return Ok(false);
    }

    // Check if WAL file has content
    let metadata =
        fs::metadata(&wal_file).map_err(|e| BackupError::io_error_at_path(&wal_file, e))?;
    if metadata.len() == 0 {
        // Empty WAL, still return true but it's empty
        let wal_dest = temp_dir.join("wal");
        fs::create_dir_all(&wal_dest).map_err(|e| {
            BackupError::io_error(
                format!("Failed to create WAL temp dir: {}", wal_dest.display()),
                e,
            )
        })?;

        // Create empty wal.log
        let wal_dst = wal_dest.join("wal.log");
        let file =
            File::create(&wal_dst).map_err(|e| BackupError::io_error_at_path(&wal_dst, e))?;
        file.sync_all()
            .map_err(|e| BackupError::io_error_at_path(&wal_dst, e))?;

        return Ok(true);
    }

    let wal_dest = temp_dir.join("wal");
    fs::create_dir_all(&wal_dest).map_err(|e| {
        BackupError::io_error(
            format!("Failed to create WAL temp dir: {}", wal_dest.display()),
            e,
        )
    })?;

    let wal_dst = wal_dest.join("wal.log");
    copy_file_with_fsync(&wal_file, &wal_dst)?;

    Ok(true)
}

/// fsync a directory recursively
///
/// Per BACKUP.md §4: fsync temp directory
pub fn fsync_recursive(dir: &Path) -> BackupResult<()> {
    if !dir.exists() {
        return Ok(());
    }

    let entries = fs::read_dir(dir).map_err(|e| BackupError::io_error_at_path(dir, e))?;

    for entry in entries {
        let entry = entry.map_err(|e| BackupError::io_error_at_path(dir, e))?;
        let path = entry.path();

        if path.is_dir() {
            fsync_recursive(&path)?;
        } else {
            // fsync the file
            let file = OpenOptions::new()
                .read(true)
                .open(&path)
                .map_err(|e| BackupError::io_error_at_path(&path, e))?;

            file.sync_all().map_err(|e| {
                BackupError::io_error(format!("Failed to fsync: {}", path.display()), e)
            })?;
        }
    }

    // fsync the directory itself
    let dir_handle = OpenOptions::new()
        .read(true)
        .open(dir)
        .map_err(|e| BackupError::io_error_at_path(dir, e))?;

    dir_handle.sync_all().map_err(|e| {
        BackupError::io_error(format!("Failed to fsync directory: {}", dir.display()), e)
    })?;

    Ok(())
}

/// Create temp backup directory
pub fn create_temp_backup_dir(data_dir: &Path) -> BackupResult<PathBuf> {
    let temp_dir = data_dir.join(".backup_temp");

    // Clean up any existing temp directory
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).map_err(|e| {
            BackupError::io_error(
                format!(
                    "Failed to clean up existing temp dir: {}",
                    temp_dir.display()
                ),
                e,
            )
        })?;
    }

    fs::create_dir_all(&temp_dir).map_err(|e| {
        BackupError::io_error(
            format!("Failed to create temp dir: {}", temp_dir.display()),
            e,
        )
    })?;

    Ok(temp_dir)
}

/// Cleanup temp directory
///
/// Per BACKUP.md §4: Temporary directories removed
pub fn cleanup_temp_dir(temp_dir: &Path) {
    if temp_dir.exists() {
        let _ = fs::remove_dir_all(temp_dir);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_snapshot(data_dir: &Path, snapshot_id: &str) -> PathBuf {
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
        f.write_all(br#"{"snapshot_id":"test"}"#).unwrap();
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

    fn create_test_wal(data_dir: &Path) {
        let wal_dir = data_dir.join("wal");
        fs::create_dir_all(&wal_dir).unwrap();

        let wal_path = wal_dir.join("wal.log");
        let mut f = File::create(&wal_path).unwrap();
        f.write_all(b"test wal data").unwrap();
        f.sync_all().unwrap();
    }

    #[test]
    fn test_find_latest_snapshot() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path();

        // Create multiple snapshots
        create_test_snapshot(data_dir, "20260101T100000Z");
        create_test_snapshot(data_dir, "20260102T100000Z");
        create_test_snapshot(data_dir, "20260103T100000Z");

        let snapshots_dir = data_dir.join("snapshots");
        let latest = find_latest_snapshot(&snapshots_dir).unwrap();

        assert!(latest.ends_with("20260103T100000Z"));
    }

    #[test]
    fn test_find_latest_snapshot_no_snapshots() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path();

        let snapshots_dir = data_dir.join("snapshots");
        fs::create_dir_all(&snapshots_dir).unwrap();

        let result = find_latest_snapshot(&snapshots_dir);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_snapshot_id() {
        let path = PathBuf::from("/data/snapshots/20260204T163000Z");
        let id = get_snapshot_id(&path).unwrap();

        assert_eq!(id, "20260204T163000Z");
    }

    #[test]
    fn test_copy_snapshot_to_temp() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path();

        let snapshot_dir = create_test_snapshot(data_dir, "20260204T163000Z");
        let backup_temp = data_dir.join("backup_temp");
        fs::create_dir_all(&backup_temp).unwrap();

        copy_snapshot_to_temp(&snapshot_dir, &backup_temp).unwrap();

        // Verify files copied
        assert!(backup_temp.join("snapshot").join("storage.dat").exists());
        assert!(backup_temp.join("snapshot").join("manifest.json").exists());
        assert!(backup_temp.join("snapshot").join("schemas").exists());
    }

    #[test]
    fn test_copy_wal_to_temp() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path();

        create_test_wal(data_dir);

        let wal_dir = data_dir.join("wal");
        let backup_temp = data_dir.join("backup_temp");
        fs::create_dir_all(&backup_temp).unwrap();

        let wal_present = copy_wal_to_temp(&wal_dir, &backup_temp).unwrap();

        assert!(wal_present);
        assert!(backup_temp.join("wal").join("wal.log").exists());
    }

    #[test]
    fn test_copy_wal_no_wal() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path();

        // No WAL created
        let wal_dir = data_dir.join("wal");
        fs::create_dir_all(&wal_dir).unwrap(); // Directory but no wal.log

        let backup_temp = data_dir.join("backup_temp");
        fs::create_dir_all(&backup_temp).unwrap();

        let wal_present = copy_wal_to_temp(&wal_dir, &backup_temp).unwrap();

        assert!(!wal_present);
    }

    #[test]
    fn test_create_temp_backup_dir() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path();

        let backup_temp = create_temp_backup_dir(data_dir).unwrap();

        assert!(backup_temp.exists());
        assert!(backup_temp.is_dir());
    }

    #[test]
    fn test_cleanup_temp_dir() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path();

        let backup_temp = create_temp_backup_dir(data_dir).unwrap();
        assert!(backup_temp.exists());

        cleanup_temp_dir(&backup_temp);

        assert!(!backup_temp.exists());
    }

    #[test]
    fn test_fsync_recursive() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path();

        // Create nested structure
        let nested = data_dir.join("a").join("b").join("c");
        fs::create_dir_all(&nested).unwrap();

        let file_path = nested.join("test.txt");
        let mut f = File::create(&file_path).unwrap();
        f.write_all(b"test").unwrap();

        // Should not error
        fsync_recursive(data_dir).unwrap();
    }
}
