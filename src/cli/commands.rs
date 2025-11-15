//! CLI command implementations
//!
//! Per LIFECYCLE.md and BOOT.md, these commands follow strict boot sequence.

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::api::{ApiHandler, Subsystems};
use crate::index::IndexManager;
use crate::recovery::RecoveryManager;
use crate::schema::SchemaLoader;
use crate::storage::{StorageReader, StorageWriter};
use crate::wal::{WalReader, WalWriter};

use super::args::Command;
use super::errors::{CliError, CliResult};
use super::io::{read_request, read_requests, write_error, write_response, write_json};

/// Configuration file structure per CONFIG.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Data directory (required)
    pub data_dir: String,
    
    /// Max WAL size in bytes (optional, default 1GB)
    #[serde(default = "default_max_wal_size")]
    pub max_wal_size_bytes: u64,
    
    /// Max memory in bytes (optional, default 512MB)
    #[serde(default = "default_max_memory")]
    pub max_memory_bytes: u64,
    
    /// WAL sync mode (optional, default "fsync")
    #[serde(default = "default_wal_sync_mode")]
    pub wal_sync_mode: String,
}

fn default_max_wal_size() -> u64 { 1073741824 } // 1GB
fn default_max_memory() -> u64 { 536870912 } // 512MB
fn default_wal_sync_mode() -> String { "fsync".to_string() }

impl Config {
    /// Load configuration from file
    pub fn load(path: &Path) -> CliResult<Self> {
        let content = fs::read_to_string(path)
            .map_err(|e| CliError::config_error(format!("Failed to read config: {}", e)))?;
        
        let config: Config = serde_json::from_str(&content)
            .map_err(|e| CliError::config_error(format!("Invalid config JSON: {}", e)))?;
        
        config.validate()?;
        
        Ok(config)
    }
    
    /// Validate configuration per CONFIG.md
    fn validate(&self) -> CliResult<()> {
        // Validate wal_sync_mode
        if self.wal_sync_mode != "fsync" {
            return Err(CliError::config_error(
                format!("Invalid wal_sync_mode: '{}'. Only 'fsync' is allowed.", self.wal_sync_mode)
            ));
        }
        
        // Validate max_wal_size_bytes
        if self.max_wal_size_bytes == 0 {
            return Err(CliError::config_error("max_wal_size_bytes must be > 0"));
        }
        
        // Validate max_memory_bytes
        if self.max_memory_bytes == 0 {
            return Err(CliError::config_error("max_memory_bytes must be > 0"));
        }
        
        Ok(())
    }
    
    /// Get data directory as Path
    pub fn data_path(&self) -> &Path {
        Path::new(&self.data_dir)
    }
}

/// Main CLI entry point
///
/// Parses arguments and dispatches to the appropriate command.
/// This is the only function that main.rs should call.
pub fn run() -> CliResult<()> {
    let cli = super::args::Cli::parse_args();
    run_command(cli.command)
}

/// Run the appropriate command based on CLI args
pub fn run_command(cmd: Command) -> CliResult<()> {
    match cmd {
        Command::Init { config } => init(&config),
        Command::Start { config } => start(&config),
        Command::Query { config } => query(&config),
        Command::Explain { config } => explain(&config),
    }
}

/// Initialize a new AeroDB data directory
///
/// Per LIFECYCLE.md §2:
/// - Creates directory structure
/// - Does NOT start server
/// - Writes no WAL records
/// - Does not create clean_shutdown marker
pub fn init(config_path: &Path) -> CliResult<()> {
    let config = Config::load(config_path)?;
    let data_dir = config.data_path();
    
    // Check if already initialized
    if is_initialized(data_dir) {
        return Err(CliError::already_initialized());
    }
    
    // Create directory structure per CONFIG.md §4
    let dirs = [
        data_dir.join("wal"),
        data_dir.join("data"),
        data_dir.join("metadata").join("schemas"),
    ];
    
    for dir in &dirs {
        fs::create_dir_all(dir)
            .map_err(|e| CliError::config_error(format!("Failed to create directory {:?}: {}", dir, e)))?;
    }
    
    write_response(json!({"initialized": true}))?;
    
    Ok(())
}

