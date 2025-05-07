//! Port management for the microsandbox server.
//!
//! This module handles port assignment and management for sandboxes:
//! - Assigns truly available ports obtained from the OS
//! - Tracks assigned ports for fast lookup
//! - Persists port assignments to disk
//! - Loads existing port assignments on startup
//! - Handles port uniqueness with bidirectional mapping
//!
//! The module provides:
//! - Port manager for tracking assigned ports
//! - Functions for assigning and releasing ports
//! - File-based persistence of port assignments

use microsandbox_utils::PORTAL_PORTS_FILE;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener},
    path::{Path, PathBuf},
};
use tokio::{fs, sync::Mutex};
use tracing::{debug, info, warn};

use crate::{MicrosandboxServerError, MicrosandboxServerResult};

//--------------------------------------------------------------------------------------------------
// Constants
//--------------------------------------------------------------------------------------------------

/// The localhost IP address used for all portal connections
pub const LOCALHOST_IP: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);

/// Lock to ensure only one thread gets a port at a time
static PORT_ASSIGNMENT_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

//--------------------------------------------------------------------------------------------------
// Types
//--------------------------------------------------------------------------------------------------

/// Port mapping for sandbox instances - bidirectional for fast lookups
#[derive(Debug, Clone, Default)]
pub struct BiPortMapping {
    /// Maps sandbox identifiers (namespace/sandboxname) to assigned port numbers
    sandbox_to_port: HashMap<String, u16>,

    /// Maps port numbers to sandbox identifiers for fast reverse lookup
    port_to_sandbox: HashMap<u16, String>,
}

/// Serializable version of the port mapping for file storage
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PortMapping {
    /// Maps sandbox identifiers (namespace/sandboxname) to assigned port numbers
    pub mappings: HashMap<String, u16>,
}

/// Port manager for handling sandbox port assignments
#[derive(Debug)]
pub struct PortManager {
    /// The port mappings data
    mappings: BiPortMapping,

    /// Path to the port mappings file
    file_path: PathBuf,
}

//--------------------------------------------------------------------------------------------------
// Methods
//--------------------------------------------------------------------------------------------------

impl BiPortMapping {
    /// Create a new bidirectional port mapping
    pub fn new() -> Self {
        Self {
            sandbox_to_port: HashMap::new(),
            port_to_sandbox: HashMap::new(),
        }
    }

    /// Insert a mapping, handling any existing mappings for the port or sandbox
    pub fn insert(&mut self, sandbox_key: String, port: u16) {
        // Check if this port is already assigned to a different sandbox
        if let Some(existing_sandbox) = self.port_to_sandbox.get(&port) {
            if existing_sandbox != &sandbox_key {
                // Port is already assigned to a different sandbox - remove that mapping
                warn!(
                    "Port {} was already assigned to sandbox {}, reassigning to {}",
                    port, existing_sandbox, sandbox_key
                );
                self.sandbox_to_port.remove(existing_sandbox);
            }
        }

        // Check if this sandbox already has a different port
        if let Some(existing_port) = self.sandbox_to_port.get(&sandbox_key) {
            if *existing_port != port {
                // Sandbox had a different port - remove that mapping
                self.port_to_sandbox.remove(existing_port);
            }
        }

        // Insert the new mapping in both directions
        self.sandbox_to_port.insert(sandbox_key.clone(), port);
        self.port_to_sandbox.insert(port, sandbox_key);
    }

    /// Remove a mapping by sandbox key
    pub fn remove_by_sandbox(&mut self, sandbox_key: &str) -> Option<u16> {
        if let Some(port) = self.sandbox_to_port.remove(sandbox_key) {
            self.port_to_sandbox.remove(&port);
            Some(port)
        } else {
            None
        }
    }

    /// Remove a mapping by port
    pub fn remove_by_port(&mut self, port: u16) -> Option<String> {
        if let Some(sandbox_key) = self.port_to_sandbox.remove(&port) {
            self.sandbox_to_port.remove(&sandbox_key);
            Some(sandbox_key)
        } else {
            None
        }
    }

    /// Get port by sandbox key
    pub fn get_port(&self, sandbox_key: &str) -> Option<u16> {
        self.sandbox_to_port.get(sandbox_key).copied()
    }

    /// Get sandbox key by port
    pub fn get_sandbox(&self, port: u16) -> Option<&String> {
        self.port_to_sandbox.get(&port)
    }

    /// Convert to serializable format
    pub fn to_port_mapping(&self) -> PortMapping {
        PortMapping {
            mappings: self.sandbox_to_port.clone(),
        }
    }

