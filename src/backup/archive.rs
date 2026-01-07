//! Tar archive creation for backup
//!
//! Per BACKUP.md §3 and §5:
//! - Standard tar format
//! - No compression (Phase 1 limitation)
//! - Deterministic file ordering
//! - fsync archive after creation

use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;

use tar::Builder;

use super::errors::{BackupError, BackupResult};

/// Create a tar archive from a source directory
///
/// Per BACKUP.md:
/// - Archive format: backup.tar
/// - Deterministic file ordering
/// - No compression
pub fn create_tar_archive(source_dir: &Path, output_path: &Path) -> BackupResult<()> {
    // Create output file
    let file = File::create(output_path).map_err(|e| {
        BackupError::io_error(
            format!("Failed to create archive file: {}", output_path.display()),
            e,
        )
    })?;

    let writer = BufWriter::new(file);
    let mut builder = Builder::new(writer);

    // Collect and sort entries for deterministic ordering
    let mut entries = collect_entries(source_dir)?;
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    // Add each entry to the archive
    for (archive_path, fs_path) in entries {
        if fs_path.is_dir() {
            builder.append_dir(&archive_path, &fs_path).map_err(|e| {
                BackupError::io_error(
                    format!("Failed to add directory to archive: {}", archive_path),
                    std::io::Error::new(std::io::ErrorKind::Other, e),
                )
            })?;
        } else {
            let mut file =
                File::open(&fs_path).map_err(|e| BackupError::io_error_at_path(&fs_path, e))?;

            builder.append_file(&archive_path, &mut file).map_err(|e| {
                BackupError::io_error(
                    format!("Failed to add file to archive: {}", archive_path),
                    std::io::Error::new(std::io::ErrorKind::Other, e),
                )
            })?;
        }
    }

    // Finish the archive
    let writer = builder.into_inner().map_err(|e| {
        BackupError::io_error(
            "Failed to finish archive",
            std::io::Error::new(std::io::ErrorKind::Other, e),
        )
    })?;

    // Flush the buffer
    let mut file = writer.into_inner().map_err(|e| {
        BackupError::io_error(
            "Failed to flush archive buffer",
            std::io::Error::new(std::io::ErrorKind::Other, e),
        )
    })?;

    // fsync the archive file
    file.sync_all().map_err(|e| {
        BackupError::io_error(
            format!("Failed to fsync archive: {}", output_path.display()),
            e,
        )
    })?;

    Ok(())
}

/// Collect all entries from a directory recursively
fn collect_entries(dir: &Path) -> BackupResult<Vec<(String, std::path::PathBuf)>> {
    let mut entries = Vec::new();
    collect_entries_recursive(dir, "", &mut entries)?;
    Ok(entries)
}