/// Start the AeroDB server
///
/// Per BOOT.md §3, startup sequence:
/// 1. Configuration Load
/// 2. Schema Load
/// 3. Recovery (WAL replay)
/// 4. Index Rebuild
/// 5. Verification
/// 6. API Activation
///
/// Then enters SERVING loop reading JSON from stdin.
pub fn start(config_path: &Path) -> CliResult<()> {
    let config = Config::load(config_path)?;
    let data_dir = config.data_path();
    
    // Check if initialized
    if !is_initialized(data_dir) {
        return Err(CliError::not_initialized());
    }
    
    // Boot the system
    let (mut wal_writer, mut storage_writer, mut storage_reader, schema_loader, mut index_manager) = 
        boot_system(data_dir)?;
    
    // Initialize API handler
    let handler = ApiHandler::new("default");
    
    // Enter SERVING loop
    // Read JSON from stdin line-by-line, write response to stdout
    for request_result in read_requests() {
        match request_result {
            Ok(request) => {
                let request_str = request.to_string();
                
                let mut subsystems = Subsystems {
                    schema_loader: &schema_loader,
                    wal_writer: &mut wal_writer,
                    storage_writer: &mut storage_writer,
                    storage_reader: &mut storage_reader,
                    index_manager: &mut index_manager,
                };
                
                let response = handler.handle(&request_str, &mut subsystems);
                write_json(&response.to_json())?;
            }
            Err(e) => {
                // I/O error reading - this is fatal
                write_error(e.code_str(), e.message())?;
                break;
            }
        }
    }
    
    // Clean shutdown - write marker
    let shutdown_marker = data_dir.join("clean_shutdown");
    let _ = fs::write(&shutdown_marker, "");
    
    Ok(())
}

/// Execute a single query and exit
///
/// Per CLI spec: Full boot → Execute single query → Print result → Exit
pub fn query(config_path: &Path) -> CliResult<()> {
    let config = Config::load(config_path)?;
    let data_dir = config.data_path();
    
    // Check if initialized
    if !is_initialized(data_dir) {
        return Err(CliError::not_initialized());
    }
    
    // Boot the system
    let (mut wal_writer, mut storage_writer, mut storage_reader, schema_loader, mut index_manager) = 
        boot_system(data_dir)?;
    
    // Read single request from stdin
    let request = read_request()?;
    
    // Ensure it's a query operation
    let mut request_obj = request;
    if let Some(obj) = request_obj.as_object_mut() {
        // Allow any operation, but this command is intended for query
        if obj.get("op").and_then(|v| v.as_str()) != Some("query") {
            // Force query operation if not specified
            if !obj.contains_key("op") {
                obj.insert("op".to_string(), json!("query"));
            }
        }
    }
    
    let request_str = request_obj.to_string();
    
    // Initialize API handler
    let handler = ApiHandler::new("default");
    
    let mut subsystems = Subsystems {
        schema_loader: &schema_loader,
        wal_writer: &mut wal_writer,
        storage_writer: &mut storage_writer,
        storage_reader: &mut storage_reader,
        index_manager: &mut index_manager,
    };
    
    let response = handler.handle(&request_str, &mut subsystems);
    write_json(&response.to_json())?;
    
    Ok(())
}

/// Execute explain on a query and exit
///
/// Same as query, but forces "op":"explain"
pub fn explain(config_path: &Path) -> CliResult<()> {
    let config = Config::load(config_path)?;
    let data_dir = config.data_path();
    
    // Check if initialized
    if !is_initialized(data_dir) {
        return Err(CliError::not_initialized());
    }
    
    // Boot the system
    let (mut wal_writer, mut storage_writer, mut storage_reader, schema_loader, mut index_manager) = 
        boot_system(data_dir)?;
    
    // Read single request from stdin
    let request = read_request()?;
    
    // Force explain operation
    let mut request_obj = request;
    if let Some(obj) = request_obj.as_object_mut() {
        obj.insert("op".to_string(), json!("explain"));
    }
    
    let request_str = request_obj.to_string();
    
    // Initialize API handler
    let handler = ApiHandler::new("default");
    
    let mut subsystems = Subsystems {
        schema_loader: &schema_loader,
        wal_writer: &mut wal_writer,
        storage_writer: &mut storage_writer,
        storage_reader: &mut storage_reader,
        index_manager: &mut index_manager,
    };
    
    let response = handler.handle(&request_str, &mut subsystems);
    write_json(&response.to_json())?;
    
    Ok(())
}

/// Check if a data directory is initialized
fn is_initialized(data_dir: &Path) -> bool {
    data_dir.join("wal").exists() &&
    data_dir.join("data").exists() &&
    data_dir.join("metadata").join("schemas").exists()
}

