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

use std::{
    net::{IpAddr, SocketAddr},
    path::PathBuf,
};

use getset::Getters;
use microsandbox_utils::{env, NAMESPACES_SUBDIR};
use serde::Deserialize;

use crate::{MicrosandboxServerError, MicrosandboxServerResult};

//--------------------------------------------------------------------------------------------------
// Constants
//--------------------------------------------------------------------------------------------------

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

    /// Host address to listen on
    host: IpAddr,

    /// Port number to listen on
    port: u16,

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
        host: String,
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

        // Parse host string to IpAddr
        let host_ip: IpAddr = host.parse().map_err(|_| {
            MicrosandboxServerError::ConfigError(format!("Invalid host address: {}", host))
        })?;

        let addr = SocketAddr::new(host_ip, port);
        let namespace_dir = namespace_dir
            .unwrap_or_else(|| env::get_microsandbox_home_path().join(NAMESPACES_SUBDIR));

        Ok(Self {
            key,
            namespace_dir,
            dev_mode,
            host: host_ip,
            port,
            addr,
        })
    }
}
