//! AeroDB CLI entry point
//!
//! Per BOOT.md, main.rs must:
//! 1. Parse args
//! 2. Dispatch to CLI commands
//! 3. Never call subsystems directly

use std::process;

use aerodb::cli::{Cli, run_command};

fn main() {
    let cli = Cli::parse_args();
    
    if let Err(e) = run_command(cli.command) {
        // Print error JSON and exit non-zero
        let error_json = serde_json::json!({
            "status": "error",
            "code": e.code_str(),
            "message": e.message()
        });
        
        eprintln!("{}", error_json);
        process::exit(1);
    }
}
