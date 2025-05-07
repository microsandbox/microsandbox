//! `microsandbox_utils::path` is a module containing path utilities for the microsandbox project.

use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use typed_path::{Utf8UnixComponent, Utf8UnixPathBuf};

use crate::{MicrosandboxUtilsError, MicrosandboxUtilsResult};

//--------------------------------------------------------------------------------------------------
// Constants
//--------------------------------------------------------------------------------------------------

/// The directory name for microsandbox's project-specific data
pub const MICROSANDBOX_ENV_DIR: &str = ".menv";

/// The directory name for microsandbox's global data
pub const MICROSANDBOX_HOME_DIR: &str = ".microsandbox";

/// The directory where project read-write layers are stored
///
/// Example: <PROJECT_ROOT>/<MICROSANDBOX_ENV_DIR>/<RW_SUBDIR>
pub const RW_SUBDIR: &str = "rw";

/// The directory where project patch layers are stored
///
/// Example: <PROJECT_ROOT>/<MICROSANDBOX_ENV_DIR>/<PATCH_SUBDIR>
pub const PATCH_SUBDIR: &str = "patch";

/// The directory where project logs are stored
///
/// Example: <PROJECT_ROOT>/<MICROSANDBOX_ENV_DIR>/<LOG_SUBDIR>
pub const LOG_SUBDIR: &str = "log";

/// The directory where global image layers are stored
///
/// Example: <MICROSANDBOX_HOME_DIR>/<LAYERS_SUBDIR>
pub const LAYERS_SUBDIR: &str = "layers";

/// The directory where installed sandboxes are stored
///
/// Example: <MICROSANDBOX_HOME_DIR>/<INSTALLS_SUBDIR>
pub const INSTALLS_SUBDIR: &str = "installs";

/// The filename for the project active sandbox database
///
/// Example: <PROJECT_ROOT>/<MICROSANDBOX_ENV_DIR>/<SANDBOX_DB_FILENAME>
pub const SANDBOX_DB_FILENAME: &str = "sandbox.db";

/// The filename for the global OCI database
///
/// Example: <MICROSANDBOX_HOME_DIR>/<OCI_DB_FILENAME>
pub const OCI_DB_FILENAME: &str = "oci.db";

/// The directory on the microvm where sandbox scripts are stored
pub const SANDBOX_DIR: &str = ".sandbox";

/// The directory on the microvm where sandbox scripts are stored
///
/// Example: <SANDBOX_DIR>/<SCRIPTS_DIR>
pub const SCRIPTS_DIR: &str = "scripts";

/// The suffix added to extracted layer directories
///
/// Example: <MICROSANDBOX_HOME_DIR>/<LAYERS_SUBDIR>/<LAYER_ID>.<EXTRACTED_LAYER_SUFFIX>
pub const EXTRACTED_LAYER_SUFFIX: &str = "extracted";

/// The microsandbox config file name.
///
/// Example: <PROJECT_ROOT>/<MICROSANDBOX_ENV_DIR>/<SANDBOX_DB_FILENAME>
pub const MICROSANDBOX_CONFIG_FILENAME: &str = "Sandboxfile";

/// The shell script name.
///
/// Example: <PROJECT_ROOT>/<MICROSANDBOX_ENV_DIR>/<PATCH_SUBDIR>/<CONFIG_NAME>/<SHELL_SCRIPT_NAME>
pub const SHELL_SCRIPT_NAME: &str = "shell";

/// The directory where namespaces are stored
///
/// Example: <MICROSANDBOX_HOME_DIR>/<NAMESPACES_SUBDIR>
pub const NAMESPACES_SUBDIR: &str = "namespaces";

/// The PID file for the server
///
/// Example: <MICROSANDBOX_HOME_DIR>/<SERVER_PID_FILE>
pub const SERVER_PID_FILE: &str = "server.pid";

/// The server secret key file
///
/// Example: <MICROSANDBOX_HOME_DIR>/<SERVER_KEY_FILE>
pub const SERVER_KEY_FILE: &str = "server.key";

