use std::{net::SocketAddr, path::PathBuf, sync::LazyLock};

use base64::{prelude::BASE64_STANDARD, Engine};
use getset::Getters;
use microsandbox_utils::env;
use serde::Deserialize;

//--------------------------------------------------------------------------------------------------
// Constants
//--------------------------------------------------------------------------------------------------

/// Default port number for the server if not specified in environment variables
pub const DEFAULT_PORT: u16 = 6666;

/// Default namespace name.
pub const DEFAULT_NAMESPACE_NAME: &str = "Default";

/// Default JWT header for HS256 algorithm in base64
pub const DEFAULT_JWT_HEADER: LazyLock<String> =
    LazyLock::new(|| BASE64_STANDARD.encode("{\"typ\":\"JWT\",\"alg\":\"HS256\"}"));

/// The directory for server namespaces
///
/// Example: <MICROSANDBOX_HOME_DIR>/<NAMESPACES_SUBDIR>
pub const NAMESPACES_SUBDIR: &str = "namespaces";

//--------------------------------------------------------------------------------------------------
// Types
//--------------------------------------------------------------------------------------------------

/// Configuration structure that holds all the application settings
/// loaded from environment variables
#[derive(Debug, Deserialize, Getters)]
#[getset(get = "pub with_prefix")]
pub struct Config {
    /// Secret key used for JWT token generation and validation
    key: Option<String>,

    /// Directory for storing namespaces
    namespace_dir: PathBuf,

    /// Whether to run the server in development mode
    dev_mode: bool,

    /// Address to listen on
    addr: SocketAddr,
}

//--------------------------------------------------------------------------------------------------
// Implementations
//--------------------------------------------------------------------------------------------------

impl Config {
    /// Create a new configuration
    pub fn new(
        key: Option<String>,
        port: u16,
        namespace_dir: Option<PathBuf>,
        dev_mode: bool,
    ) -> anyhow::Result<Self> {
        // Check key requirement based on dev mode
        let key = match key {
            Some(k) => Some(k),
            None if dev_mode => None,
            None => anyhow::bail!("No key provided. A key is required when not in dev mode"),
        };

        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let namespace_dir = namespace_dir
            .unwrap_or_else(|| env::get_microsandbox_home_path().join(NAMESPACES_SUBDIR));

        Ok(Self {
            key,
            namespace_dir,
            dev_mode,
            addr,
        })
    }
}
