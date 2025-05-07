//! `microsandbox` is a secure MicroVM provisioning system for running untrusted code in isolated environments.
//!
//! # Overview
//!
//! microsandbox provides a robust foundation for running AI workloads in isolated microVMs. It handles:
//! - VM lifecycle management
//! - OCI image distribution and management
//! - Service orchestration and coordination
//! - Resource constraints and monitoring
//! - Database persistence for system state
//!
//! # Key Features
//!
//! - **Secure Isolation**: True VM-level isolation through microVMs
//! - **Container Experience**: Works with standard OCI/Docker images
//! - **Fast Startup**: Millisecond-level VM provisioning
//! - **Resource Control**: Fine-grained CPU, memory and network limits
//! - **Simple API**: RESTful interface for service management
//! - **Persistence**: Database-backed state management
//!
//! # Architecture
//!
//! microsandbox consists of several key components:
//!
//! - **VM**: Low-level microVM configuration and management
//! - **OCI**: Image pulling, layer handling, and registry interactions
//! - **Management**: Orchestration, sandbox lifecycle, and coordination
//! - **Runtime**: Process supervision and monitoring
//! - **Models**: Database and persistence schema
//!
//! # Modules
//!
//! - [`config`] - Configuration types and validation
//! - [`management`] - Central management for sandboxes, images, and orchestration
//! - [`models`] - Database models and persistence schema
//! - [`oci`] - OCI image and registry operations
//! - [`runtime`] - Process supervision and monitoring
//! - [`utils`] - Common utilities and helpers
//! - [`vm`] - MicroVM configuration and control

#![warn(missing_docs)]

mod error;

//--------------------------------------------------------------------------------------------------
// Exports
//--------------------------------------------------------------------------------------------------

pub mod config;
pub mod management;
pub mod models;
pub mod oci;
pub mod runtime;
pub mod utils;
pub mod vm;

pub use error::*;