fn collect_entries_recursive(
    current_dir: &Path,
    prefix: &str,
    entries: &mut Vec<(String, std::path::PathBuf)>,
) -> BackupResult<()> {
    let mut dir_entries: Vec<_> = fs::read_dir(current_dir)
        .map_err(|e| BackupError::io_error_at_path(current_dir, e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| BackupError::io_error_at_path(current_dir, e))?;

    // Sort by file name for deterministic ordering
    dir_entries.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    for entry in dir_entries {
        let fs_path = entry.path();
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();

        let archive_path = if prefix.is_empty() {
            file_name_str.to_string()
        } else {
            format!("{}/{}", prefix, file_name_str)
        };

        entries.push((archive_path.clone(), fs_path.clone()));

        if fs_path.is_dir() {
            collect_entries_recursive(&fs_path, &archive_path, entries)?;
        }
    }

    Ok(())
}

/// fsync the archive file
///
/// Per BACKUP.md §5: fsync backup.tar
pub fn fsync_archive(archive_path: &Path) -> BackupResult<()> {
    let file = OpenOptions::new()
        .read(true)
        .open(archive_path)
        .map_err(|e| BackupError::io_error_at_path(archive_path, e))?;

    file.sync_all().map_err(|e| {
        BackupError::io_error(
            format!("Failed to fsync archive: {}", archive_path.display()),
            e,
        )
    })?;

    // Also fsync parent directory
    if let Some(parent) = archive_path.parent() {
        let dir = OpenOptions::new()
            .read(true)
            .open(parent)
            .map_err(|e| BackupError::io_error_at_path(parent, e))?;

        dir.sync_all().map_err(|e| {
            BackupError::io_error(
                format!("Failed to fsync archive directory: {}", parent.display()),
                e,
            )
        })?;
    }

    Ok(())
}

/// Delete a partial archive if it exists
///
/// Per BACKUP.md §5: Partial backups must be deleted
pub fn cleanup_partial_archive(archive_path: &Path) {
    if archive_path.exists() {
        let _ = fs::remove_file(archive_path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use tar::Archive;
    use tempfile::TempDir;

    fn create_test_backup_structure(temp_dir: &Path) {
        // Create snapshot directory
        let snapshot_dir = temp_dir.join("snapshot");
        fs::create_dir_all(&snapshot_dir).unwrap();

        let mut f = File::create(snapshot_dir.join("storage.dat")).unwrap();
        f.write_all(b"storage data").unwrap();

        let mut f = File::create(snapshot_dir.join("manifest.json")).unwrap();
        f.write_all(br#"{"id":"test"}"#).unwrap();

        let schemas_dir = snapshot_dir.join("schemas");
        fs::create_dir_all(&schemas_dir).unwrap();

        let mut f = File::create(schemas_dir.join("user_v1.json")).unwrap();
        f.write_all(br#"{"name":"user"}"#).unwrap();

        // Create WAL directory
        let wal_dir = temp_dir.join("wal");
        fs::create_dir_all(&wal_dir).unwrap();

        let mut f = File::create(wal_dir.join("wal.log")).unwrap();
        f.write_all(b"wal data").unwrap();

        // Create backup manifest
        let mut f = File::create(temp_dir.join("backup_manifest.json")).unwrap();
        f.write_all(br#"{"backup_id":"test"}"#).unwrap();
    }

    #[test]
    fn test_create_tar_archive() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("source");
        fs::create_dir_all(&source_dir).unwrap();

        create_test_backup_structure(&source_dir);

        let archive_path = temp_dir.path().join("backup.tar");
        create_tar_archive(&source_dir, &archive_path).unwrap();

        assert!(archive_path.exists());
    }

    #[test]
    fn test_archive_contains_required_files() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("source");
        fs::create_dir_all(&source_dir).unwrap();

        create_test_backup_structure(&source_dir);

        let archive_path = temp_dir.path().join("backup.tar");
        create_tar_archive(&source_dir, &archive_path).unwrap();

        // Read archive and verify contents
        let file = File::open(&archive_path).unwrap();
        let mut archive = Archive::new(file);

        let entries: Vec<String> = archive
            .entries()
            .unwrap()
            .map(|e| e.unwrap().path().unwrap().to_string_lossy().to_string())
            .collect();

        assert!(entries.iter().any(|e| e.contains("snapshot")));
        assert!(entries.iter().any(|e| e.contains("storage.dat")));
        assert!(entries.iter().any(|e| e.contains("manifest.json")));
        assert!(entries.iter().any(|e| e.contains("schemas")));
        assert!(entries.iter().any(|e| e.contains("wal")));
        assert!(entries.iter().any(|e| e.contains("backup_manifest.json")));
    }

    #[test]
    fn test_archive_deterministic() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("source");
        fs::create_dir_all(&source_dir).unwrap();

        create_test_backup_structure(&source_dir);

        // Create first archive
        let archive1_path = temp_dir.path().join("backup1.tar");
        create_tar_archive(&source_dir, &archive1_path).unwrap();

        // Create second archive
        let archive2_path = temp_dir.path().join("backup2.tar");
        create_tar_archive(&source_dir, &archive2_path).unwrap();

        // Read both archives
        let mut content1 = Vec::new();
        File::open(&archive1_path)
            .unwrap()
            .read_to_end(&mut content1)
            .unwrap();

        let mut content2 = Vec::new();
        File::open(&archive2_path)
            .unwrap()
            .read_to_end(&mut content2)
            .unwrap();

        // Should be identical
        assert_eq!(content1, content2);
    }

    #[test]
    fn test_fsync_archive() {
        let temp_dir = TempDir::new().unwrap();
        let archive_path = temp_dir.path().join("test.tar");

        // Create a test file
        let mut f = File::create(&archive_path).unwrap();
        f.write_all(b"test archive").unwrap();
        f.sync_all().unwrap();

        // Should not error
        fsync_archive(&archive_path).unwrap();
    }

    #[test]
    fn test_cleanup_partial_archive() {
        let temp_dir = TempDir::new().unwrap();
        let archive_path = temp_dir.path().join("partial.tar");

        // Create partial archive
        let mut f = File::create(&archive_path).unwrap();
        f.write_all(b"partial").unwrap();

        assert!(archive_path.exists());

        cleanup_partial_archive(&archive_path);

        assert!(!archive_path.exists());
    }

    #[test]
    fn test_cleanup_nonexistent_archive() {
        let temp_dir = TempDir::new().unwrap();
        let archive_path = temp_dir.path().join("nonexistent.tar");

        // Should not panic
        cleanup_partial_archive(&archive_path);
    }
}
