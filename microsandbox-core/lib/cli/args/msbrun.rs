use std::path::PathBuf;

use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};

use crate::cli::styles;

//--------------------------------------------------------------------------------------------------
// Types
//--------------------------------------------------------------------------------------------------

/// Arguments for the msbrun command
#[derive(Debug, Parser)]
#[command(name = "msbrun", author, styles=styles::styles())]
pub struct McrunArgs {
    /// The subcommand to run
    #[command(subcommand)]
    pub subcommand: McrunSubcommand,
}

/// Available subcommands for managing microvms
#[derive(Subcommand, Debug)]
pub enum McrunSubcommand {
    /// Run as microvm
    Microvm {
        /// Log level
        #[arg(long)]
        log_level: Option<u8>,

        /// Native root filesystem path
        #[arg(long)]
        native_rootfs: Option<PathBuf>,

        /// Overlayfs root filesystem layers
        #[arg(long)]
        overlayfs_layer: Vec<PathBuf>,

        /// Number of virtual CPUs
        #[arg(long)]
        num_vcpus: Option<u8>,

        /// RAM size in MiB
        #[arg(long)]
        ram_mib: Option<u32>,

        /// Working directory path
        #[arg(long)]
        workdir_path: Option<String>,

        /// Executable path
        #[arg(long, required = true)]
        exec_path: String,

        /// Environment variables (KEY=VALUE format)
        #[arg(long)]
        env: Vec<String>,

        /// Directory mappings (host:guest format)
        #[arg(long)]
        mapped_dir: Vec<String>,

        /// Port mappings (host:guest format)
        #[arg(long)]
        port_map: Vec<String>,

        /// Network communication scope
        #[arg(long)]
        scope: Option<String>,

        /// Assigned IP address
        #[arg(long)]
        ip: Option<String>,

        /// Assigned subnet
        #[arg(long)]
        subnet: Option<String>,

        /// Additional arguments after `--`
        #[arg(last = true)]
        args: Vec<String>,
    },
    /// Run as supervisor
    Supervisor {
        /// Directory for log files
        #[arg(long)]
        log_dir: PathBuf,

        /// Path to the sandbox metrics and metadata database file
        #[arg(long)]
        sandbox_db_path: PathBuf,

        /// Name of the child process
        #[arg(long)]
        sandbox_name: String,

        /// Path to the sandbox config file
        #[arg(long)]
        config_file: String,

        /// Last modified timestamp of the sandbox config file
        #[arg(long)]
        config_last_modified: DateTime<Utc>,

        /// Log level
        #[arg(long)]
        log_level: Option<u8>,

        /// Whether to forward output to stdout/stderr
        #[arg(long, default_value = "true")]
        forward_output: bool,

        // Sandbox specific arguments
        /// Native root filesystem path
        #[arg(long)]
        native_rootfs: Option<PathBuf>,

        /// Overlayfs root filesystem layers
        #[arg(long)]
        overlayfs_layer: Vec<PathBuf>,

        /// Number of virtual CPUs
        #[arg(long)]
        num_vcpus: Option<u8>,

        /// RAM size in MiB
        #[arg(long)]
        ram_mib: Option<u32>,

        /// Working directory path
        #[arg(long)]
        workdir_path: Option<String>,

        /// Executable path
        #[arg(long, required = true)]
        exec_path: String,

        /// Environment variables (KEY=VALUE format)
        #[arg(long)]
        env: Vec<String>,

        /// Directory mappings (host:guest format)
        #[arg(long)]
        mapped_dir: Vec<String>,

        /// Port mappings (host:guest format)
        #[arg(long)]
        port_map: Vec<String>,

        /// Network communication scope
        #[arg(long)]
        scope: Option<String>,

        /// Assigned IP address
        #[arg(long)]
        ip: Option<String>,

        /// Assigned subnet
        #[arg(long)]
        subnet: Option<String>,

        /// Additional arguments after `--`
        #[arg(last = true)]
        args: Vec<String>,
    },
    /// Start the sandbox server
    Server {
        /// Port to listen on
        #[arg(long)]
        port: Option<u16>,

        /// Path to the namespace directory
        #[arg(long)]
        path: Option<PathBuf>,

        /// Disable default namespace
        #[arg(long, default_value_t = false)]
        disable_default: bool,

        /// Set server secret key to authenticate API requests
        #[arg(long)]
        key: Option<String>,
    },
}
