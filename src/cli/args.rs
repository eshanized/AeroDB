//! CLI argument definitions using clap
//!
//! Commands:
//! - aerodb init --config <path>
//! - aerodb start --config <path>
//! - aerodb query --config <path>
//! - aerodb explain --config <path>

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// AeroDB - A strict, deterministic, self-hostable database
#[derive(Parser, Debug)]
#[command(name = "aerodb")]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Initialize a new AeroDB data directory
    Init {
        /// Path to configuration file
        #[arg(long, default_value = "./aerodb.json")]
        config: PathBuf,
    },

    /// Start the AeroDB server
    Start {
        /// Path to configuration file
        #[arg(long, default_value = "./aerodb.json")]
        config: PathBuf,
    },

    /// Execute a single query and exit
    Query {
        /// Path to configuration file
        #[arg(long, default_value = "./aerodb.json")]
        config: PathBuf,
    },

    /// Execute explain on a query and exit
    Explain {
        /// Path to configuration file
        #[arg(long, default_value = "./aerodb.json")]
        config: PathBuf,
    },
}

impl Cli {
    /// Parse command line arguments
    pub fn parse_args() -> Self {
        Cli::parse()
    }
}