/// The file where sandbox portal ports are stored
///
/// Example: <MICROSANDBOX_HOME_DIR>/<NAMESPACE_SUBDIR>/<PORTAL_PORTS_FILE>
pub const PORTAL_PORTS_FILE: &str = "portal.ports";

/// The XDG home directory
///
/// Example: <HOME>/.local
pub static XDG_HOME_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| dirs::home_dir().unwrap().join(".local"));

/// The bin subdirectory for microsandbox
///
/// Example: <XDG_HOME_DIR>/bin
pub const XDG_BIN_DIR: &str = "bin";

/// The lib subdirectory for microsandbox
///
/// Example: <XDG_HOME_DIR>/lib
pub const XDG_LIB_DIR: &str = "lib";

/// The suffix for log files
pub const LOG_SUFFIX: &str = "log";

/// The filename for the supervisor's log file
pub const SUPERVISOR_LOG_FILENAME: &str = "supervisor.log";

//--------------------------------------------------------------------------------------------------
// Types
//--------------------------------------------------------------------------------------------------

/// The type of a supported path.
pub enum SupportedPathType {
    /// Any path type.
    Any,

    /// An absolute path.
    Absolute,

    /// A relative path.
    Relative,
}

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

/// Normalizes a path string for volume mount comparison.
///
/// Rules:
/// - Resolves . and .. components where possible
/// - Prevents path traversal that would escape the root
/// - Removes redundant separators and trailing slashes
/// - Case-sensitive comparison (Unix standard)
/// - Can enforce path type requirements (absolute, relative, or any)
///
/// # Arguments
/// * `path` - The path to normalize
/// * `path_type` - The required path type (absolute, relative, or any)
///
/// # Returns
/// An error if the path is invalid, would escape root, or doesn't meet path type requirement
pub fn normalize_path(path: &str, path_type: SupportedPathType) -> MicrosandboxUtilsResult<String> {
    if path.is_empty() {
        return Err(MicrosandboxUtilsError::PathValidation(
            "Path cannot be empty".to_string(),
        ));
    }

    let path = Utf8UnixPathBuf::from(path);
    let mut normalized = Vec::new();
    let mut is_absolute = false;
    let mut depth = 0;

    for component in path.components() {
        match component {
            // Root component must come first if present
            Utf8UnixComponent::RootDir => {
                if normalized.is_empty() {
                    is_absolute = true;
                    normalized.push("/".to_string());
                } else {
                    return Err(MicrosandboxUtilsError::PathValidation(
                        "Invalid path: root component '/' found in middle of path".to_string(),
                    ));
                }
            }
            // Handle parent directory references
            Utf8UnixComponent::ParentDir => {
                if depth > 0 {
                    // Can go up if we have depth
                    normalized.pop();
                    depth -= 1;
                } else {
                    // Trying to go above root
                    return Err(MicrosandboxUtilsError::PathValidation(
                        "Invalid path: cannot traverse above root directory".to_string(),
                    ));
                }
            }
            // Skip current dir components
            Utf8UnixComponent::CurDir => continue,
            // Normal components are fine
            Utf8UnixComponent::Normal(c) => {
                if !c.is_empty() {
                    normalized.push(c.to_string());
                    depth += 1;
                }
            }
        }
    }

    // Check path type requirements
    match path_type {
        SupportedPathType::Absolute if !is_absolute => {
            return Err(MicrosandboxUtilsError::PathValidation(
                "Path must be absolute (start with '/')".to_string(),
            ));
        }
        SupportedPathType::Relative if is_absolute => {
            return Err(MicrosandboxUtilsError::PathValidation(
                "Path must be relative (must not start with '/')".to_string(),
            ));
        }
        _ => {}
    }

    if is_absolute {
        if normalized.len() == 1 {
            // Just root
            Ok("/".to_string())
        } else {
            // Join all components with "/" and add root at start
            Ok(format!("/{}", normalized[1..].join("/")))
        }
    } else {
        // For relative paths, just join all components
        Ok(normalized.join("/"))
    }
}

