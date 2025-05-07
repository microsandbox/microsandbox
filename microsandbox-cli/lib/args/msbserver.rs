use std::path::PathBuf;

use clap::Parser;
use microsandbox_utils::DEFAULT_SERVER_PORT;

use crate::styles;

//--------------------------------------------------------------------------------------------------
// Types
//--------------------------------------------------------------------------------------------------

/// Arguments for the msbserver command
#[derive(Debug, Parser)]
#[command(name = "msbserver", author, styles=styles::styles())]
pub struct MsbserverArgs {
    /// Secret key used for JWT token generation and validation
    #[arg(short = 'k', long = "key")]
    pub key: Option<String>,

    /// Port number to listen on
    #[arg(long, default_value_t = DEFAULT_SERVER_PORT)]
    pub port: u16,

    /// Directory for storing namespaces
    #[arg(short = 'p', long = "path")]
    pub namespace_dir: Option<PathBuf>,

    /// Run in development mode
    #[arg(long = "dev", default_value_t = false)]
    pub dev_mode: bool,
}
