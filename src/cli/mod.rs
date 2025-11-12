//! CLI module for AeroDB
//!
//! Provides command-line interface for:
//! - init: Create directory structure
//! - start: Boot system and enter serving loop
//! - query: One-shot query execution
//! - explain: One-shot explain execution

mod args;
mod commands;
mod errors;
mod io;

pub use args::{Cli, Command};
pub use commands::{run_command, init, start, query, explain};
pub use errors::{CliError, CliResult};
pub use io::{read_request, write_response, write_error};