/// Boot the system per BOOT.md with mandatory recovery
///
/// Steps (strict order, all mandatory):
/// 1. Load schemas
/// 2. Open WAL reader for replay
/// 3. Open recovery storage (combined writer + scanner)
/// 4. Execute RecoveryManager::recover() which:
///    - Replays WAL from offset 0
///    - Applies all records to storage
///    - Rebuilds indexes from storage
///    - Verifies consistency
///    - Removes clean_shutdown marker
/// 5. Return initialized subsystems
///
/// FATAL: Any failure at any step halts startup immediately.
/// No partial startup. No serving without complete recovery.
fn boot_system(data_dir: &Path) -> CliResult<(WalWriter, StorageWriter, StorageReader, SchemaLoader, IndexManager)> {
    use crate::recovery::RecoveryStorage;
    
    // Step 1: Load schemas (required for schema validation during recovery)
    let mut schema_loader = SchemaLoader::new(data_dir);
    schema_loader.load_all()
        .map_err(|e| CliError::boot_failed(format!("Schema load failed: {}", e)))?;
    
    // Step 2: Open WAL reader for replay
    let wal_path = data_dir.join("wal").join("wal.log");
    let wal_exists = wal_path.exists();
    
    // Step 3: Create index manager
    let indexed_fields: HashSet<String> = HashSet::new();
    let mut index_manager = IndexManager::new(indexed_fields);
    
    // Step 4: Execute RecoveryManager::recover() - MANDATORY
    // This performs: WAL replay -> Index rebuild -> Consistency verification
    let recovery_manager = RecoveryManager::new(data_dir);
    
    let (storage_writer, storage_reader) = if wal_exists {
        // Open WAL reader
        let mut wal_reader = WalReader::open(&wal_path)
            .map_err(|e| CliError::boot_failed(format!("WAL reader open failed: {}", e)))?;
        
        // Open recovery storage (implements both StorageApply + StorageScan)
        let mut recovery_storage = RecoveryStorage::open(data_dir)
            .map_err(|e| CliError::boot_failed(format!("Recovery storage open failed: {}", e)))?;
        
        // Execute full recovery sequence
        // This MUST succeed before we can serve any requests
        let _recovery_state = recovery_manager.recover(
            &mut wal_reader,
            &mut recovery_storage,
            &mut index_manager,
            &schema_loader,
        ).map_err(|e| {
            // Recovery failure is FATAL - system cannot serve
            CliError::boot_failed(format!(
                "Recovery failed (FATAL): {}. System cannot serve requests.",
                e
            ))
        })?;
        
        // Extract writer and reader from recovery storage
        recovery_storage.into_parts()
    } else {
        // No WAL file exists - fresh database
        // Still need to remove shutdown marker if present
        let shutdown_marker = data_dir.join("clean_shutdown");
        if shutdown_marker.exists() {
            fs::remove_file(&shutdown_marker)
                .map_err(|e| CliError::boot_failed(format!("Failed to remove shutdown marker: {}", e)))?;
        }
        
        // Open storage directly
        let storage_writer = StorageWriter::open(data_dir)
            .map_err(|e| CliError::boot_failed(format!("Storage writer open failed: {}", e)))?;
        let storage_reader = StorageReader::open_from_data_dir(data_dir)
            .map_err(|e| CliError::boot_failed(format!("Storage reader open failed: {}", e)))?;
        
        (storage_writer, storage_reader)
    };
    
    // Step 5: Open WAL writer for new writes
    let wal_writer = WalWriter::open(data_dir)
        .map_err(|e| CliError::boot_failed(format!("WAL writer open failed: {}", e)))?;
    
    // Recovery complete - system may now enter SERVING state
    Ok((wal_writer, storage_writer, storage_reader, schema_loader, index_manager))
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::errors::CliErrorCode;
    use std::fs;
    use tempfile::TempDir;
    
    fn create_config(temp_dir: &TempDir) -> std::path::PathBuf {
        let config_path = temp_dir.path().join("aerodb.json");
        let data_dir = temp_dir.path().join("data");
        
        let config = json!({
            "data_dir": data_dir.to_string_lossy()
        });
        
        fs::write(&config_path, config.to_string()).unwrap();
        config_path
    }
    
    #[test]
    fn test_init_creates_directories() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = create_config(&temp_dir);
        let data_dir = temp_dir.path().join("data");
        
        // Init should succeed
        init(&config_path).unwrap();
        
        // Check directories exist
        assert!(data_dir.join("wal").exists());
        assert!(data_dir.join("data").exists());
        assert!(data_dir.join("metadata").join("schemas").exists());
    }
    
    #[test]
    fn test_init_refuses_reinit() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = create_config(&temp_dir);
        
        // First init succeeds
        init(&config_path).unwrap();
        
        // Second init fails
        let result = init(&config_path);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), &CliErrorCode::AlreadyInitialized);
    }
    
    #[test]
    fn test_start_requires_init() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = create_config(&temp_dir);
        
        // Start without init fails
        let result = start(&config_path);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), &CliErrorCode::NotInitialized);
    }
    
    #[test]
    fn test_config_validates_sync_mode() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("aerodb.json");
        let data_dir = temp_dir.path().join("data");
        
        let config = json!({
            "data_dir": data_dir.to_string_lossy(),
            "wal_sync_mode": "none"  // Invalid!
        });
        
        fs::write(&config_path, config.to_string()).unwrap();
        
        let result = Config::load(&config_path);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_config_defaults() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("aerodb.json");
        let data_dir = temp_dir.path().join("data");
        
        let config_json = json!({
            "data_dir": data_dir.to_string_lossy()
        });
        
        fs::write(&config_path, config_json.to_string()).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        assert_eq!(config.max_wal_size_bytes, 1073741824);
        assert_eq!(config.max_memory_bytes, 536870912);
        assert_eq!(config.wal_sync_mode, "fsync");
    }
}
