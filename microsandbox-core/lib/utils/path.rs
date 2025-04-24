//! Utility functions for working with paths.

use microsandbox_utils::SupportedPathType;

use crate::{MicrosandboxError, MicrosandboxResult};

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

/// Checks if two paths conflict (one is a parent/child of the other or they are the same)
pub fn paths_overlap(path1: &str, path2: &str) -> bool {
    let path1 = if path1.ends_with('/') {
        path1.to_string()
    } else {
        format!("{}/", path1)
    };

    let path2 = if path2.ends_with('/') {
        path2.to_string()
    } else {
        format!("{}/", path2)
    };

    path1.starts_with(&path2) || path2.starts_with(&path1)
}

/// Helper function to normalize and validate volume paths
pub fn normalize_volume_path(base_path: &str, requested_path: &str) -> MicrosandboxResult<String> {
    // First normalize both paths
    let normalized_base =
        microsandbox_utils::normalize_path(base_path, SupportedPathType::Absolute)?;

    // If requested path is absolute, verify it's under base_path
    if requested_path.starts_with('/') {
        let normalized_requested =
            microsandbox_utils::normalize_path(requested_path, SupportedPathType::Absolute)?;
        // Check if normalized_requested starts with normalized_base
        if !normalized_requested.starts_with(&normalized_base) {
            return Err(MicrosandboxError::PathValidation(format!(
                "Absolute path '{}' must be under base path '{}'",
                normalized_requested, normalized_base
            )));
        }
        Ok(normalized_requested)
    } else {
        // For relative paths, first normalize the requested path to catch any ../ attempts
        let normalized_requested =
            microsandbox_utils::normalize_path(requested_path, SupportedPathType::Relative)?;

        // Then join with base and normalize again
        let full_path = format!("{}/{}", normalized_base, normalized_requested);
        microsandbox_utils::normalize_path(&full_path, SupportedPathType::Absolute)
            .map_err(Into::into)
    }
}

//--------------------------------------------------------------------------------------------------
// Tests
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paths_overlap() {
        // Test cases that should conflict
        assert!(paths_overlap("/data", "/data"));
        assert!(paths_overlap("/data", "/data/app"));
        assert!(paths_overlap("/data/app", "/data"));
        assert!(paths_overlap("/data/app/logs", "/data/app"));

        // Test cases that should not conflict
        assert!(!paths_overlap("/data", "/database"));
        assert!(!paths_overlap("/var/log", "/var/lib"));
        assert!(!paths_overlap("/data/app1", "/data/app2"));
        assert!(!paths_overlap("/data/app/logs", "/data/web/logs"));
    }
}
