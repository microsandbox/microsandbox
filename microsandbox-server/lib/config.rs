//! Configuration module for the microsandbox server.
//!
//! This module handles server configuration including:
//! - Server settings and environment variables
//! - JWT token configuration
//! - Namespace management
//! - Development and production mode settings
//!
//! The module provides:
//! - Configuration structure for server settings
//! - Default values for server configuration
//! - Environment-based configuration loading
//! - Namespace directory management

use std::{net::SocketAddr, path::PathBuf, sync::LazyLock};

use base64::{prelude::BASE64_STANDARD, Engine};
use getset::Getters;
use microsandbox_utils::{env, NAMESPACES_SUBDIR};
use serde::Deserialize;

use crate::{port::LOCALHOST_IP, MicrosandboxServerError, MicrosandboxServerResult};

//--------------------------------------------------------------------------------------------------
// Constants
//--------------------------------------------------------------------------------------------------

/// Default JWT header for HS256 algorithm in base64
pub const DEFAULT_JWT_HEADER: LazyLock<String> =
    LazyLock::new(|| BASE64_STANDARD.encode("{\"typ\":\"JWT\",\"alg\":\"HS256\"}"));

/// The header name for the proxy authorization
pub const PROXY_AUTH_HEADER: &str = "Proxy-Authorization";

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
    ) -> MicrosandboxServerResult<Self> {
        // Check key requirement based on dev mode
        let key = match key {
            Some(k) => Some(k),
            None if dev_mode => None,
            None => {
                return Err(MicrosandboxServerError::ConfigError(
                    "No key provided. A key is required when not in dev mode".to_string(),
                ));
            }
        };

        let addr = SocketAddr::new(LOCALHOST_IP, port);
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