/// Resolves the path to a file, checking both environment variable and default locations.
///
/// First checks the environment variable specified by `env_var`.
/// If that's not set, falls back to `default_path`.
/// Returns an error if the file is not found at the resolved location.
pub fn resolve_env_path(
    env_var: &str,
    default_path: impl AsRef<Path>,
) -> MicrosandboxUtilsResult<PathBuf> {
    let (path, source) = std::env::var(env_var)
        .map(|p| (PathBuf::from(p), "environment variable"))
        .unwrap_or_else(|_| (default_path.as_ref().to_path_buf(), "default path"));

    if !path.exists() {
        return Err(MicrosandboxUtilsError::FileNotFound(
            path.to_string_lossy().to_string(),
            source.to_string(),
        ));
    }

    Ok(path)
}

//--------------------------------------------------------------------------------------------------
// Tests
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path() {
        // Test with SupportedPathType::Absolute
        assert_eq!(
            normalize_path("/data/app/", SupportedPathType::Absolute).unwrap(),
            "/data/app"
        );
        assert_eq!(
            normalize_path("/data//app", SupportedPathType::Absolute).unwrap(),
            "/data/app"
        );
        assert_eq!(
            normalize_path("/data/./app", SupportedPathType::Absolute).unwrap(),
            "/data/app"
        );

        // Test with SupportedPathType::Relative
        assert_eq!(
            normalize_path("data/app/", SupportedPathType::Relative).unwrap(),
            "data/app"
        );
        assert_eq!(
            normalize_path("./data/app", SupportedPathType::Relative).unwrap(),
            "data/app"
        );
        assert_eq!(
            normalize_path("data//app", SupportedPathType::Relative).unwrap(),
            "data/app"
        );

        // Test with SupportedPathType::Any
        assert_eq!(
            normalize_path("/data/app", SupportedPathType::Any).unwrap(),
            "/data/app"
        );
        assert_eq!(
            normalize_path("data/app", SupportedPathType::Any).unwrap(),
            "data/app"
        );

        // Path traversal within bounds
        assert_eq!(
            normalize_path("/data/temp/../app", SupportedPathType::Absolute).unwrap(),
            "/data/app"
        );
        assert_eq!(
            normalize_path("data/temp/../app", SupportedPathType::Relative).unwrap(),
            "data/app"
        );

        // Invalid paths
        assert!(matches!(
            normalize_path("data/app", SupportedPathType::Absolute),
            Err(MicrosandboxUtilsError::PathValidation(e)) if e.contains("must be absolute")
        ));
        assert!(matches!(
            normalize_path("/data/app", SupportedPathType::Relative),
            Err(MicrosandboxUtilsError::PathValidation(e)) if e.contains("must be relative")
        ));
        assert!(matches!(
            normalize_path("/data/../..", SupportedPathType::Any),
            Err(MicrosandboxUtilsError::PathValidation(e)) if e.contains("cannot traverse above root")
        ));
    }

    #[test]
    fn test_normalize_path_complex() {
        // Complex but valid paths
        assert_eq!(
            normalize_path(
                "/data/./temp/../logs/app/./config/../",
                SupportedPathType::Absolute
            )
            .unwrap(),
            "/data/logs/app"
        );
        assert_eq!(
            normalize_path(
                "/data///temp/././../app//./test/..",
                SupportedPathType::Absolute
            )
            .unwrap(),
            "/data/app"
        );

        // Edge cases
        assert_eq!(
            normalize_path("/data/./././.", SupportedPathType::Absolute).unwrap(),
            "/data"
        );
        assert_eq!(
            normalize_path("/data/test/../../data/app", SupportedPathType::Absolute).unwrap(),
            "/data/app"
        );

        // Invalid complex paths
        assert!(matches!(
            normalize_path("/data/test/../../../root", SupportedPathType::Any),
            Err(MicrosandboxUtilsError::PathValidation(e)) if e.contains("cannot traverse above root")
        ));
        assert!(matches!(
            normalize_path("/./data/../..", SupportedPathType::Any),
            Err(MicrosandboxUtilsError::PathValidation(e)) if e.contains("cannot traverse above root")
        ));
    }
}