    /// Load from serializable format
    pub fn from_port_mapping(mapping: PortMapping) -> Self {
        let mut result = Self::new();

        for (sandbox_key, port) in mapping.mappings {
            result.insert(sandbox_key, port);
        }

        result
    }
}

impl PortManager {
    /// Create a new port manager
    pub async fn new(namespace_dir: impl AsRef<Path>) -> MicrosandboxServerResult<Self> {
        let file_path = namespace_dir.as_ref().join(PORTAL_PORTS_FILE);
        let mappings = Self::load_mappings(&file_path).await?;

        Ok(Self {
            mappings,
            file_path,
        })
    }

    /// Load port mappings from file
    async fn load_mappings(file_path: &Path) -> MicrosandboxServerResult<BiPortMapping> {
        if file_path.exists() {
            let contents = fs::read_to_string(file_path).await.map_err(|e| {
                MicrosandboxServerError::ConfigError(format!(
                    "Failed to read port mappings file: {}",
                    e
                ))
            })?;

            let port_mapping: PortMapping = serde_json::from_str(&contents).map_err(|e| {
                MicrosandboxServerError::ConfigError(format!(
                    "Failed to parse port mappings file: {}",
                    e
                ))
            })?;

            Ok(BiPortMapping::from_port_mapping(port_mapping))
        } else {
            debug!("No port mappings file found, creating a new one");
            Ok(BiPortMapping::new())
        }
    }

    /// Save port mappings to file
    async fn save_mappings(&self) -> MicrosandboxServerResult<()> {
        let port_mapping = self.mappings.to_port_mapping();
        let contents = serde_json::to_string_pretty(&port_mapping).map_err(|e| {
            MicrosandboxServerError::ConfigError(format!(
                "Failed to serialize port mappings: {}",
                e
            ))
        })?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = self.file_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).await.map_err(|e| {
                    MicrosandboxServerError::ConfigError(format!(
                        "Failed to create directory for port mappings file: {}",
                        e
                    ))
                })?;
            }
        }

        fs::write(&self.file_path, contents).await.map_err(|e| {
            MicrosandboxServerError::ConfigError(format!(
                "Failed to write port mappings file: {}",
                e
            ))
        })
    }

    /// Assign a port to a sandbox
    pub async fn assign_port(&mut self, key: &str) -> MicrosandboxServerResult<u16> {
        // Check if port is already assigned
        if let Some(port) = self.mappings.get_port(key) {
            // Verify this port is still available
            if self.verify_port_availability(port) {
                return Ok(port);
            } else {
                // Port is no longer available, so we need to assign a new one
                warn!("Previously assigned port {} for sandbox {} is no longer available, reassigning", port, key);
                self.mappings.remove_by_sandbox(key);
            }
        }

        // Get a lock to ensure only one thread gets a port at a time
        let _lock = PORT_ASSIGNMENT_LOCK.lock().await;

        // Get a truly available port from the OS
        let port = self.get_available_port_from_os()?;

        // Save the mapping
        self.mappings.insert(key.to_string(), port);
        self.save_mappings().await?;

        info!("Assigned port {} to sandbox {}", port, key);
        Ok(port)
    }

    /// Release a port assignment
    pub async fn release_port(&mut self, key: &str) -> MicrosandboxServerResult<()> {
        if self.mappings.remove_by_sandbox(key).is_some() {
            self.save_mappings().await?;
            info!("Released port for sandbox {}", key);
        }

        Ok(())
    }

    /// Get a port for a sandbox if assigned
    pub fn get_port(&self, key: &str) -> Option<u16> {
        self.mappings.get_port(key)
    }

    /// Verify that a port is still available (not bound by something else)
    fn verify_port_availability(&self, port: u16) -> bool {
        let addr = SocketAddr::new(LOCALHOST_IP, port);
        match TcpListener::bind(addr) {
            Ok(_) => true,   // We could bind, so it's available
            Err(_) => false, // We couldn't bind, so it's not available
        }
    }

    /// Get an available port from the OS
    fn get_available_port_from_os(&self) -> MicrosandboxServerResult<u16> {
        // Bind to port 0 to let the OS assign an available port
        let addr = SocketAddr::new(LOCALHOST_IP, 0);
        let listener = TcpListener::bind(addr).map_err(|e| {
            MicrosandboxServerError::ConfigError(format!(
                "Failed to bind to address to get available port: {}",
                e
            ))
        })?;

        // Get the port assigned by the OS
        let port = listener
            .local_addr()
            .map_err(|e| {
                MicrosandboxServerError::ConfigError(format!(
                    "Failed to get local address from socket: {}",
                    e
                ))
            })?
            .port();

        debug!("OS assigned port {}", port);

        // The listener will be dropped here, releasing the port
        // We return the port value to be used by the caller

        Ok(port)
    }
}
